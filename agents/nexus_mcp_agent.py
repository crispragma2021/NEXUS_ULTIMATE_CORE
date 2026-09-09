#!/usr/bin/env python3
"""
nexus_mcp_agent — Motor MCP autónomo: escribe scripts GDScript en el proyecto Godot.

Cambios de robustez:
  * Llaves desde el entorno (antes, placeholders hardcodeados).
  * Transporte con fallback de modelos (``nexo_api``).
  * Ruta del proyecto portable: en Android/Termux resuelve bajo
    ``$HOME/storage/shared`` y en PC bajo ``~/GodotProjects``, ambas
    sobreescribibles con ``NEXUS_GODOT_PROJECT_DIR``.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional

if __package__ in (None, ""):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import nexo_api  # noqa: E402
import nexo_plataforma as plataforma  # noqa: E402

INSTRUCCION = """
Eres el motor MCP autónomo de NEXUS para Godot 4.
Archivos actuales en la raíz del proyecto: {files}.
Petición del usuario: {prompt}

Responde EXCLUSIVAMENTE con un JSON válido con la siguiente estructura:
{{
    "actions": [
        {{
            "type": "create_script",
            "filename": "nombre_archivo.gd",
            "content": "codigo GDScript completo para Godot 4"
        }}
    ]
}}
"""


def project_dir(env: Optional[dict] = None) -> Path:
    env = os.environ if env is None else env
    override = env.get("NEXUS_GODOT_PROJECT_DIR")
    if override:
        return Path(override).expanduser()
    home = Path(env.get("HOME") or os.path.expanduser("~")).expanduser()
    if plataforma.is_android(env):
        # En Termux, el almacenamiento compartido se monta bajo $HOME/storage.
        return home / "storage" / "shared" / "GodotProjects"
    return home / "GodotProjects"


def execute_mcp_actions(actions: Iterable[Dict[str, Any]], root: Path) -> List[str]:
    """Escribe los archivos pedidos, sin salir jamás de la raíz del proyecto."""
    root.mkdir(parents=True, exist_ok=True)
    written: List[str] = []
    for item in actions:
        filename = str(item.get("filename") or "").strip()
        if not filename:
            continue
        target = (root / filename).resolve()
        # Defensa contra path traversal ("../../etc/passwd").
        try:
            target.relative_to(root.resolve())
        except ValueError:
            print(f"⚠️  Ruta rechazada por salir del proyecto: {filename}")
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(str(item.get("content", "")), encoding="utf-8")
        written.append(filename)
        print(f"✨ Archivo creado/actualizado en proyecto: {filename}")
    return written


def ask_zenit_mcp(user_prompt: str, *, transport: Any = None) -> Optional[Dict[str, Any]]:
    send = transport or nexo_api.call_gemini
    root = project_dir()
    existing = sorted(p.name for p in root.iterdir()) if root.is_dir() else []

    contents = [
        {
            "role": "user",
            "parts": [
                {"text": INSTRUCCION.format(files=existing, prompt=user_prompt)}
            ],
        }
    ]

    try:
        print("🧠 Consultando a Zenit Pool (Gemini MCP Engine)...")
        result = send(
            contents,
            model=os.environ.get("NEXUS_GEMINI_MODEL"),
            generation_config={
                "temperature": 0.3,
                "responseMimeType": "application/json",
            },
            timeout=float(os.environ.get("NEXUS_GEMINI_TIMEOUT", 15)),
        )
    except nexo_api.TransportError as exc:
        print(f"❌ Error durante la ejecución MCP: {exc.describe()}")
        return None

    if result.used_fallback:
        print("🔁 Modelo de respaldo usado: {}".format(result.model))

    raw = nexo_api.extract_text(result.raw)
    try:
        return json.loads(raw)
    except ValueError as exc:
        print(f"❌ Respuesta no es JSON valido: {exc}")
        return None


def main(argv: Optional[list] = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv:
        print("Uso: python3 agents/nexus_mcp_agent.py 'Tu orden'")
        return 1
    if not nexo_api.load_pool_keys():
        print(
            "⚠️  Sin llaves: exporta NEXUS_GEMINI_KEYS (o GEMINI_API_KEY). "
            "Ver .env.example."
        )
        return 1

    response = ask_zenit_mcp(" ".join(argv))
    if not response or "actions" not in response:
        print("⚠️  No se generaron acciones válidas.")
        return 1

    written = execute_mcp_actions(response["actions"], project_dir())
    if not written:
        print("⚠️  Ningún archivo aplicable.")
        return 1
    print("🚀 Cambios aplicados con éxito en la carpeta del juego.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
