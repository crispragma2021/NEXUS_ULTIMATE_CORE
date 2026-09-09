"""Configuración compartida de pytest para los agentes NEXUS.

Añade ``agents/`` al ``sys.path`` para poder importar los módulos tal y como lo
hacen los scripts cuando se ejecutan sueltos en Termux.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent
AGENTS_DIR = REPO_ROOT / "agents"

for candidate in (str(AGENTS_DIR), str(REPO_ROOT)):
    if candidate not in sys.path:
        sys.path.insert(0, candidate)

#: Entorno Termux simulado para las pruebas de portabilidad de rutas.
TERMUX_ENV = {
    "PREFIX": "/data/data/com.termux/files/usr",
    "TMPDIR": "/data/data/com.termux/files/usr/tmp",
    "HOME": "/data/data/com.termux/files/home",
    "TEMP": "",
    "TMP": "",
    "LOCALAPPDATA": "",
    "NEXUS_TMPDIR": "",
}

#: Prefijo Termux falso pero ESCRIBIBLE (bajo tmp_path) para pruebas reales.
def termux_env_en(tmp_path) -> dict:
    """Termux simulado con árbol escribible: $PREFIX/tmp y $HOME existen de verdad."""
    prefix = tmp_path / "usr"
    (prefix / "tmp").mkdir(parents=True)
    home = tmp_path / "home"
    home.mkdir()
    return {
        "PREFIX": str(prefix),
        "TMPDIR": str(prefix / "tmp"),
        "HOME": str(home),
        "TEMP": "",
        "TMP": "",
        "LOCALAPPDATA": "",
        "NEXUS_TMPDIR": "",
    }

#: Entorno de PC de escritorio (Linux) simulado.
PC_ENV = {
    "PREFIX": "",
    "TMPDIR": "",
    "HOME": "/home/crisp",
    "TEMP": "",
    "TMP": "",
    "LOCALAPPDATA": "",
    "NEXUS_TMPDIR": "",
}


@pytest.fixture()
def android(monkeypatch):
    """Simula Android de verdad: la detección no puede depender solo del env."""
    monkeypatch.setattr(sys, "platform", "android", raising=False)
    return monkeypatch


@pytest.fixture()
def pc_linux(monkeypatch):
    """Simula un PC de escritorio Linux."""
    monkeypatch.setattr(sys, "platform", "linux", raising=False)
    return monkeypatch


def fake_gemini_response(text: str = "ok") -> dict:
    """Cuerpo de respuesta ``generateContent`` mínimo válido."""
    return {
        "candidates": [
            {"content": {"role": "model", "parts": [{"text": text}]}}
        ]
    }


def gemini_error(status: str, message: str = "") -> dict:
    """Cuerpo de error de la API de Gemini."""
    return {"error": {"code": 0, "message": message, "status": status}}


def make_transport_script(*results):
    """Devuelve un ``_http_post`` falso que responde `results` en orden.

    Cada elemento es ``(status, cuerpo)`` o una excepción a lanzar.
    """
    calls = []
    scripted = list(results)

    def _http_post(url, payload, headers, timeout):
        calls.append({"url": url, "payload": payload, "headers": headers})
        if not scripted:
            raise AssertionError("Se agotaron las respuestas simuladas")
        item = scripted.pop(0)
        if isinstance(item, Exception):
            raise item
        return item

    _http_post.calls = calls
    return _http_post


def model_of(url: str) -> str:
    """Extrae el nombre de modelo de una URL ``generateContent``."""
    return url.split("/models/", 1)[1].split(":", 1)[0]


def key_of(url: str) -> str:
    """Extrae la API key de una URL ``generateContent``."""
    return url.split("key=", 1)[1]
