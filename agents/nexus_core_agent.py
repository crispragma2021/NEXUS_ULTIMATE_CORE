#!/usr/bin/env python3
"""
nexus_core_agent — Orquestador principal de NEXUS en Termux/PC.

Cambios de robustez respecto a la versión original:
  * Transporte con fallback de modelos y rotación de llaves (``nexo_api``).
  * Memoria de corto plazo recortada ANTES de persistir y con escritura atómica
    (``nexo_cache``): el fichero ya no crece sin límite ni se re-serializa con
    ``indent=2`` en cada turno.
  * ``execute_cmd`` con salida truncada (cabeza + cola) y timeout: un comando
    ruidoso ya no puede inflar el proceso hasta que Android lo mate por OOM.
  * ``[Cache Audit]`` solo cuando aporta: se silencia bajo perfil de memoria
    baja y cuando no hay caché de prompt que auditar.
  * Rutas portables: nada de ``/tmp`` ni de rutas fijas de Windows/PC.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

# Permite ejecutarlo como script suelto (``python agents/nexus_core_agent.py``).
if __package__ in (None, ""):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import nexo_api  # noqa: E402
import nexo_cache  # noqa: E402
import nexo_plataforma as plataforma  # noqa: E402
import nexo_shell  # noqa: E402

# --------------------------------------------------------------------------
# Configuración
# --------------------------------------------------------------------------
DEFAULT_CHAT_URL = "https://api.deepseek.com/v1/chat/completions"
DEFAULT_CHAT_MODEL = "deepseek-chat"

# Prefijo inmutable optimizado para Prompt Caching. No tocar sin medir: cualquier
# cambio aquí invalida el caché de prompt del proveedor.
SYSTEM_PROMPT = """Eres el orquestador principal de Nexus_Engine_2D en Termux (ARM64).
Directivas obligatorias:
- Proyecto único: Nexus_Engine_2D (TanStack Start SSR, Vercel).
- Prohibido interactuar con GDevelop o compilar paquetes pesados en local.
- Usa la herramienta execute_cmd para inspeccionar archivos, git, puertos o procesos.
- Sé conciso y responde siempre con pasos claros."""

TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "execute_cmd",
            "description": (
                "Ejecuta comandos seguros de shell en Termux (ls, ps, git, curl, cat)"
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Comando a ejecutar"}
                },
                "required": ["command"],
            },
        },
    }
]


def _load_env_file(path: Optional[str] = None) -> None:
    """Carga ``.env`` en ``os.environ`` sin dependencias externas."""
    candidates = []
    if path:
        candidates.append(Path(path))
    root = Path(__file__).resolve().parent.parent
    candidates.append(root / ".env")
    home = os.environ.get("HOME")
    if home:
        candidates.append(Path(home) / ".env")

    for candidate in candidates:
        if not candidate.is_file():
            continue
        try:
            for line in candidate.read_text(encoding="utf-8", errors="ignore").splitlines():
                line = line.strip()
                if not line or line.startswith("#") or "=" not in line:
                    continue
                key, _, value = line.partition("=")
                key, value = key.strip(), value.strip().strip('"').strip("'")
                if key and key not in os.environ:
                    os.environ[key] = value
        except OSError:
            continue
        break


def get_api_key(env: Optional[dict] = None) -> str:
    env = os.environ if env is None else env
    return str(
        env.get("DEEPSEEK_API_KEY")
        or env.get("NEXUS_CHAT_API_KEY")
        or env.get("DEEPSEEK_API_TOKEN")
        or ""
    ).strip()


def get_chat_url(env: Optional[dict] = None) -> str:
    env = os.environ if env is None else env
    return str(env.get("NEXUS_CHAT_URL") or DEFAULT_CHAT_URL)


def get_chat_model(env: Optional[dict] = None) -> str:
    env = os.environ if env is None else env
    return str(env.get("NEXUS_CHAT_MODEL") or DEFAULT_CHAT_MODEL)


def memory_path(env: Optional[dict] = None) -> str:
    """Ruta del historial. En Android usa la raíz temporal portable; en PC se
    conserva el ``~/nexus_chat_memory.json`` original."""
    env = os.environ if env is None else env
    override = env.get("NEXUS_MEMORY_FILE")
    if override:
        return override
    home = env.get("HOME") or os.path.expanduser("~")
    if plataforma.is_android(env):
        return plataforma.temp_path("nexus_chat_memory.json", env=env)
    return os.path.join(home, "nexus_chat_memory.json")


def cache_audit(usage: Optional[Dict[str, Any]]) -> Optional[str]:
    """Traza de Prompt Caching. Devuelve ``None`` si no hay nada que auditar.

    Imprimir esta línea en cada turno cuesta I/O de consola y despierta el TTY
    de Termux sin aportar nada cuando el proveedor no informa de caché.
    """
    if not isinstance(usage, dict) or not usage:
        return None
    hit = int(usage.get("prompt_cache_hit_tokens") or 0)
    miss = int(usage.get("prompt_cache_miss_tokens") or 0)
    if hit == 0 and miss == 0:
        return None
    return "[Cache Audit] Hit: {} | Miss: {}".format(hit, miss)


def execute_tool(name: str, arguments: Dict[str, Any], profile: Dict[str, Any]) -> str:
    """Despacha una llamada de herramienta del modelo."""
    if name != "execute_cmd":
        return "Herramienta no soportada: {}".format(name)
    return nexo_shell.run_shell(
        str(arguments.get("command", "")),
        timeout=int(os.environ.get("NEXUS_CMD_TIMEOUT", nexo_shell.DEFAULT_TIMEOUT)),
        max_output_bytes=int(profile["max_output_bytes"]),
    )


def query_agent(
    prompt: str,
    *,
    env: Optional[dict] = None,
    chat_fn=None,
    verbose: Optional[bool] = None,
) -> str:
    """Ciclo completo: contexto -> modelo -> herramientas -> respuesta."""
    os_env = os.environ if env is None else env
    profile = plataforma.memory_profile(os_env)
    history = nexo_cache.TurnCache(
        memory_path(os_env),
        max_turns=int(profile["max_history_turns"]),
    )

    messages: List[Dict[str, Any]] = [{"role": "system", "content": SYSTEM_PROMPT}]
    messages.extend(history.messages)
    messages.append({"role": "user", "content": prompt})

    api_key = get_api_key(os_env)
    payload = {
        "model": get_chat_model(os_env),
        "messages": messages,
        "tools": TOOLS,
        "tool_choice": "auto",
        "temperature": 0.2,
    }
    send = chat_fn or (
        lambda p: nexo_api.chat(
            p,
            url=get_chat_url(os_env),
            api_key=api_key,
            timeout=float(os_env.get("NEXUS_CHAT_TIMEOUT", 30)),
        )
    )

    try:
        res = send(payload)
    except nexo_api.TransportError as exc:
        return "Error de transporte: {}".format(exc.describe())

    choices = (res or {}).get("choices") or []
    if not choices or not choices[0].get("message"):
        return "Error en API: {}".format(res)

    msg = choices[0]["message"]
    audit = cache_audit(res.get("usage"))
    show_trace = profile["cache_audit"] if verbose is None else verbose
    if audit and show_trace:
        print(audit)

    tool_calls = msg.get("tool_calls") or []
    if tool_calls:
        # Un solo mensaje assistant con todos los tool_calls: evita duplicar el
        # contexto (y volver a pagar tokens) por cada herramienta invocada.
        messages.append(msg)
        for call in tool_calls:
            function = call.get("function") or {}
            name = function.get("name", "")
            try:
                arguments = json.loads(function.get("arguments") or "{}")
            except ValueError:
                arguments = {}
            if show_trace:
                print("[Tool: {}] -> {}".format(name, arguments.get("command")))
            messages.append(
                {
                    "role": "tool",
                    "tool_call_id": call.get("id"),
                    "content": execute_tool(name, arguments, profile),
                }
            )

        payload["messages"] = messages
        try:
            second = send(payload)
        except nexo_api.TransportError as exc:
            return "Error de transporte: {}".format(exc.describe())
        second_choices = (second or {}).get("choices") or []
        if not second_choices or not second_choices[0].get("message"):
            return "Error en API: {}".format(second)
        final_reply = second_choices[0]["message"].get("content") or ""
        audit = cache_audit(second.get("usage"))
        if audit and show_trace:
            print(audit)
    else:
        final_reply = msg.get("content") or ""

    history.extend(
        [
            {"role": "user", "content": prompt},
            {"role": "assistant", "content": final_reply},
        ]
    )
    history.save()
    return final_reply


def main(argv: Optional[List[str]] = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    _load_env_file()

    if argv and argv[0] in ("--reset", "reset"):
        path = memory_path()
        try:
            os.unlink(path)
            print("[OK] Historial reiniciado:", path)
        except OSError:
            print("[OK] No había historial que reiniciar.")
        return 0

    prompt = " ".join(argv) or "Comprueba el estado del repositorio Nexus_Engine_2D"
    profile = plataforma.memory_profile()
    print(
        "[NEXUS] plataforma={} mem={}({} MB libres)".format(
            plataforma.detect_platform(),
            profile["level"],
            (profile["available_bytes"] or 0) // (1024 * 1024),
        )
    )
    print("\n" + query_agent(prompt))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
