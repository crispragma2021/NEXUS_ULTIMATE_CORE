#!/usr/bin/env python3
"""
bench_multimodal — Mide el pipeline inline_data contra el método clásico.

Ejecuta cada método en un SUBPROCESO distinto: `ru_maxrss` es una marca de agua
máxima del proceso y nunca baja, así que medir ambos en el mismo proceso daría
un empate falso.

    python3 tools/bench_multimodal.py
    python3 tools/bench_multimodal.py --mb 8 --repeticiones 5
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
AGENTES = REPO / "agents"

VIEJO = r'''
import base64, json, resource, time, sys
img = sys.argv[1]
t = time.time()
raw = open(img, 'rb').read()
b64 = base64.b64encode(raw).decode()
cuerpo = {"contents": [{"role": "user", "parts": [
    {"text": "q"},
    {"inlineData": {"mimeType": "image/png", "data": b64}}]}],
    "generationConfig": {"temperature": 0.4}}
payload = json.dumps(cuerpo).encode("utf-8")
ms = (time.time() - t) * 1000
rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024
print("%.3f %.2f %d" % (ms, rss, len(payload)))
'''

NUEVO = r'''
import resource, sys, time
sys.path.insert(0, sys.argv[2])
import nexo_vision as v
img = sys.argv[1]
t = time.time()
payload = v.build_payload_bytes("q", [img])
ms = (time.time() - t) * 1000
rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024
print("%.3f %.2f %d" % (ms, rss, len(payload)))
'''

BASE = r'''
import resource
print("%.3f %.2f 0" % (0.0, resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024))
'''


def medir(codigo: str, imagen: str) -> tuple:
    salida = subprocess.run(
        [sys.executable, "-c", codigo, imagen, str(AGENTES)],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    ms, rss, tamano = salida.split()
    return float(ms), float(rss), int(tamano)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mb", type=float, default=4.0, help="tamaño de imagen en MB")
    parser.add_argument("--repeticiones", type=int, default=3)
    args = parser.parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        imagen = os.path.join(tmp, "captura.png")
        with open(imagen, "wb") as fh:
            escrito = 0
            bloque = bytes(range(256)) * 64  # 16 KB
            while escrito < args.mb * 1024 * 1024:
                fh.write(bloque)
                escrito += len(bloque)
        real = os.path.getsize(imagen) / 1e6

        base = min(medir(BASE, imagen)[1] for _ in range(args.repeticiones))
        print("imagen de prueba: {:.1f} MB".format(real))
        print("linea base del interprete: {:.1f} MB de RSS\n".format(base))

        resultados = {}
        for nombre, codigo in (("clasico", VIEJO), ("nexus", NUEVO)):
            tomas = [medir(codigo, imagen) for _ in range(args.repeticiones)]
            ms = min(t[0] for t in tomas)
            rss = min(t[1] for t in tomas)
            tamano = tomas[0][2]
            resultados[nombre] = (ms, rss, tamano)
            print(
                "{:<8} {:>7.1f} ms | pico RSS {:>6.1f} MB | sobre base {:>6.1f} MB | payload {} bytes".format(
                    nombre, ms, rss, rss - base, tamano
                )
            )

        viejo, nuevo = resultados["clasico"], resultados["nexus"]
        print(
            "\ndelta: {:.1f} ms menos ({:.1f}x mas rapido), {:.1f} MB menos de pico RSS ({:.0f}% menos)".format(
                viejo[0] - nuevo[0],
                viejo[0] / nuevo[0] if nuevo[0] else 0,
                viejo[1] - nuevo[1],
                100 * (viejo[1] - nuevo[1]) / viejo[1] if viejo[1] else 0,
            )
        )
        print(
            "nota: los payloads son JSON equivalente; difieren en {} bytes por los "
            "separadores compactos.".format(abs(viejo[2] - nuevo[2]))
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
