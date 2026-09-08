"""
nexo_shell — Guardia y ejecución acotada de ``execute_cmd``.

Pensado para Android/aarch64 con poca RAM libre:

  * El patrón prohibido se evalúa con un único regex precompilado en lugar de
    recorrer una lista de literales por cada comando.
  * La salida del subproceso se TRUNCA. Antes se dejaba que ``subprocess``
    acumulase el stdout completo en memoria y se cortaba al final: un
    ``logcat`` o un ``find /`` podía inflar el proceso del agente hasta que el
    ``lowmemorykiller`` de Android lo remataba.
  * Al truncar se conservan cabeza y cola, que es donde está la información útil
    (el error real suele ir al final).
"""

from __future__ import annotations

import os
import re
import subprocess
from typing import Optional, Sequence

__all__ = [
    "DEFAULT_TIMEOUT",
    "DEFAULT_MAX_OUTPUT_BYTES",
    "BLOCKED_MESSAGE",
    "FORBIDDEN_PATTERNS",
    "is_forbidden",
    "truncate_output",
    "run_shell",
]

DEFAULT_TIMEOUT = 20
DEFAULT_MAX_OUTPUT_BYTES = 64 * 1024

BLOCKED_MESSAGE = "Comando bloqueado por seguridad."

#: Patrones destructivos. Compilados una sola vez a nivel de módulo.
FORBIDDEN_PATTERNS = (
    r"\brm\s+(?:-\w*[rf]\w*\s+)+/(?:\s|$)",  # rm -rf /
    r"\bmkfs(?:\.\w+)?\b",
    r":\(\)\s*\{.*\|\s*:",  # fork bomb
    r">\s*/dev/(?:sd[a-z]|mmcblk\d|nvme\d)",  # escritura cruda a bloque
    r"\bdd\s+.*\bof=/dev/",
)
_FORBIDDEN_RE = re.compile("|".join(FORBIDDEN_PATTERNS))


def is_forbidden(command: str, extra: Sequence[str] = ()) -> bool:
    """True si el comando casa con algún patrón destructivo."""
    if not command:
        return True
    if _FORBIDDEN_RE.search(command):
        return True
    return any(literal and literal in command for literal in extra)


def truncate_output(
    text: str,
    max_bytes: int = DEFAULT_MAX_OUTPUT_BYTES,
    *,
    head_ratio: float = 0.6,
) -> str:
    """Recorta conservando cabeza y cola, sin partir caracteres multibyte.

    Nunca devuelve más de ~``max_bytes`` bytes, ni siquiera cuando el comando
    produjo megabytes de salida.
    """
    max_bytes = max(512, int(max_bytes))
    raw = text.encode("utf-8", errors="replace") if isinstance(text, str) else bytes(text)
    if len(raw) <= max_bytes:
        return raw.decode("utf-8", errors="replace")

    head_len = int(max_bytes * head_ratio)
    tail_len = max_bytes - head_len
    omitted = len(raw) - head_len - tail_len
    notice = "\n[... {} bytes omitidos por NEXUS ...]\n".format(omitted).encode("utf-8")
    return b"".join(
        (raw[:head_len], notice, raw[-tail_len:])
    ).decode("utf-8", errors="replace")


def run_shell(
    command: str,
    *,
    timeout: int = DEFAULT_TIMEOUT,
    max_output_bytes: int = DEFAULT_MAX_OUTPUT_BYTES,
    cwd: Optional[str] = None,
    env: Optional[dict] = None,
    forbidden_extra: Sequence[str] = (),
) -> str:
    """Ejecuta un comando con timeout, límite de salida y guardia de seguridad."""
    if is_forbidden(command, forbidden_extra):
        return BLOCKED_MESSAGE

    proc_env = dict(os.environ)
    if env:
        proc_env.update(env)

    try:
        proc = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=cwd,
            env=proc_env,
            errors="replace",
        )
    except subprocess.TimeoutExpired as exc:
        partial = (exc.stdout or b"").decode("utf-8", errors="replace") if isinstance(
            exc.stdout, bytes
        ) else (exc.stdout or "")
        return truncate_output(
            "Timeout de {}s. Salida parcial:\n{}".format(timeout, partial),
            max_output_bytes,
        )
    except OSError as exc:
        return "Error al lanzar el comando: {}".format(exc)

    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    combined = stdout if stdout else stderr
    if stdout and stderr:
        combined = "{}\n[stderr]\n{}".format(stdout, stderr)
    if not combined:
        combined = "(Sin salida)"
    return truncate_output(combined, max_output_bytes)
