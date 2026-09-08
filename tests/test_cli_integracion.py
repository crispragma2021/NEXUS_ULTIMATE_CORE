"""Integración del CLI `bin/nexus` contra un daemon simulado por socket Unix.

Verifica el protocolo completo (petición JSON -> respuesta JSON) sin necesitar
ni Android ni el binario Rust.
"""

from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import threading
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent
NEXUS_CLI = REPO / "bin" / "nexus"
NEXUSNET = REPO / "bin" / "nexusnet"

pytestmark = pytest.mark.skipif(
    os.name != "posix", reason="el socket Unix solo existe en POSIX"
)


class DaemonFalso:
    """Servidor Unix mínimo que imita a nexus_host_daemon."""

    def __init__(self, ruta):
        self.ruta = str(ruta)
        self.peticiones = []
        self._servidor = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self._servidor.bind(self.ruta)
        self._servidor.listen(5)
        self._vivo = True
        self._hilo = threading.Thread(target=self._atender, daemon=True)

    def _atender(self):
        self._servidor.settimeout(0.2)
        while self._vivo:
            try:
                conexion, _ = self._servidor.accept()
            except socket.timeout:
                continue
            except OSError:
                break
            with conexion:
                datos = b""
                while b"\n" not in datos:
                    bloque = conexion.recv(4096)
                    if not bloque:
                        break
                    datos += bloque
                if not datos.strip():
                    continue
                try:
                    peticion = json.loads(datos.decode("utf-8"))
                except ValueError:
                    respuesta = {"ok": False, "result": "JSON invalido"}
                else:
                    self.peticiones.append(peticion)
                    respuesta = self._responder(peticion)
                conexion.sendall((json.dumps(respuesta) + "\n").encode("utf-8"))

    @staticmethod
    def _responder(peticion):
        accion = peticion.get("action")
        if accion == "GetScreenSize":
            return {"ok": True, "result": "Physical size: 1080x2400"}
        if accion == "Screenshot":
            return {"ok": True, "result": "/ruta/portable/nexus_shot.png"}
        if accion == "Tap":
            return {"ok": True, "result": "OK"}
        if accion == "Exec":
            return {"ok": True, "result": peticion["payload"]["command"]}
        return {"ok": False, "result": "accion desconocida: {}".format(accion)}

    def __enter__(self):
        self._hilo.start()
        return self

    def __exit__(self, *exc):
        self._vivo = False
        self._hilo.join(timeout=2)
        self._servidor.close()
        try:
            os.unlink(self.ruta)
        except OSError:
            pass
        return False


def correr_cli(args, socket_path, env_extra=None):
    env = dict(os.environ)
    env["NEXUS_SOCKET_PATH"] = socket_path
    env["NEXUS_CORE_DIR"] = str(REPO)
    env["PYTHONPATH"] = str(REPO / "agents")
    env.pop("PREFIX", None)
    env.update(env_extra or {})
    return subprocess.run(
        ["bash", str(NEXUS_CLI), *args],
        capture_output=True,
        text=True,
        env=env,
        timeout=60,
    )


@pytest.fixture()
def daemon(tmp_path):
    with DaemonFalso(tmp_path / "nexus.sock") as servidor:
        yield servidor


# --------------------------------------------------------------------------
def test_cli_screen(daemon, tmp_path):
    proc = correr_cli(["screen"], daemon.ruta)
    assert proc.returncode == 0, proc.stderr
    assert "1080x2400" in proc.stdout
    assert daemon.peticiones[-1] == {"action": "GetScreenSize"}


def test_cli_shot_devuelve_la_ruta_portable(daemon, tmp_path):
    proc = correr_cli(["shot"], daemon.ruta)
    assert proc.returncode == 0, proc.stderr
    assert "/ruta/portable/nexus_shot.png" in proc.stdout
    assert "/sdcard" not in proc.stdout


def test_cli_tap_envia_coordenadas(daemon, tmp_path):
    proc = correr_cli(["tap", "100", "250"], daemon.ruta)
    assert proc.returncode == 0, proc.stderr
    assert daemon.peticiones[-1] == {
        "action": "Tap",
        "payload": {"x": 100, "y": 250},
    }


def test_cli_exec_con_comillas_no_rompe_el_protocolo(daemon, tmp_path):
    """Un apóstrofe en el comando ya no rompe el JSON (antes se interpolaba)."""
    comando = "echo 'hola mundo' && cat fichero"
    proc = correr_cli(["exec", comando], daemon.ruta)
    assert proc.returncode == 0, proc.stderr
    assert daemon.peticiones[-1] == {
        "action": "Exec",
        "payload": {"command": comando},
    }


def test_cli_sin_argumentos_muestra_uso(daemon, tmp_path):
    proc = correr_cli([], daemon.ruta)
    assert proc.returncode == 0
    assert "Uso: nexus" in proc.stdout


def test_cli_tap_sin_argumentos_falla(daemon, tmp_path):
    proc = correr_cli(["tap"], daemon.ruta)
    assert proc.returncode == 1
    assert "Uso: nexus tap" in proc.stdout


def test_cli_exec_sin_comando_falla(daemon, tmp_path):
    proc = correr_cli(["exec"], daemon.ruta)
    assert proc.returncode == 1
    assert "Uso: nexus exec" in proc.stdout


def test_cli_daemon_status(daemon, tmp_path):
    proc = correr_cli(["daemon", "status"], daemon.ruta)
    assert proc.returncode == 0
    assert "Daemon inactivo" in proc.stdout or "Daemon activo" in proc.stdout


def test_cli_doctor_reporta_rutas(daemon, tmp_path):
    env = {"DEEPSEEK_API_KEY": "sk-prueba"}
    proc = correr_cli(["doctor"], daemon.ruta, env)
    assert proc.returncode == 0, proc.stderr
    assert "temp_root" in proc.stdout
    assert "gemini-3.8-flash" in proc.stdout


def test_cli_doctor_sin_llaves_avisa(daemon, tmp_path):
    env = {k: "" for k in ("DEEPSEEK_API_KEY", "NEXUS_GEMINI_KEYS", "GEMINI_API_KEY")}
    proc = correr_cli(["doctor"], daemon.ruta, env)
    assert proc.returncode == 1
    assert "Sin llaves" in proc.stdout


def test_cli_daemon_start_con_socket_vivo_no_reinicia(daemon, tmp_path):
    """Si el socket responde, el CLI no intenta arrancar nada."""
    proc = correr_cli(["daemon", "start"], daemon.ruta, {"NEXUS_DAEMON_BIN": "/no/existe"})
    assert proc.returncode == 0, proc.stderr
    assert "corriendo" in proc.stdout


def test_cli_daemon_start_sin_binario_da_instrucciones(tmp_path):
    """Sin socket vivo y sin binario, se explican los pasos de compilación."""
    socket_muerto = str(tmp_path / "no_existe.sock")
    proc = correr_cli(
        ["daemon", "start"], socket_muerto, {"NEXUS_DAEMON_BIN": "/no/existe"}
    )
    assert proc.returncode == 1
    assert "cargo build" in proc.stdout


# --------------------------------------------------------------------------
# nexusnet
# --------------------------------------------------------------------------
def test_nexusnet_sin_prompt_muestra_uso():
    proc = subprocess.run(
        ["bash", str(NEXUSNET)], capture_output=True, text=True, timeout=30
    )
    assert proc.returncode == 1
    assert "Uso: nexusnet" in proc.stdout


def test_nexusnet_sin_llave_no_llama_a_la_api():
    env = dict(os.environ)
    env["DEEPSEEK_API_KEY"] = ""
    env["NEXUS_CHAT_API_KEY"] = ""
    proc = subprocess.run(
        ["bash", str(NEXUSNET), "hola"],
        capture_output=True,
        text=True,
        env=env,
        timeout=30,
    )
    assert proc.returncode == 1
    assert "Sin llave" in proc.stdout


def test_binarios_sin_llaves_hardcodeadas():
    """Regresión: las llaves reales que había en git no pueden volver."""
    import re

    patron = re.compile(r"AIza[0-9A-Za-z_\-]{20,}|sk-[0-9a-f]{24,}")
    for archivo in (NEXUS_CLI, NEXUSNET):
        assert not patron.search(archivo.read_text(encoding="utf-8")), archivo.name
