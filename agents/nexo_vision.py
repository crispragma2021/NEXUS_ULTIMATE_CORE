"""
nexo_vision — Pipeline multimodal ``inline_data`` de una sola pasada.

Problema que resuelve: el flujo clásico de captura en Android hace
``screencap -> /sdcard/Download/x.png -> open() -> base64 -> f-string -> .encode()``,
es decir 2 lecturas de disco y 3-4 copias del buffer. En un aarch64 con menos de
1 GB libre, una captura de 4 MB se convierte en ~20 MB transitorios.

Aquí:
  * ``encode_file_b64``   : base64 en UNA lectura, por trozos alineados a 3 bytes
                            (cada trozo se codifica de forma independiente, sin
                            recomponer padding).
  * ``build_payload_bytes``: escribe el base64 DIRECTAMENTE en el ``bytearray``
                            del payload. Cero archivos temporales intermedios y
                            cero ``str`` gigante que luego haya que ``.encode()``.
  * ``capture_screenshot``: consume la captura del daemon en su ruta portable,
                            sin volver a copiarla a otro temporal.

Todo el módulo usa la librería estándar: funciona en Termux sin pip.
"""

from __future__ import annotations

import base64
import json
import os
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Union

__all__ = [
    "VisionError",
    "CHUNK_SIZE",
    "b64_stream",
    "encode_file_b64",
    "guess_mime",
    "pack_inline_data",
    "vision_contents",
    "build_payload_bytes",
    "capture_screenshot",
    "describe_image_for_text_models",
]

#: Múltiplo de 3 => cada trozo produce base64 independiente, sin padding parcial.
CHUNK_SIZE = 48 * 1024

#: Cadela de control imposible de producir con base64 estándar (A-Z a-z 0-9 + / =).
_B64_SENTINEL = "\u0000NXB64\u0000"

_MIME_BY_SUFFIX = {
    ".png": "image/png",
    ".jpg": "image/jpeg",
    ".jpeg": "image/jpeg",
    ".webp": "image/webp",
    ".gif": "image/gif",
    ".bmp": "image/bmp",
    ".heic": "image/heic",
    ".wav": "audio/wav",
    ".mp3": "audio/mpeg",
    ".ogg": "audio/ogg",
}


class VisionError(RuntimeError):
    """Fallo del pipeline multimodal (archivo ilegible, vacío o demasiado grande)."""


# --------------------------------------------------------------------------
# Codificación
# --------------------------------------------------------------------------
def b64_stream(source, chunk_size: int = CHUNK_SIZE, max_bytes: Optional[int] = None) -> bytearray:
    """Codifica a base64 leyendo ``source`` en trozos, sin cargarlo entero.

    ``source`` es cualquier objeto con ``.read(n)`` que devuelva ``bytes``.
    """
    out = bytearray()
    read = 0
    while True:
        chunk = source.read(chunk_size)
        if not chunk:
            break
        read += len(chunk)
        if max_bytes is not None and read > max_bytes:
            raise VisionError(
                "Imagen de {} bytes supera el limite de {} bytes".format(read, max_bytes)
            )
        out += base64.b64encode(chunk)
    return out


def encode_file_b64(path: Union[str, os.PathLike], max_bytes: Optional[int] = None) -> bytearray:
    """Base64 de un archivo en UNA sola lectura por streaming."""
    p = Path(path)
    try:
        with p.open("rb") as fh:
            return b64_stream(fh, max_bytes=max_bytes)
    except OSError as exc:
        raise VisionError("No se pudo leer {}: {}".format(p, exc)) from exc


def guess_mime(path: Union[str, os.PathLike], default: str = "image/png") -> str:
    return _MIME_BY_SUFFIX.get(Path(path).suffix.lower(), default)


# --------------------------------------------------------------------------
# Empaquetado
# --------------------------------------------------------------------------
def pack_inline_data(
    path: Union[str, os.PathLike],
    *,
    mime: Optional[str] = None,
    max_bytes: Optional[int] = None,
    data: Optional[bytes] = None,
) -> Dict[str, Any]:
    """Devuelve el part ``inlineData`` de Gemini leyendo el archivo una sola vez.

    Si ya se dispone del ``data`` base64 (p. ej. de ``build_payload_bytes``), se
    reutiliza en vez de volver a leer el disco.
    """
    p = Path(path)
    payload = data if data is not None else encode_file_b64(p, max_bytes=max_bytes)
    if not payload:
        raise VisionError("El archivo {} esta vacio".format(p))
    return {"inlineData": {"mimeType": mime or guess_mime(p), "data": payload}}


def vision_contents(
    prompt: str,
    image_paths: Sequence[Union[str, os.PathLike]],
    *,
    max_bytes: Optional[int] = None,
) -> List[Dict[str, Any]]:
    """``contents`` de Gemini con texto + N imágenes inline (ruta cómoda)."""
    parts: List[Dict[str, Any]] = [{"text": prompt}]
    for path in image_paths:
        parts.append(pack_inline_data(path, max_bytes=max_bytes))
    return [{"role": "user", "parts": parts}]


def build_payload_bytes(
    prompt: str,
    image_paths: Sequence[Union[str, os.PathLike]],
    *,
    generation_config: Optional[Dict[str, Any]] = None,
    max_bytes: Optional[int] = None,
    chunk_size: int = CHUNK_SIZE,
) -> bytes:
    """Construye el cuerpo JSON completo de ``generateContent`` en una pasada.

    El base64 se va escribiendo directamente en el buffer de salida mientras se
    lee el archivo: nunca existe un ``str`` intermedio con la imagen entera ni
    un archivo temporal de por medio.
    """
    paths = [Path(p) for p in image_paths]
    if not paths:
        raise VisionError("build_payload_bytes requiere al menos una imagen")

    skeleton_parts: List[Dict[str, Any]] = []
    for path in paths:
        if not path.exists():
            raise VisionError("No existe la imagen {}".format(path))
        skeleton_parts.append(
            {
                "inlineData": {
                    "mimeType": guess_mime(path),
                    "data": _B64_SENTINEL,
                }
            }
        )

    body = {
        "contents": [
            {"role": "user", "parts": [{"text": prompt}, *skeleton_parts]}
        ],
        "generationConfig": dict(generation_config or {"temperature": 0.4}),
    }
    # ensure_ascii=True garantiza que el esqueleto sea ASCII puro, de modo que
    # el indice en bytes coincida con el indice en caracteres al suturar.
    skeleton = json.dumps(body, ensure_ascii=True, separators=(",", ":"))
    encoded = skeleton.encode("ascii")
    # json escapa los bytes de control del centinela (\u0000), asi que el marcador
    # se deriva del propio json.dumps para que coincida byte a byte.
    marker = json.dumps(_B64_SENTINEL, ensure_ascii=True).encode("ascii")

    out = bytearray()
    cursor = 0
    for path in paths:
        found = encoded.find(marker, cursor)
        if found < 0:
            raise VisionError("Fallo interno al suturar inline_data de {}".format(path))
        out += encoded[cursor:found]
        out += b'"'  # el marcador incluye las comillas; el base64 va entre ellas
        written = 0
        with path.open("rb") as fh:
            while True:
                chunk = fh.read(chunk_size)
                if not chunk:
                    break
                written += len(chunk)
                if max_bytes is not None and written > max_bytes:
                    raise VisionError(
                        "Imagen {} de {} bytes supera el limite de {} bytes".format(
                            path, written, max_bytes
                        )
                    )
                out += base64.b64encode(chunk)
        out += b'"'
        cursor = found + len(marker)
    out += encoded[cursor:]

    # El centinela viaja escapado por json (\u0000); si queda algún resto, la
    # sutura falló y es mejor abortar que enviar un payload corrupto.
    escaped = marker[1:-1]  # sin las comillas envolventes
    if escaped in out:
        raise VisionError("Fallo interno: centinela inline_data sin suturar")
    return bytes(out)


def describe_image_for_text_models(image_paths: Sequence[Union[str, os.PathLike]]) -> str:
    """Degradación elegante cuando el modelo activo no admite ``inline_data``.

    En vez de reintentar la llamada (más cuota y más RAM), se informa al modelo
    de la ruta local para que decida la acción siguiente.
    """
    listed = ", ".join(str(p) for p in image_paths) or "(ninguna)"
    return (
        "[NEXUS] El modelo activo no acepta imagenes inline_data. "
        "Captura disponible en: {}. Responde indicando la accion siguiente "
        "o solicita un modelo con vision (NEXUS_GEMINI_MODEL_CHAIN).".format(listed)
    )


# --------------------------------------------------------------------------
# Captura
# --------------------------------------------------------------------------
def capture_screenshot(socket_path: Optional[str] = None, *, timeout: float = 20.0) -> str:
    """Pide una captura al ``nexus_host_daemon`` y devuelve la ruta resultante.

    La imagen NO se copia a un temporal intermedio: el daemon ya escribe en la
    raíz portable y esta función consume directamente ese archivo.
    """
    import socket as _socket

    target = socket_path or os.environ.get("NEXUS_SOCKET_PATH")
    if not target:
        home = os.environ.get("HOME") or os.path.expanduser("~")
        target = os.path.join(home, ".nexus_host.sock")

    client = _socket.socket(_socket.AF_UNIX, _socket.SOCK_STREAM)
    client.settimeout(timeout)
    try:
        client.connect(target)
        client.sendall(b'{"action":"Screenshot"}\n')
        raw = b""
        while b"\n" not in raw:
            block = client.recv(4096)
            if not block:
                break
            raw += block
    finally:
        client.close()

    if not raw.strip():
        raise VisionError("El daemon no respondio a la accion Screenshot")
    payload = json.loads(raw.decode("utf-8"))
    result = str(payload.get("result", ""))
    if not payload.get("ok") or not result:
        raise VisionError("Captura rechazada por el daemon: {}".format(result or payload))
    return result
