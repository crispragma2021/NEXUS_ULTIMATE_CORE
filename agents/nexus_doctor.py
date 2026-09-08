#!/usr/bin/env python3
"""
nexus_doctor — Diagnóstico de una pantalla: rutas resueltas, memoria y llaves.

Útil para confirmar en el propio dispositivo (Termux o PC) que la resolución de
rutas y el perfil de memoria son los esperados antes de culpar al agente.

    nexus doctor
    python3 agents/nexus_doctor.py --json
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from typing import Any, Dict

if __package__ in (None, ""):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import nexo_api  # noqa: E402
import nexo_plataforma as plataforma  # noqa: E402
import nexo_proveedores  # noqa: E402


def collect() -> Dict[str, Any]:
    perfil = plataforma.memory_profile()
    disponibles = perfil.get("available_bytes")
    return {
        "plataforma": plataforma.detect_platform(),
        "es_termux": plataforma.is_termux(),
        "python": sys.version.split()[0],
        "temp_root": plataforma.temp_root(),
        "socket": os.environ.get(
            "NEXUS_SOCKET_PATH",
            os.path.join(os.environ.get("HOME", "~"), ".nexus_host.sock"),
        ),
        "memoria": {
            "perfil": perfil["level"],
            "disponible_mb": (disponibles // (1024 * 1024)) if disponibles else None,
            "cache_audit": perfil["cache_audit"],
            "max_output_bytes": perfil["max_output_bytes"],
            "max_history_turns": perfil["max_history_turns"],
        },
        "modelos": {
            "cadena": list(nexo_api.default_chain()),
            "llaves_gemini": len(nexo_api.load_pool_keys()),
            "llave_deepseek": bool(os.environ.get("DEEPSEEK_API_KEY")),
        },
        "proveedores": nexo_proveedores.resumen(),
        "cascada": nexo_proveedores.cadena_cascada(),
    }


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(prog="nexus doctor")
    parser.add_argument("--json", action="store_true", help="salida en JSON")
    args = parser.parse_args(argv)

    datos = collect()
    if args.json:
        print(json.dumps(datos, indent=2, ensure_ascii=False))
        return 0

    print("[i] plataforma : {}".format(datos["plataforma"]))
    print("[i] termux     : {}".format("sí" if datos["es_termux"] else "no"))
    print("[i] temp_root  : {}".format(datos["temp_root"]))
    print("[i] socket     : {}".format(datos["socket"]))
    mem = datos["memoria"]
    print(
        "[i] memoria    : perfil={} disponible={} MB".format(
            mem["perfil"], mem["disponible_mb"]
        )
    )
    print(
        "[i] límites    : salida={} B, historial={} turnos, cache_audit={}".format(
            mem["max_output_bytes"], mem["max_history_turns"], mem["cache_audit"]
        )
    )
    modelos = datos["modelos"]
    print("[i] modelos    : {}".format(" -> ".join(modelos["cadena"])))
    print(
        "[i] llaves     : gemini={}, deepseek={}".format(
            modelos["llaves_gemini"], "sí" if modelos["llave_deepseek"] else "no"
        )
    )
    print("[i] cascada    : {}".format(datos["cascada"]))
    for prov in datos["proveedores"]:
        if not prov["armado"]:
            continue
        marca = "OK " if prov["listo"] else "-- "
        print(
            "    [{}] {:<11} restantes hoy: {:>6} | rpm usados: {:>3} | {}".format(
                marca, prov["nombre"], prov["restantes_hoy"], prov["rpm_usados"], prov["motivo"]
            )
        )
    if datos["cascada"].startswith("(ningun"):
        print("[!] Sin proveedores armados: copia .env.example a .env y rellena al menos una llave.")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
