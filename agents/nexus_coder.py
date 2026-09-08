#!/usr/bin/env python3
"""
nexus_coder — Genera comandos GDScript para Godot 4 y los despacha por TCP.

Cambios de robustez:
  * Las llaves del Zenith Pool se leen del entorno (antes estaban hardcodeadas
    en el repositorio, lo que las dejaba expuestas en el histórico de git).
  * La consulta a Gemini pasa por ``nexo_api.call_gemini``: si el modelo
    principal responde 503 UNAVAILABLE o 429, el reintento salta al siguiente
    modelo de la cadena sin intervención manual.
"""

from __future__ import annotations

import json
import os
import socket
import sys
from typing import Any, Dict, Optional

if __package__ in (None, ""):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import nexo_api  # noqa: E402

GODOT_HOST = os.environ.get("NEXUS_GODOT_HOST", "127.0.0.1")
GODOT_PORT = int(os.environ.get("NEXUS_GODOT_PORT", "8900"))

PROMPT_PLANTILLA = (
    "Eres un asistente experto en Godot 4 (GDScript). Genera únicamente un objeto "
    "JSON con el comando a ejecutar en el juego. Estructura esperada: "
    '{{"action": "create_node", "type": "Sprite2D", "name": "Jugador"}}. '
    "Petición: {prompt}"
)


def ask_gemini(
    prompt_text: str,
    *,
    model: Optional[str] = None,
    transport: Any = None,
) -> Optional[Dict[str, Any]]:
    """Consulta a Gemini con fallback automático de modelos."""
    send = transport or nexo_api.call_gemini
    contents = [{"role": "user", "parts": [{"text": PROMPT_PLANTILLA.format(prompt=prompt_text)}]}]

    try:
        result = send(
            contents,
            model=model or os.environ.get("NEXUS_GEMINI_MODEL"),
            generation_config={
                "temperature": 0.2,
                "responseMimeType": "application/json",
            },
            timeout=float(os.environ.get("NEXUS_GEMINI_TIMEOUT", 10)),
        )
    except nexo_api.TransportError as exc:
        print("❌ Zenit Pool agotado: {}".format(exc.describe()))
        return None

    if result.used_fallback:
        print("🔁 Modelo de respaldo usado: {}".format(result.model))

    raw = nexo_api.extract_text(result.raw)
    try:
        return json.loads(raw) if raw.strip() else None
    except ValueError:
        return None


def send_to_godot(
    command_dict: Dict[str, Any], host: str = GODOT_HOST, port: int = GODOT_PORT
) -> bool:
    """Envía el comando al editor/juego por TCP."""
    print(f"🧠 Conectando con Godot en {host}:{port}...")
    try:
        with socket.create_connection((host, port), timeout=5) as sock:
            sock.sendall((json.dumps(command_dict) + "\n").encode("utf-8"))
            response = sock.recv(1024).decode("utf-8", errors="replace")
        print(f"📥 Respuesta de Godot: {response.strip()}")
        return True
    except OSError as exc:
        print(f"❌ Error de conexión con Godot: {exc}")
        return False


def main(argv: Optional[list] = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    prompt = " ".join(argv) or "Crea un nodo de prueba en la escena activa"
    print(f"🚀 Procesando orden: '{prompt}'")

    if not nexo_api.load_pool_keys():
        print(
            "⚠️  Sin llaves: exporta NEXUS_GEMINI_KEYS (o GEMINI_API_KEY). "
            "Ver .env.example. No se guardan llaves en el código."
        )
        return 1

    if ask_gemini(prompt) is None:
        print("⚠️  Sin respuesta util del modelo; no se envía nada a Godot.")
        return 1

    payload = {"action": "log", "message": f"Orden recibida desde Termux: {prompt}"}
    return 0 if send_to_godot(payload) else 1


if __name__ == "__main__":
    raise SystemExit(main())
