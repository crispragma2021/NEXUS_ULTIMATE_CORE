"""
nexo_plataforma — Resolución de rutas portables para NEXUS.

Un único punto de verdad para decidir DÓNDE vive cada archivo temporal, de forma
que el mismo código corra sin modificaciones en:

  * Android / Termux (aarch64) -> $TMPDIR, $PREFIX/tmp, $HOME/tmp
  * PC escritorio (Linux)      -> $TMPDIR, /tmp        (comportamiento original)
  * PC escritorio (Windows)    -> %TEMP%, %TMP%, %LOCALAPPDATA%\\Temp

Regla de diseño: ninguna ruta dura a /tmp en el resto del proyecto. Todo pasa
por `temp_root()` / `temp_path()`, que resuelven en tiempo de ejecución.
"""

from __future__ import annotations

import os
import sys
import tempfile
from pathlib import Path
from typing import Iterable, List, Optional

__all__ = [
    "detect_platform",
    "is_android",
    "is_termux",
    "is_windows",
    "temp_root",
    "temp_path",
    "available_memory_bytes",
    "memory_profile",
]


# --------------------------------------------------------------------------
# Detección de plataforma
# --------------------------------------------------------------------------
def detect_platform(env: Optional[dict] = None) -> str:
    """Devuelve 'android' | 'windows' | 'macos' | 'linux'.

    `env` es inyectable para pruebas; por defecto usa ``os.environ``.

    Orden de decisión:
      1. ``NEXUS_PLATFORM`` explícito (contenedores, CI, Termux no estándar).
      2. ``sys.platform`` empieza por ``android``.
      3. ``$PREFIX`` contiene ``com.termux`` (heurística de Termux).
      4. El ``sys.platform`` que toque.
    """
    env = os.environ if env is None else env

    forzado = str(env.get("NEXUS_PLATFORM", "")).strip().lower()
    if forzado in ("android", "windows", "macos", "linux"):
        return forzado

    prefix = str(env.get("PREFIX", ""))
    if sys.platform.startswith("android") or "com.termux" in prefix:
        return "android"
    if sys.platform == "win32":
        return "windows"
    if sys.platform == "darwin":
        return "macos"
    return "linux"


def is_android(env: Optional[dict] = None) -> bool:
    return detect_platform(env) == "android"


def is_termux(env: Optional[dict] = None) -> bool:
    """Termux es Android + un $PREFIX propio."""
    env = os.environ if env is None else env
    return is_android(env) and bool(str(env.get("PREFIX", "")).strip())


def is_windows(env: Optional[dict] = None) -> bool:
    return detect_platform(env) == "windows"


# --------------------------------------------------------------------------
# Raíz temporal
# --------------------------------------------------------------------------
def _dir_is_usable(path: str) -> bool:
    """Un directorio es usable si existe (o se puede crear) y es escribible."""
    if not path:
        return False
    try:
        os.makedirs(path, exist_ok=True)
    except OSError:
        return False
    return os.path.isdir(path) and os.access(path, os.W_OK | os.X_OK)


def _candidates(env: dict, platform: str) -> List[str]:
    """Orden de preferencia por plataforma. El primero usable gana."""
    override = env.get("NEXUS_TMPDIR")
    if override:
        return [override]

    tmpdir = env.get("TMPDIR")
    prefix = env.get("PREFIX")
    home = env.get("HOME")

    if platform == "android":
        # Termux expone $TMPDIR (=/data/data/com.termux/files/usr/tmp) y $PREFIX.
        # /tmp NO existe en Android: jamás se ofrece como candidato.
        out: List[str] = []
        if tmpdir:
            out.append(tmpdir)
        if prefix:
            out.append(os.path.join(prefix, "tmp"))
        if home:
            out.append(os.path.join(home, "tmp"))
        return out

    if platform == "windows":
        out = []
        for var in ("TEMP", "TMP"):
            if env.get(var):
                out.append(env[var])
        local = env.get("LOCALAPPDATA")
        if local:
            out.append(os.path.join(local, "Temp"))
        return out

    # linux / macos -> comportamiento de PC intacto.
    return [tmpdir or "", tempfile.gettempdir()]


def temp_root(env: Optional[dict] = None) -> str:
    """Raíz temporal resuelta dinámicamente. Siempre devuelve un path absoluto."""
    env = os.environ if env is None else env
    platform = detect_platform(env)

    for cand in _candidates(env, platform):
        if cand and os.path.isabs(cand) and _dir_is_usable(cand):
            return os.path.normpath(cand)

    # Ningún candidato es usable: degradar al directorio de trabajo y por último
    # a HOME. No se consulta tempfile.gettempdir() aquí porque lee os.environ a
    # espaldas del `env` recibido, lo que haría el resultado impredecible.
    # temp_root() nunca lanza: un agente no debe morir por una ruta temporal.
    for ultimo in (os.getcwd(), env.get("HOME") or os.path.expanduser("~")):
        if _dir_is_usable(ultimo):
            return os.path.normpath(ultimo)
    return os.path.normpath(os.getcwd())


def temp_path(*parts: str, env: Optional[dict] = None) -> str:
    """Une `parts` bajo la raíz temporal portable."""
    return os.path.join(temp_root(env), *parts)


# --------------------------------------------------------------------------
# Perfil de memoria (dispositivos aarch64 con poca RAM libre)
# --------------------------------------------------------------------------
def _read_proc_meminfo() -> Optional[int]:
    try:
        with open("/proc/meminfo", "r", encoding="utf-8", errors="ignore") as fh:
            for line in fh:
                if line.startswith("MemAvailable:"):
                    return int(line.split()[1]) * 1024
    except OSError:
        return None
    return None


def _read_vm_stat() -> Optional[int]:
    """macOS: páginas libres + inactivas * page size."""
    import subprocess

    try:
        out = subprocess.run(
            ["vm_stat"], capture_output=True, text=True, timeout=5
        ).stdout
    except (OSError, subprocess.SubprocessError):
        return None
    page_size, free, inactive = 4096, 0, 0
    for line in out.splitlines():
        low = line.lower()
        if "page size of" in low:
            try:
                page_size = int(line.split()[-2])
            except (ValueError, IndexError):
                pass
        elif low.startswith("pages free"):
            free = int("".join(ch for ch in line.split(":")[-1] if ch.isdigit()) or 0)
        elif low.startswith("pages inactive"):
            inactive = int(
                "".join(ch for ch in line.split(":")[-1] if ch.isdigit()) or 0
            )
    return (free + inactive) * page_size


def available_memory_bytes() -> Optional[int]:
    """RAM disponible en bytes, o None si el SO no la expone (Windows)."""
    if sys.platform.startswith(("linux", "android")):
        return _read_proc_meminfo()
    if sys.platform == "darwin":
        return _read_vm_stat()
    return None


def memory_profile(
    env: Optional[dict] = None, available: Optional[int] = None
) -> dict:
    """Perfil de operación según RAM libre.

    'critical' (< 448 MiB) y 'low' (< 1 GiB) recortan buffers y apagan trazas
    para no provocar el OOM-killer de Android sobre aarch64. 'normal' mantiene
    el comportamiento de PC.
    """
    env = os.environ if env is None else env
    available = available_memory_bytes() if available is None else available

    forced = str(env.get("NEXUS_MEM_PROFILE", "")).strip().lower()
    if forced in ("normal", "low", "critical"):
        level = forced
    elif available is None:
        level = "normal"
    elif available < 448 * 1024 * 1024:
        level = "critical"
    elif available < 1024 * 1024 * 1024:
        level = "low"
    else:
        level = "normal"

    table = {
        "normal": {
            "max_output_bytes": 65536,
            "max_history_turns": 8,
            "cache_audit": True,
            "transport_debug": True,
            "screenshot_max_bytes": 12 * 1024 * 1024,
        },
        "low": {
            "max_output_bytes": 24576,
            "max_history_turns": 4,
            "cache_audit": False,
            "transport_debug": False,
            "screenshot_max_bytes": 6 * 1024 * 1024,
        },
        "critical": {
            "max_output_bytes": 8192,
            "max_history_turns": 2,
            "cache_audit": False,
            "transport_debug": False,
            "screenshot_max_bytes": 3 * 1024 * 1024,
        },
    }
    profile = dict(table[level])
    profile["level"] = level
    profile["available_bytes"] = available
    return profile
