"""Tarea 2 — Compatibilidad de rutas Android/Termux sin romper el PC.

Regla de aceptación: en Termux todo resuelve bajo $TMPDIR/$PREFIX/tmp y NUNCA
aparece "/tmp" duro; en PC el comportamiento original (/tmp) se conserva.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

import nexo_plataforma as plataforma
from conftest import PC_ENV, TERMUX_ENV, termux_env_en


# --------------------------------------------------------------------------
# Detección
# --------------------------------------------------------------------------
def test_termux_se_detecta_por_prefix():
    assert plataforma.detect_platform(TERMUX_ENV) == "android"
    assert plataforma.is_android(TERMUX_ENV) is True
    assert plataforma.is_termux(TERMUX_ENV) is True


def test_pc_linux_sin_prefix_no_es_android():
    assert plataforma.detect_platform(PC_ENV) == "linux"
    assert plataforma.is_android(PC_ENV) is False
    assert plataforma.is_termux(PC_ENV) is False


def test_windows_se_detecta(monkeypatch):
    monkeypatch.setattr(sys, "platform", "win32")
    env = dict(PC_ENV, TEMP="C:\\Users\\crisp\\AppData\\Local\\Temp")
    assert plataforma.detect_platform(env) == "windows"
    assert plataforma.is_windows(env) is True


# --------------------------------------------------------------------------
# Resolución de la raíz temporal
# --------------------------------------------------------------------------
def test_termux_usa_tmpdir(android, tmp_path):
    env = termux_env_en(tmp_path)
    assert plataforma.temp_root(env) == env["TMPDIR"]


def test_termux_cae_a_prefix_tmp_si_no_hay_tmpdir(android, tmp_path):
    env = termux_env_en(tmp_path)
    env["TMPDIR"] = ""
    assert plataforma.temp_root(env) == str(Path(env["PREFIX"]) / "tmp")


def test_termux_cae_a_home_tmp_sin_tmpdir_ni_prefix(android, tmp_path):
    env = termux_env_en(tmp_path)
    env["TMPDIR"] = ""
    env["PREFIX"] = ""
    esperado = str(Path(env["HOME"]) / "tmp")
    os.makedirs(esperado, exist_ok=True)
    assert plataforma.temp_root(env) == esperado


def test_termux_nunca_devuelve_tmp_duro(android, tmp_path):
    """Ninguna resolución puede salirse del árbol de Termux simulado.

    Ojo: el ``tmp_path`` de pytest vive bajo /tmp en Linux, así que la
    comprobación se hace contra el árbol simulado, no contra el literal.
    """
    env = termux_env_en(tmp_path)
    arbol_valido = (env["PREFIX"], env["HOME"])
    for variante in (
        env,
        dict(env, TMPDIR=""),
        dict(env, TMPDIR="", PREFIX=""),
        dict(env, TMPDIR="/no/existe/nexus", PREFIX="/tampoco"),
    ):
        resuelto = plataforma.temp_root(variante)
        assert resuelto.startswith(arbol_valido), (variante, resuelto)


def test_pc_linux_conserva_tmp_original(monkeypatch):
    """El PC se queda exactamente como estaba: /tmp."""
    monkeypatch.setattr(sys, "platform", "linux")
    resuelto = plataforma.temp_root(dict(PC_ENV))
    assert resuelto in ("/tmp", os.path.realpath("/tmp"))


def test_pc_usa_tmpdir_si_esta_definido(tmp_path, monkeypatch):
    monkeypatch.setattr(sys, "platform", "linux")
    custom = tmp_path / "mitmp"
    custom.mkdir()
    env = dict(PC_ENV, TMPDIR=str(custom))
    assert plataforma.temp_root(env) == str(custom)


def test_windows_usa_temp(tmp_path, monkeypatch):
    monkeypatch.setattr(sys, "platform", "win32")
    temp = tmp_path / "Temp"
    temp.mkdir()
    env = dict(PC_ENV, TEMP=str(temp), LOCALAPPDATA="")
    assert plataforma.temp_root(env) == str(temp)


def test_override_explicito_gana_siempre(tmp_path):
    custom = tmp_path / "override"
    custom.mkdir()
    env = dict(TERMUX_ENV, NEXUS_TMPDIR=str(custom))
    assert plataforma.temp_root(env) == str(custom)


def test_directorio_inusable_se_descarta(android, tmp_path):
    """Una ruta imposible no puede colar: debe caer en un candidato válido."""
    env = termux_env_en(tmp_path)
    env["TMPDIR"] = "/proc/no/escribible/nexus"
    assert plataforma.temp_root(env) == str(Path(env["PREFIX"]) / "tmp")


def test_temp_root_nunca_lanza(android, tmp_path):
    """Un agente no puede morir por no encontrar un temporal."""
    env = {
        "PREFIX": "/ruta/inexistente",
        "TMPDIR": "/ruta/inexistente/tmp",
        "HOME": "/ruta/inexistente/home",
    }
    resuelto = plataforma.temp_root(env)
    assert resuelto and os.path.isdir(resuelto)


def test_temp_path_concatena_bajo_la_raiz(tmp_path):
    tmpdir = tmp_path / "t"
    tmpdir.mkdir()
    env = dict(TERMUX_ENV, TMPDIR=str(tmpdir))
    assert plataforma.temp_path("nexus_chat_memory.json", env=env) == str(
        tmpdir / "nexus_chat_memory.json"
    )


def test_home_tmp_es_candidato_valido_en_termux(android, tmp_path):
    """Sin $TMPDIR ni $PREFIX, se crea y usa $HOME/tmp."""
    env = termux_env_en(tmp_path)
    env["TMPDIR"] = ""
    env["PREFIX"] = ""
    esperado = str(Path(env["HOME"]) / "tmp")
    assert not Path(esperado).exists()
    assert plataforma.temp_root(env) == esperado
    assert Path(esperado).is_dir()  # se creó al vuelo


def test_ultimo_recurso_es_el_cwd(android, tmp_path):
    """Con todo inservible se usa el cwd: temp_root no lanza nunca."""
    readonly = tmp_path / "ro"
    readonly.mkdir()
    os.chmod(readonly, 0o500)
    env = {"TMPDIR": "", "PREFIX": "", "HOME": str(readonly)}
    try:
        resuelto = plataforma.temp_root(env)
        assert os.path.isdir(resuelto)
    finally:
        os.chmod(readonly, 0o700)


# --------------------------------------------------------------------------
# Sin rutas duras en el código
# --------------------------------------------------------------------------
def _literales_de_ruta(archivo: Path):
    """Literales de cadena que empiezan por /tmp, ignorando comentarios."""
    import io
    import tokenize

    sospechosos = []
    with archivo.open("rb") as fh:
        try:
            tokens = tokenize.tokenize(fh.readline)
            for tipo, texto, inicio, _fin, _linea in tokens:
                if tipo != tokenize.STRING:
                    continue
                try:
                    valor = eval(texto, {"__builtins__": {}}, {})  # noqa: S307
                except Exception:
                    continue
                if isinstance(valor, str) and (
                    valor == "/tmp" or valor.startswith("/tmp/")
                ):
                    sospechosos.append((inicio[0], texto))
        except (tokenize.TokenError, IndentationError):
            pass
    return sospechosos


def test_agentes_no_contienen_rutas_tmp_duras():
    root = Path(__file__).resolve().parent.parent / "agents"
    archivos = sorted(root.glob("*.py"))
    assert archivos, "no se encontró ningún agente que auditar"
    for archivo in archivos:
        sospechosos = _literales_de_ruta(archivo)
        assert not sospechosos, "Ruta /tmp dura en {} -> {}".format(
            archivo.name, sospechosos
        )


def test_daemon_rust_no_contiene_tmp_duro():
    """El daemon no hardcodea /tmp ni /sdcard fuera de sus propios tests."""
    fuente = (
        Path(__file__).resolve().parent.parent / "daemon" / "src" / "main.rs"
    ).read_text(encoding="utf-8")
    produccion = fuente.split("#[cfg(test)]", 1)[0]
    assert '"/tmp' not in produccion
    assert "/sdcard/" not in produccion


# --------------------------------------------------------------------------
# Perfil de memoria
# --------------------------------------------------------------------------
def test_perfil_normal_con_ram_suficiente():
    perfil = plataforma.memory_profile({}, available=4 * 1024**3)
    assert perfil["level"] == "normal"
    assert perfil["cache_audit"] is True


def test_perfil_low_bajo_1gb_libre():
    """El caso pedido: aarch64 con menos de 1 GB de RAM libre."""
    perfil = plataforma.memory_profile({}, available=900 * 1024**2)
    assert perfil["level"] == "low"
    assert perfil["cache_audit"] is False
    assert perfil["max_output_bytes"] < 65536


def test_perfil_critical_bajo_448mb():
    perfil = plataforma.memory_profile({}, available=256 * 1024**2)
    assert perfil["level"] == "critical"
    assert perfil["max_history_turns"] == 2
    assert perfil["screenshot_max_bytes"] <= 3 * 1024**2


def test_perfil_forzable_por_variable_de_entorno():
    perfil = plataforma.memory_profile(
        {"NEXUS_MEM_PROFILE": "critical"}, available=8 * 1024**3
    )
    assert perfil["level"] == "critical"


def test_available_memory_devuelve_int_o_none():
    valor = plataforma.available_memory_bytes()
    assert valor is None or isinstance(valor, int)
