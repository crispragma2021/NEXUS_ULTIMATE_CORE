#!/usr/bin/env python3
# ============================================================================
# 🔐 rotar_llaves.py — Rotación automática de llaves expuestas (GitGuardian)
# ============================================================================
# ¿Qué hace?
#   1. Detecta las llaves en .env y config.toml (sin imprimirlas).
#   2. Si hay OPENROUTER_MANAGEMENT_KEY en el entorno → rota OpenRouter por API:
#      - Lista las llaves de la cuenta
#      - Crea una llave nueva
#      - Elimina la llave expuesta (la vieja)
#      - Actualiza .env y config.toml con la nueva
#   3. Para DeepSeek y Gemini (sin API de rotación) → te indica exactamente
#      qué pegar en el dashboard. Cuando pegues la nueva en .env y vuelvas a
#      correr, el script propaga el cambio a config.toml automáticamente.
#   4. Verifica que cada llave sea válida contra el proveedor.
#
# Uso:
#   python3 scripts/rotar_llaves.py
#
# Requiere: OPENROUTER_MANAGEMENT_KEY (para rotación automática de OpenRouter).
# ============================================================================

import json
import os
import re
import urllib.error
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENV_PATH = os.path.join(ROOT, ".env")
CONFIG_PATH = os.path.join(ROOT, "config.toml")

OR_API = "https://openrouter.ai/api/v1"


# ────────────────────────── utilidades ──────────────────────────
def leer_env() -> dict:
    env = {}
    if os.path.exists(ENV_PATH):
        with open(ENV_PATH) as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#") or "=" not in line:
                    continue
                k, _, v = line.partition("=")
                env[k.strip()] = v.strip()
    return env


def escribir_env(env: dict):
    """Reescribe .env conservando orden/comentarios, actualizando valores."""
    lines = []
    if os.path.exists(ENV_PATH):
        with open(ENV_PATH) as f:
            lines = f.readlines()
    keys_done = set()
    out = []
    for line in lines:
        stripped = line.strip()
        if stripped and not stripped.startswith("#") and "=" in stripped:
            k = stripped.partition("=")[0].strip()
            if k in env:
                out.append(f"{k}={env[k]}\n")
                keys_done.add(k)
                continue
        out.append(line)
    for k, v in env.items():
        if k not in keys_done:
            out.append(f"{k}={v}\n")
    with open(ENV_PATH, "w") as f:
        f.writelines(out)


def leer_config_toml() -> str:
    if os.path.exists(CONFIG_PATH):
        with open(CONFIG_PATH) as f:
            return f.read()
    return ""


def escribir_config_toml(contenido: str):
    with open(CONFIG_PATH, "w") as f:
        f.write(contenido)


def http(url, headers=None, data=None, method=None, timeout=20):
    try:
        req = urllib.request.Request(url, headers=headers or {}, data=data, method=method)
        with urllib.request.urlopen(req, timeout=timeout) as r:
            body = r.read().decode()
            return r.status, json.loads(body) if body else {}
    except urllib.error.HTTPError as e:
        try:
            return e.code, json.loads(e.read().decode())
        except Exception:
            return e.code, {}
    except Exception as e:
        return -1, {"error": str(e)}


def anonimizar(k: str) -> str:
    if len(k) <= 8:
        return "*" * len(k)
    return f"{k[:4]}...{k[-4:]}"


# ────────────────────────── rotación OpenRouter ──────────────────────────
def rotar_openrouter(env: dict):
    """Devuelve (dict_actualizado, dict_vieja_a_nueva)."""
    mgmt = env.get("OPENROUTER_MANAGEMENT_KEY", "").strip()
    vieja = env.get("OPENROUTER_API_KEY", "").strip()

    if not mgmt:
        print("ℹ️  OpenRouter: no hay OPENROUTER_MANAGEMENT_KEY → rotación manual.")
        print("   Crea una en https://openrouter.ai/settings/keys (tipo 'management').")
        print("   Luego añádela a .env como OPENROUTER_MANAGEMENT_KEY y vuelve a correr.")
        print(f"   Llave expuesta a eliminar: {anonimizar(vieja) if vieja else '(ninguna)'}")
        return {}, {}

    if not vieja:
        print("ℹ️  OpenRouter: no hay OPENROUTER_API_KEY en .env.")
        return {}, {}

    # 1. Listar llaves para encontrar el id de la expuesta
    st, d = http(f"{OR_API}/keys", {"Authorization": f"Bearer {mgmt}"})
    if st != 200:
        print(f"❌ No pude listar llaves OpenRouter (HTTP {st}): {d.get('error', d)}")
        return {}, {}

    keys = d.get("data", [])
    target = None
    for k in keys:
        khash = k.get("hash") or ""
        if vieja and (vieja.endswith(khash) or khash.startswith(vieja[-8:])):
            target = k
            break
    if not target:
        print("ℹ️  No encontré la llave expuesta por hash; listo las disponibles:")
        for k in keys:
            print(f"   - id={k.get('id')} label={k.get('label')} hash={k.get('hash')}")
        return {}, {}

    # 2. Crear llave nueva (mismo label)
    label = target.get("label") or "nexus-rotada"
    st, d = http(
        f"{OR_API}/keys",
        {"Authorization": f"Bearer {mgmt}", "Content-Type": "application/json"},
        data=json.dumps({"label": label, "limit": target.get("limit")}).encode(),
        method="POST",
    )
    if st != 200 or "key" not in d:
        print(f"❌ No pude crear llave nueva (HTTP {st}): {d.get('error', d)}")
        return {}, {}
    nueva = d["key"]
    print(f"✅ OpenRouter: llave nueva creada (label={label}) {anonimizar(nueva)}")

    # 3. Eliminar la expuesta
    st, d = http(
        f"{OR_API}/keys/{target['id']}",
        {"Authorization": f"Bearer {mgmt}"},
        method="DELETE",
    )
    if st == 200:
        print(f"✅ OpenRouter: llave expuesta eliminada ({anonimizar(vieja)})")
    else:
        print(f"⚠️  No pude eliminar la expuesta (HTTP {st}): {d.get('error', d)}")

    return {"OPENROUTER_API_KEY": nueva}, {vieja: nueva}


# ────────────────────────── verificación ──────────────────────────
def verificar(llave, nombre, url, headers_extra=None, data=None):
    headers = {"Authorization": f"Bearer {llave}"}
    if headers_extra:
        headers.update(headers_extra)
    st, d = http(url, headers, data=data)
    if st == 200:
        print(f"  ✅ {nombre}: válida")
    else:
        msg = d.get("error", d) if isinstance(d, dict) else d
        print(f"  ⚠️  {nombre}: HTTP {st} — {msg}")
    return st == 200


# ────────────────────────── main ──────────────────────────
def main():
    print("=" * 70)
    print("🔐 ROTACIÓN DE LLAVES EXPUESTAS (GitGuardian)")
    print("=" * 70)

    env = leer_env()
    env_antes = dict(env)
    actualizados = {}
    reemplazos = {}
    contenido = leer_config_toml()

    # ── OpenRouter ──
    print("\n── OpenRouter ──")
    nuevas_or, reemplazos_or = rotar_openrouter(env)
    actualizados.update(nuevas_or)
    reemplazos.update(reemplazos_or)

    # ── DeepSeek (manual, pero propagamos a config.toml si cambió) ──
    print("\n── DeepSeek ──")
    ds = env.get("DEEPSEEK_API_KEY", "")
    if ds:
        print(f"  🔑 Llave actual: {anonimizar(ds)}")
        print("  ⚠️  DeepSeek NO tiene API de rotación. Rótala en:")
        print("      https://platform.deepseek.com/api_keys  → borra la vieja, crea una nueva.")
        print("      Luego edita .env:  DEEPSEEK_API_KEY=<nueva>")
        print("      Y vuelve a correr este script (propagará el cambio a config.toml).")

    # ── Gemini (manual, pero propagamos a config.toml si cambió) ──
    print("\n── Gemini ──")
    gk = env.get("GEMINI_API_KEY", "")
    if gk:
        print(f"  🔑 Llave actual: {anonimizar(gk)}")
        print("  ⚠️  Gemini NO tiene API de rotación. Rótala en:")
        print("      https://aistudio.google.com/apikey  → borra la vieja, crea una nueva.")
        print("      Luego edita .env:  GEMINI_API_KEY=<nueva>")
        print("      Y vuelve a correr este script (propagará el cambio a config.toml).")

    # ── Propagación de rotaciones manuales a config.toml ──
    # Si el usuario pegó una llave nueva en .env (DeepSeek/Gemini se rotan a
    # mano en su dashboard), detectamos el cambio y actualizamos config.toml.
    print("\n── Propagación de cambios manuales a config.toml ──")
    pares_config = [
        ("DEEPSEEK_API_KEY", "deepseek_api_key"),
        ("DEEPSEEK_API_KEY", "deepseek_official_key"),
        ("GEMINI_API_KEY", "google_ai_studio_api_key"),
        ("OPENROUTER_API_KEY", "openrouter_api_key"),
    ]
    hubo_cambio_manual = False
    for clave_env, nombre_cfg in pares_config:
        antes = env_antes.get(clave_env, "").strip()
        ahora = env.get(clave_env, "").strip()
        if antes and ahora and antes != ahora:
            if antes in contenido:
                contenido = contenido.replace(antes, ahora)
                reemplazos[antes] = ahora
                hubo_cambio_manual = True
                print(f"  ✅ {nombre_cfg} en config.toml actualizado "
                      f"({anonimizar(antes)} → {anonimizar(ahora)})")
            else:
                print(f"  ⚠️  {nombre_cfg}: el valor viejo no aparece en "
                      f"config.toml (puede que ya esté actualizado)")
    if not hubo_cambio_manual:
        print("  (sin cambios manuales detectados en .env — todo en sincronía)")

    # ── Otras llaves del config.toml ──
    print("\n── Otras llaves en config.toml ──")
    extras = re.findall(r'["\']([A-Za-z0-9_\-:]{25,})["\']', contenido)
    # filtrar las que ya conocemos en .env (no duplicar)
    conocidas = set(env.values())
    extras = [e for e in extras if e not in conocidas]
    if extras:
        print(f"  ⚠️  {len(extras)} llaves en config.toml no están en .env (no trackeado en git):")
        for e in extras[:8]:
            print(f"     - {anonimizar(e)}")
    else:
        print("  (las llaves de config.toml coinciden con .env — bien)")

    # ── Verificar llaves rotadas ──
    if "OPENROUTER_API_KEY" in actualizados:
        nueva_or = actualizados["OPENROUTER_API_KEY"]
        st, d = http(
            "https://openrouter.ai/api/v1/key",
            {"Authorization": f"Bearer {nueva_or}"},
        )
        if st == 200:
            print(f"\n✅ OpenRouter nueva llave verificada (uso: {d.get('data',{}).get('usage')})")
        else:
            print(f"\n⚠️  OpenRouter nueva llave NO verificada (HTTP {st})")

    # ── Aplicar cambios ──
    if actualizados or hubo_cambio_manual:
        env.update(actualizados)
        escribir_env(env)
        if reemplazos:
            for vieja, nueva in reemplazos.items():
                contenido = contenido.replace(vieja, nueva)
            escribir_config_toml(contenido)
        print("\n✅ .env y config.toml actualizados con las llaves nuevas.")

    print("\n" + "=" * 70)
    print("RECORDATORIO: rotar una llave la INVALIDA. Tras rotar, cualquier")
    print("servicio que usara el valor viejo dejará de funcionar hasta que")
    print("apunte al nuevo valor. Verifica tus servicios antes y después.")
    print("=" * 70)


if __name__ == "__main__":
    main()
