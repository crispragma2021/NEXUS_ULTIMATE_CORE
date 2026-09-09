#!/usr/bin/env python3
"""
🧩 NEXUS CAPTCHA RESOLVER LOCAL v2 — Stack escalonado Soberano
==============================================================
Resuelve CAPTCHAs de forma LOCAL, sin depender de servicios externos.
Diseñado para los 64GB RAM + i7-12700 del Arquitecto: cada tier se carga
BAJO DEMANDA (lazy), solo cuando el tier anterior no es suficiente.

  TIER 0 (CNN rápida)          → clasificador ligero de caracteres/dígitos
                                 (onxxruntime/tf si está instalado; si no, se salta)
  TIER 1 (OCR de precisión)    → PaddleOCR (>tesseract en texto distorsionado),
                                 con tesseract como fallback nativo
  TIER 2 (OCR nativo)          → tesseract local (siempre disponible)
  TIER 3 (visión semántica)    → Qwen2.5-VL vía Ollama local (CAPTCHA abstracto)
  TIER 4 (Proof-of-Work)       → no resoluble localmente → fallback Capsolver/evasión

PILOS DE SOBERANÍA:
  - Cero API keys. Todo corre en la máquina del Arquitecto.
  - Carga bajo demanda: no se importa PaddleOCR (pesado) salvo que se necesite.
  - Prevención > resolución: fingerprint estable, IP limpia, input humano.

USO:
  nexus_captcha_local.py detect <imagen.png>          → detectar tipo
  nexus_captcha_local.py solve <imagen.png> [tipo]    → resolver (stack completo)
  nexus_captcha_local.py solve-text <imagen.png>      → forzar OCR (PaddleOCR→tesseract)
  nexus_captcha_local.py solve-vision <imagen.png>    → forzar Ollama visión
  nexus_captcha_local.py solve-cnn <imagen.png>       → forzar CNN (si disponible)
"""
import argparse
import base64
import json
import os
import re
import subprocess
import sys

# El contenedor CUA usa network_mode host → alcanza Ollama en 127.0.0.1.
OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://127.0.0.1:11434")
VISION_MODEL = os.environ.get("OLLAMA_VISION_MODEL", "qwen2.5vl:7b")
# Alternativas: "qwen2.5vl:7b", "llava:13b", "bakllava", "moondream", "gemma3:27b"


# ─── Detección de tipo ─────────────────────────────────────────────────────
def _image_bytes(path_or_b64: str) -> bytes:
    if os.path.exists(path_or_b64):
        with open(path_or_b64, "rb") as f:
            return f.read()
    return base64.b64decode(path_or_b64)


def _escribir_tmp(data: bytes, suffix: str = ".png"):
    import tempfile
    fd, tmp = tempfile.mkstemp(suffix=suffix)
    with os.fdopen(fd, "wb") as f:
        f.write(data)
    return tmp


def _limpiar_tmp(*paths):
    for p in paths:
        if p and os.path.exists(p):
            try:
                os.unlink(p)
            except OSError:
                pass


def detect_type(path_or_b64: str) -> dict:
    """Detecta el tipo de CAPTCHA por análisis básico de la imagen."""
    try:
        data = _image_bytes(path_or_b64)
    except Exception as e:  # noqa: BLE001
        return {"ok": False, "error": f"no se pudo leer imagen: {e}"}

    size = len(data)
    ext = "b64"
    if os.path.exists(path_or_b64):
        ext = os.path.splitext(path_or_b64)[1].lower()

    tipo = "desconocido"
    confianza = 0.5

    # Imágenes OCR de texto suelen ser pequeñas (< 150KB) y con poco color
    if size < 150_000:
        tipo = "texto_ocr"
        confianza = 0.7
    elif size < 400_000:
        tipo = "semantico"
        confianza = 0.6
    else:
        tipo = "semantico"
        confianza = 0.5

    return {
        "ok": True,
        "tipo": tipo,
        "confianza": confianza,
        "bytes": size,
        "extension": ext,
    }


# ─── TIER 0: CNN (carga bajo demanda) ──────────────────────────────────────
_CNN_CACHE = {"modelo": None, "error": None}


def _cargar_cnn():
    """Carga un clasificador CNN ligero si está disponible (onnx/tf). No rompe si falta."""
    if _CNN_CACHE["error"] is not None:
        return None
    if _CNN_CACHE["modelo"] is not None:
        return _CNN_CACHE["modelo"]
    # Intentar carga vía onnxruntime (modelo local .onnx) o tf-lite.
    for mod_name in ("onnxruntime", "tensorflow"):
        try:
            __import__(mod_name)
            _CNN_CACHE["modelo"] = True  # marcador: motor disponible
            return _CNN_CACHE["modelo"]
        except Exception:  # noqa: BLE001
            continue
    _CNN_CACHE["error"] = "sin motor CNN (onnx/tf no instalado)"
    return None


def solve_cnn(path_or_b64: str) -> dict:
    """Tier 0: clasificación de dígitos/caracteres con CNN (si está disponible)."""
    try:
        modelo = _cargar_cnn()
        if modelo is None:
            return {"ok": False, "metodo": "cnn", "skipped": _CNN_CACHE.get("error", "no disponible")}
        # Punto de integración: cargar pesos reales aquí. Hoy devolvemos señal
        # de que el motor existe y delegamos al OCR para no romper la cadena.
        return {"ok": False, "metodo": "cnn", "skipped": "motor disponible pero sin pesos; delegando a OCR"}
    except Exception as e:  # noqa: BLE001
        return {"ok": False, "metodo": "cnn", "error": str(e)}


# ─── TIER 1: PaddleOCR (carga bajo demanda) ────────────────────────────────
_PADDLE_CACHE = {"ocr": None, "error": None}


def _cargar_paddle():
    """Importa PaddleOCR solo cuando se necesita (pesado, ~import lento)."""
    if _PADDLE_CACHE["error"] is not None:
        return None
    if _PADDLE_CACHE["ocr"] is not None:
        return _PADDLE_CACHE["ocr"]
    try:
        from paddleocr import PaddleOCR  # noqa: PLC0415
        # API 2.x (paddlepaddle 2.6.2): use_angle_cls + show_log soportados.
        ocr = PaddleOCR(use_angle_cls=True, lang="en", show_log=False)
        _PADDLE_CACHE["ocr"] = ocr
        return ocr
    except Exception as e:  # noqa: BLE001
        _PADDLE_CACHE["error"] = str(e)
        return None


def _ocr_paddle(img_path: str) -> str:
    """OCR con PaddleOCR (mayor precisión en texto distorsionado que tesseract)."""
    ocr = _cargar_paddle()
    if ocr is None:
        return ""
    res = ocr.ocr(img_path, cls=True)
    tokens = []
    if not res:
        return ""
    for line in res:
        if not line:
            continue
        for item in line:
            try:
                tokens.append(item[1][0])
            except Exception:  # noqa: BLE001
                continue
    return "".join(tokens).strip()


# ─── TIER 2: tesseract OCR (nativo, siempre disponible) ────────────────────
def _ocr_tesseract(img_path: str) -> str:
    """OCR con tesseract (fallback nativo)."""
    prepared = img_path + "_prep.png"
    try:
        subprocess.run(
            ["convert", img_path, "-resize", "200%", "-colorspace", "Gray",
             "-threshold", "60%", prepared],
            capture_output=True, timeout=20)
    except Exception:  # noqa: BLE001
        prepared = img_path
    p = subprocess.run(
        ["tesseract", prepared, "stdout", "--psm", "7",
         "-c", "tessedit_char_whitelist=abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"],
        capture_output=True, text=True, timeout=30)
    text = p.stdout.strip()
    text = re.sub(r"[\s]+", "", text)  # eliminar espacios (los captcha no suelen tener)
    if prepared != img_path:
        _limpiar_tmp(prepared)
    return text


def solve_text(path_or_b64: str) -> dict:
    """Tier 1+2: OCR de precisión con PaddleOCR y fallback tesseract."""
    tmp = None
    try:
        data = _image_bytes(path_or_b64)
        if os.path.exists(path_or_b64):
            img_path = path_or_b64
        else:
            tmp = _escribir_tmp(data)
            img_path = tmp

        # Primero PaddleOCR (si está instalado) — más preciso.
        text = _ocr_paddle(img_path)
        if text:
            return {"ok": True, "respuesta": text, "metodo": "paddleocr", "largo": len(text)}

        # Fallback tesseract (siempre disponible).
        text = _ocr_tesseract(img_path)
        if text:
            return {"ok": True, "respuesta": text, "metodo": "tesseract", "largo": len(text)}

        return {"ok": False, "metodo": "ocr", "error": "OCR vacío (ni PaddleOCR ni tesseract hallaron texto)"}
    except Exception as e:  # noqa: BLE001
        return {"ok": False, "error": str(e), "metodo": "ocr"}
    finally:
        _limpiar_tmp(tmp)


# ─── TIER 3: Ollama visión (Qwen2.5-VL) ────────────────────────────────────
def _ollama_vision(prompt: str, path_or_b64: str) -> str:
    """Consulta Ollama con imagen para CAPTCHAs semánticos."""
    import urllib.request

    data = _image_bytes(path_or_b64)
    if os.path.exists(path_or_b64):
        b64 = base64.b64encode(data).decode()
    else:
        b64 = path_or_b64 if len(data) < 1024 * 1024 * 5 else base64.b64encode(data).decode()

    payload = json.dumps({
        "model": VISION_MODEL,
        "prompt": prompt,
        "images": [b64],
        "stream": False,
        "options": {"temperature": 0.0},
    }).encode("utf-8")

    req = urllib.request.Request(
        f"{OLLAMA_URL}/api/generate", data=payload,
        headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as resp:
        result = json.loads(resp.read().decode("utf-8"))
    return result.get("response", "").strip()


def solve_vision(path_or_b64: str) -> dict:
    """Tier 3: CAPTCHAs semánticos (semáforos, buses, señales) con Ollama local."""
    try:
        prompt = (
            "Eres un resolutor de CAPTCHAs de imágenes. Observa la imagen y responde "
            "SOLO con la acción correcta, sin explicaciones. "
            "Si es texto distorsionado, transcríbelo exactamente (sin espacios). "
            "Si hay semáforos elige cuáles están en ROJO (derecha/izquierda/ambos); "
            "si hay objetos elige 'calle/vehiculo/peatones/autobus' según se pida. "
            "Responde únicamente con la respuesta correcta, en una palabra o cadena."
        )
        resp = _ollama_vision(prompt, path_or_b64)
        return {"ok": True, "respuesta": resp, "metodo": f"ollama-{VISION_MODEL}"}
    except Exception as e:  # noqa: BLE001
        return {"ok": False, "error": f"ollama visión falló: {e}", "metodo": "ollama"}


# ─── Orquestador (stack escalonado) ────────────────────────────────────────
def solve(path_or_b64: str, tipo: str = None) -> dict:
    """Resuelve un CAPTCHA escalando por tiers hasta hallar respuesta."""
    det = detect_type(path_or_b64) if not tipo else {"ok": True, "tipo": tipo}
    if not det.get("ok"):
        return {"ok": False, "tipo": "desconocido", "error": det.get("error", "no se pudo detectar el CAPTCHA")}
    tipo_det = det["tipo"]

    # Tier 0: CNN (si disponible) — intenta primero, sin bloquear la cadena.
    if tipo_det == "texto_ocr":
        cnn = solve_cnn(path_or_b64)
        if cnn.get("ok"):
            cnn["tipo"] = tipo_det
            return cnn

        # Tier 1+2: OCR de precisión (PaddleOCR → tesseract).
        ocr = solve_text(path_or_b64)
        if ocr["ok"] and ocr.get("largo", 0) > 0:
            ocr["tipo"] = tipo_det
            return ocr

        # Tier 3: visión semántica como último recurso local.
        vis = solve_vision(path_or_b64)
        vis["tipo"] = tipo_det
        return vis

    if tipo_det == "semantico":
        vis = solve_vision(path_or_b64)
        vis["tipo"] = tipo_det
        return vis

    return {
        "ok": False,
        "tipo": tipo_det,
        "error": "tipo no resoluble localmente; usar Capsolver (Proof-of-Work) o evasión biométrica",
        "fallback": "capsolver",
    }


# ─── CLI ───────────────────────────────────────────────────────────────────
def main():
    ap = argparse.ArgumentParser(description="NEXUS CAPTCHA RESOLVER LOCAL v2")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p_detect = sub.add_parser("detect")
    p_detect.add_argument("imagen")

    p_solve = sub.add_parser("solve")
    p_solve.add_argument("imagen")
    p_solve.add_argument("--tipo", default=None)

    p_text = sub.add_parser("solve-text")
    p_text.add_argument("imagen")

    p_vision = sub.add_parser("solve-vision")
    p_vision.add_argument("imagen")

    p_cnn = sub.add_parser("solve-cnn")
    p_cnn.add_argument("imagen")

    args = ap.parse_args()

    if args.cmd == "detect":
        print(json.dumps(detect_type(args.imagen), ensure_ascii=False))
    elif args.cmd == "solve":
        print(json.dumps(solve(args.imagen, args.tipo), ensure_ascii=False))
    elif args.cmd == "solve-text":
        print(json.dumps(solve_text(args.imagen), ensure_ascii=False))
    elif args.cmd == "solve-vision":
        print(json.dumps(solve_vision(args.imagen), ensure_ascii=False))
    elif args.cmd == "solve-cnn":
        print(json.dumps(solve_cnn(args.imagen), ensure_ascii=False))


if __name__ == "__main__":
    main()
