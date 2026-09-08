#!/usr/bin/env python3
"""
check_termux_env — Comprueba que, con un $PREFIX de Termux presente, la
resolución de rutas cae dentro del prefijo y no en el temporal del sistema.

Se usa en CI para validar el contrato de portabilidad en un runner que NO es
Android. Sale con código 1 si el contrato se rompe.

    PREFIX=/ruta/usr TMPDIR=/ruta/usr/tmp python tools/check_termux_env.py
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "agents"))

import nexo_plataforma as plataforma  # noqa: E402


def main() -> int:
    prefix = os.environ.get("PREFIX", "")
    tmpdir = os.environ.get("TMPDIR", "")
    fallos = []

    print("PREFIX  =", prefix or "(no definido)")
    print("TMPDIR  =", tmpdir or "(no definido)")

    if not prefix:
        print("::error::Esta comprobación requiere un $PREFIX de Termux")
        return 1

    if not plataforma.is_android():
        fallos.append("no se detectó Android pese a tener $PREFIX")

    raiz = plataforma.temp_root()
    print("temp_root =", raiz)

    esperado = tmpdir or os.path.join(prefix, "tmp")
    if os.path.normpath(raiz) != os.path.normpath(esperado):
        fallos.append(
            "temp_root={} pero se esperaba {}".format(raiz, esperado)
        )

    if not raiz.startswith(os.path.normpath(prefix)):
        fallos.append("temp_root={} queda fuera de $PREFIX".format(raiz))

    perfil = plataforma.memory_profile()
    print("perfil de memoria =", perfil["level"])

    if fallos:
        for fallo in fallos:
            print("::error::{}".format(fallo))
        return 1

    print("OK: entorno Termux resuelto sin rutas duras")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
