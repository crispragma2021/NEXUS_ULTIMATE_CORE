#!/usr/bin/env python3
"""
nexus_vision_agent — Analiza la pantalla del dispositivo con Gemini.

Flujo completo sin lecturas redundantes:
    daemon (screencap)  ->  ruta portable  ->  base64 en streaming  ->  inline_data

La captura se lee UNA sola vez y el base64 se escribe directamente en el
payload JSON: no hay archivo temporal intermedio ni ``str`` gigante que luego
haya que re-codificar.

Uso:
    nexus vision "¿qué hay en pantalla?"
    python3 agents/nexus_vision_agent.py --imagen /ruta/captura.png "descríbela"
"""

from __future__ import annotations

import argparse
import os
import sys
from typing import List, Optional

if __package__ in (None, ""):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import nexo_api  # noqa: E402
import nexo_plataforma as plataforma  # noqa: E402
import nexo_vision as vision  # noqa: E402


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="nexus vision",
        description="Analiza una captura de pantalla con Gemini (inline_data).",
    )
    parser.add_argument("prompt", nargs="*", help="Qué quieres saber de la imagen")
    parser.add_argument(
        "--imagen",
        action="append",
        default=[],
        help="Ruta de imagen ya existente (repite para varias). Sin ella se captura la pantalla.",
    )
    parser.add_argument(
        "--socket",
        default=None,
        help="Ruta del socket del nexus_host_daemon (por defecto $NEXUS_SOCKET_PATH)",
    )
    parser.add_argument(
        "--modelo",
        default=None,
        help="Modelo inicial de la cadena (por defecto $NEXUS_GEMINI_MODEL)",
    )
    return parser


def analizar(
    prompt: str,
    image_paths: Optional[List[str]] = None,
    *,
    socket_path: Optional[str] = None,
    model: Optional[str] = None,
) -> str:
    """Captura (si hace falta), empaqueta inline_data y consulta al modelo."""
    profile = plataforma.memory_profile()
    paths = list(image_paths or [])
    if not paths:
        paths = [vision.capture_screenshot(socket_path)]

    # Cadena de modelos: primero el pedido, luego el fallback configurado.
    chain = nexo_api.model_chain(model or os.environ.get("NEXUS_GEMINI_MODEL"))
    vision_model = next((m for m in chain if nexo_api.model_supports_images(m)), None)

    if vision_model is None:
        # Ningún modelo de la cadena admite imágenes: degradar a texto en vez de
        # gastar cuota en una llamada que la API va a rechazar.
        return vision.describe_image_for_text_models(paths)

    payload = vision.build_payload_bytes(
        prompt, paths, max_bytes=int(profile["screenshot_max_bytes"])
    )
    try:
        result = nexo_api.call_gemini(
            [],  # el cuerpo ya viene serializado en `payload`
            model=vision_model,
            payload=payload,
        )
    except vision.VisionError as exc:
        return "Error del pipeline multimodal: {}".format(exc)
    except nexo_api.TransportError as exc:
        return "Error de transporte: {}".format(exc.describe())

    texto = nexo_api.extract_text(result.raw)
    if result.used_fallback:
        texto = "🔁 Modelo de respaldo: {}\n{}".format(result.model, texto)
    return texto or "(el modelo no devolvió texto)"


def main(argv: Optional[List[str]] = None) -> int:
    args = _build_parser().parse_args(argv)

    if not nexo_api.load_pool_keys():
        print(
            "⚠️  Sin llaves: exporta NEXUS_GEMINI_KEYS (o GEMINI_API_KEY). "
            "Ver .env.example."
        )
        return 1

    prompt = " ".join(args.prompt) or "Describe qué ves en la pantalla."
    print(analizar(prompt, args.imagen, socket_path=args.socket, model=args.modelo))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
