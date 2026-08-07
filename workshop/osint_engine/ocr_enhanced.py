#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════╗
║  NEXUS OSINT ENGINE - OCR Enhanced Pipeline                     ║
║  FASE 5: Preprocesamiento + OCR + Búsqueda de chapa vehicular   ║
║  © 2026 NEXUS — Soberanía Técnica                               ║
╚══════════════════════════════════════════════════════════════════╝
"""
import cv2
import numpy as np
import sys
import os
import re
import subprocess
import json
from pathlib import Path

# ─── CONFIG ────────────────────────────────────────────────────
FRAMES_DIR = Path("/home/soberano/NEXUS_ULTIMATE_CORE/downloads/videos")
OUTPUT_DIR = FRAMES_DIR / "chapa_extracts"
TESSERACT_CMD = "/usr/bin/tesseract"
TESSDATA_DIR = "/usr/share/tesseract-ocr/5/tessdata"

# Patrones de chapa paraguaya (formateados)
CHAPA_PATTERNS = [
    (r'[A-Z]{3}\s*\d{4}', 'ABC 1234 (auto moderno)'),
    (r'[A-Z]{3}\s*\d{3}', 'ABC 123 (moto)'),
    (r'\d{3}\s*[A-Z]{3}', '123 ABC (formato antiguo)'),
    (r'[A-Z]{2}\s*\d{4}', 'AB 1234'),
    (r'[A-Z]{1}\s*\d{4}', 'A 1234'),
    (r'[A-Z]{3}\s*\d{2}', 'ABC 12'),
    (r'[A-Z]{4}\s*\d{3}', 'ABCD 123'),
]

# Letras usadas en chapas paraguayas (sin Ñ, Q)
CHAPA_LETTERS = 'ABCDEFGHIJKLMNOPRSTUVWXYZ'


def preprocess_image(image_path, output_prefix="preprocessed"):
    """
    Pipeline de preprocesamiento OpenCV para optimizar OCR.
    Retorna lista de (label, path) con imágenes procesadas.
    """
    img = cv2.imread(str(image_path))
    if img is None:
        print(f"[ERROR] No se pudo leer: {image_path}")
        return []
    
    h, w = img.shape[:2]
    print(f"[INFO] Imagen: {w}x{h}, canales: {img.shape[2] if len(img.shape) > 2 else 1}")
    
    results = []
    base_name = output_prefix
    
    # ─── 1. Aumentar resolución 3x ────────────────────────────
    img_big = cv2.resize(img, (w*3, h*3), interpolation=cv2.INTER_CUBIC)
    
    # ─── 2. Escala de grises ──────────────────────────────────
    gray = cv2.cvtColor(img_big, cv2.COLOR_BGR2GRAY)
    
    # ─── 3. CLAHE (contraste adaptativo) ──────────────────────
    clahe = cv2.createCLAHE(clipLimit=4.0, tileGridSize=(8,8))
    enhanced = clahe.apply(gray)
    cv2.imwrite(str(OUTPUT_DIR / f"{base_name}_clahe.jpg"), enhanced)
    results.append(("clahe", OUTPUT_DIR / f"{base_name}_clahe.jpg"))
    
    # ─── 4. Threshold adaptativo (Gaussian) ───────────────────
    thresh = cv2.adaptiveThreshold(
        enhanced, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C,
        cv2.THRESH_BINARY, 41, 6
    )
    cv2.imwrite(str(OUTPUT_DIR / f"{base_name}_thresh.jpg"), thresh)
    results.append(("thresh", OUTPUT_DIR / f"{base_name}_thresh.jpg"))
    
    # ─── 5. Otsu ──────────────────────────────────────────────
    _, otsu = cv2.threshold(enhanced, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    cv2.imwrite(str(OUTPUT_DIR / f"{base_name}_otsu.jpg"), otsu)
    results.append(("otsu", OUTPUT_DIR / f"{base_name}_otsu.jpg"))
    
    # ─── 6. Denoise + sharpen ─────────────────────────────────
    denoised = cv2.fastNlMeansDenoising(enhanced, None, 10, 7, 21)
    kernel = np.array([[-1,-1,-1], [-1,9,-1], [-1,-1,-1]])
    sharp = cv2.filter2D(denoised, -1, kernel)
    cv2.imwrite(str(OUTPUT_DIR / f"{base_name}_sharp.jpg"), sharp)
    results.append(("sharp", OUTPUT_DIR / f"{base_name}_sharp.jpg"))
    
    # ─── 7. Invertido sobre thresh ────────────────────────────
    inv = cv2.bitwise_not(thresh)
    cv2.imwrite(str(OUTPUT_DIR / f"{base_name}_inv.jpg"), inv)
    results.append(("inv", OUTPUT_DIR / f"{base_name}_inv.jpg"))
    
    print(f"[OK] {len(results)} imágenes preprocesadas")
    return results


def ocr_image(image_path, psm=7):
    """
    OCR en una imagen con Tesseract. Retorna el texto.
    psm=7 trata como single line (ideal para chapas).
    """
    env = os.environ.copy()
    env['TESSDATA_PREFIX'] = TESSDATA_DIR
    
    try:
        result = subprocess.run(
            [TESSERACT_CMD, str(image_path), 'stdout',
             '--psm', str(psm),
             '-l', 'spa+eng',
             '-c', f'tessedit_char_whitelist={CHAPA_LETTERS}0123456789- ',
             '--oem', '3'],
            capture_output=True, text=True, timeout=15, env=env
        )
        return result.stdout.strip()
    except Exception as e:
        print(f"     [WARN] OCR error: {e}")
        return ""


def extract_plates(text):
    """Extrae posibles chapas del texto OCR."""
    found = []
    for pattern, desc in CHAPA_PATTERNS:
        matches = re.findall(pattern, text.upper())
        for m in matches:
            found.append((m.strip(), desc))
    return found


def analyze_plate_region(image_path, output_prefix="plate_crop"):
    """
    Detecta regiones de posible chapa (contornos con ratio 2:1 a 6:1)
    y extrae cada una para OCR.
    """
    img = cv2.imread(str(image_path))
    if img is None:
        return []
    
    h, w = img.shape[:2]
    img = cv2.resize(img, (w*2, h*2), interpolation=cv2.INTER_CUBIC)
    
    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    edges = cv2.Canny(gray, 30, 120)
    
    kernel = np.ones((3,3), np.uint8)
    dilated = cv2.dilate(edges, kernel, iterations=3)
    
    contours, _ = cv2.findContours(dilated, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    
    candidates = []
    for cnt in contours:
        x, y, cw, ch = cv2.boundingRect(cnt)
        aspect = cw / ch if ch > 0 else 0
        
        # Filtrar: ratio de chapa típico, tamaño mínimo
        if 2.0 < aspect < 7.0 and cw > 40 and ch > 12 and cw < w*0.9:
            pad = 3
            x1, y1 = max(0, x-pad), max(0, y-pad)
            x2, y2 = min(img.shape[1], x+cw+pad), min(img.shape[0], y+ch+pad)
            
            roi = img[y1:y2, x1:x2]
            fname = OUTPUT_DIR / f"{output_prefix}_crop_{len(candidates)}.jpg"
            cv2.imwrite(str(fname), roi)
            candidates.append({'file': fname, 'bbox': f'({x},{y},{cw},{ch})'})
    
    if candidates:
        print(f"     [🔍] {len(candidates)} regiones candidatas detectadas")
    return candidates


def search_plate_online(plate):
    """Busca chapa en motores de búsqueda."""
    results = {}
    plate_clean = re.sub(r'\s+', '', plate)
    
    sources = [
        ('DuckDuckGo', f"https://html.duckduckgo.com/html/?q=\"{plate_clean}\"+paraguay+chapa+vehiculo"),
        ('Google', f"https://www.google.com/search?q=%22{plate_clean}%22+paraguay+automovil"),
    ]
    
    for name, url in sources:
        try:
            proxy = "socks5://127.0.0.1:9050"
            cmd = ["curl", "-s", "-L", "--max-time", "15", "--proxy", proxy, url]
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=20)
            
            snippets = []
            for line in r.stdout.split('\n'):
                up = line.upper()
                if plate_clean[:4] in up or 'CORONEL' in up or 'TORRES' in up or 'CAMUS' in up:
                    clean = re.sub(r'<[^>]+>', '', line).strip()
                    if clean and len(clean) > 10:
                        snippets.append(clean[:200])
            
            results[name] = {
                'status': 'OK' if r.returncode == 0 else 'FAIL',
                'snippets': snippets[:5],
                'url': url
            }
        except Exception as e:
            results[name] = {'status': 'ERROR', 'error': str(e)}
    
    return results


def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    print("╔══════════════════════════════════════════════════╗")
    print("║   NEXUS OSINT ENGINE — OCR ENHANCED PIPELINE     ║")
    print("║   Análisis de chapa (Paraguay) — {0} frames       ║".format(
        len(list(FRAMES_DIR.glob("chapa_frame_*.jpg"))) +
        len(list(FRAMES_DIR.glob("aldoframe_*.jpg")))
    ))
    print("╚══════════════════════════════════════════════════╝")
    
    # ─── RECOLECTAR TODAS LAS IMÁGENES ──────────────────────
    all_images = []
    
    # Chapa extracts existentes
    for f in sorted(FRAMES_DIR.glob("chapa_extracts/*.jpg")):
        all_images.append(('prev_extract', f))
    
    # Frames densos nuevos (cada 5s)
    for f in sorted(FRAMES_DIR.glob("chapa_frame_*.jpg")):
        all_images.append(('dense_frame', f))
    
    # Frames originales (cada 30s)
    for f in sorted(FRAMES_DIR.glob("aldoframe_*.jpg")):
        all_images.append(('orig_frame', f))
    
    # Rostros extraídos
    for f in sorted(FRAMES_DIR.glob("aldoface_*.jpg")):
        all_images.append(('face', f))
    
    print(f"\n[📂] Total imágenes a procesar: {len(all_images)}")
    
    all_results = {}  # candidate -> list of sources
    
    for img_type, img_path in all_images:
        print(f"\n{'─'*50}")
        print(f"[{img_type}] {img_path.name} ({img_path.stat().st_size//1024}KB)")
        
        # ─── OCR DIRECTO ────────────────────────────────────
        for psm in [7, 8, 6, 13]:
            text = ocr_image(img_path, psm=psm)
            if text:
                plates = extract_plates(text)
                if plates:
                    for plate, desc in plates:
                        if plate not in all_results:
                            all_results[plate] = []
                        all_results[plate].append({
                            'source': f"{img_path.name} (direct PSM{psm})",
                            'desc': desc,
                            'text': text[:80]
                        })
                    print(f"     [🔑] {plates[0][0]} ({desc}) PSM{psm}: '{text[:60]}'")
                else:
                    # Mostrar texto encontrado incluso sin patrón exacto
                    clean = re.sub(r'[^A-Z0-9]', '', text.upper())
                    if len(clean) >= 5 and len(clean) <= 8:
                        print(f"     [⚠️] Posible chapa no estándar: '{text[:60]}' (PSM{psm})")
                        if clean not in all_results:
                            all_results[clean] = []
                        all_results[clean].append({
                            'source': f"{img_path.name} (direct PSM{psm})",
                            'desc': f'no_standard_{len(clean)}_chars',
                            'text': text[:80]
                        })
        
        # ─── PREPROCESAR + OCR ─────────────────────────────
        preprocessed = preprocess_image(img_path, img_path.stem.replace('.', '_'))
        for label, pp_path in preprocessed:
            for psm in [7, 8]:
                text = ocr_image(pp_path, psm=psm)
                if text:
                    plates = extract_plates(text)
                    if plates:
                        for plate, desc in plates:
                            if plate not in all_results:
                                all_results[plate] = []
                            all_results[plate].append({
                                'source': f"{img_path.name}/{label} PSM{psm}",
                                'desc': desc,
                                'text': text[:80]
                            })
                        print(f"     [🔑] {plates[0][0]} ({label} PSM{psm}): '{text[:60]}'")
                    else:
                        clean = re.sub(r'[^A-Z0-9]', '', text.upper())
                        if 5 <= len(clean) <= 8:
                            if clean not in all_results:
                                all_results[clean] = []
                            all_results[clean].append({
                                'source': f"{img_path.name}/{label} PSM{psm}",
                                'desc': f'no_standard_{len(clean)}_chars',
                                'text': text[:80]
                            })
        
        # ─── REGIONES CANDIDATAS ───────────────────────────
        plates = analyze_plate_region(img_path, img_path.stem)
        for pdata in plates:
            for psm in [7, 8, 6]:
                text = ocr_image(pdata['file'], psm=psm)
                if text:
                    plates_found = extract_plates(text)
                    if plates_found:
                        for plate, desc in plates_found:
                            if plate not in all_results:
                                all_results[plate] = []
                            all_results[plate].append({
                                'source': f"{img_path.name}/crop PSM{psm}",
                                'desc': desc,
                                'text': text[:80]
                            })
                            print(f"     [🔑] {plate} ({pdata['bbox']} PSM{psm}): '{text[:60]}'")
    
    # ─── REPORTE FINAL ──────────────────────────────────────
    print(f"\n{'='*60}")
    print("📊 REPORTE FINAL DE OCR")
    print(f"{'='*60}\n")
    
    if all_results:
        print(f"🔑 CHAPAS CANDIDATAS ({len(all_results)}):\n")
        for i, (candidate, sources) in enumerate(sorted(all_results.items()), 1):
            print(f"  {i}. [{candidate}]")
            print(f"     Tipo: {sources[0]['desc']}")
            print(f"     Fuentes: {len(sources)} detecciones")
            for s in sources[:3]:
                print(f"       └─ {s['source']}: \"{s['text'][:50]}\"")
            
            # Buscar online la más prometedora
            if len(candidate) >= 6:
                print(f"     🔍 Buscando en línea...")
                search_results = search_plate_online(candidate)
                for src_name, data in search_results.items():
                    if data.get('snippets'):
                        print(f"       [{src_name}] Hallazgos:")
                        for snip in data['snippets'][:3]:
                            print(f"         └─ {snip}")
                    else:
                        print(f"       [{src_name}] Sin resultados")
            print()
    else:
        print("⚠️  NO se detectaron patrones de chapa estándar.\n")
        print("   Estrategias alternativas:")
        print("   1. La chapa podría no estar legible en los frames extraídos")
        print("   2. El video muestra la chapa en movimiento (necesita deblurring)")
        print("   3. Intentar con modelo de IA (Grok/DeepSeek) para análisis visual")
        print("   4. Buscar por nombre en registros vehiculares (SET/Registro Civil)")
        print()
        # Mostrar todo el texto OCR encontrado
        print("   Texto OCR encontrado en frames:")
        for img_type, img_path in all_images[:5]:
            for psm in [7]:
                text = ocr_image(img_path, psm=psm)
                if text:
                    print(f"     [{img_path.name}] '{text[:80]}'")
    
    # ─── GUARDAR REPORTE ─────────────────────────────────────
    report_data = {
        'candidates': {k: v for k, v in all_results.items()},
        'total_candidates': len(all_results),
    }
    with open(OUTPUT_DIR / "ocr_analysis_report.json", 'w') as f:
        json.dump(report_data, f, indent=2)
    
    # Reporte texto
    with open(OUTPUT_DIR / "ocr_analysis_report.txt", 'w') as f:
        f.write("=== NEXUS OCR ENHANCED REPORT ===\n\n")
        f.write(f"Candidatos: {len(all_results)}\n\n")
        for candidate, sources in sorted(all_results.items()):
            f.write(f"[{candidate}] ({sources[0]['desc']})\n")
            for s in sources:
                f.write(f"  {s['source']}: \"{s['text']}\"\n")
            f.write("\n")
    
    print(f"\n[💾] Reportes guardados en: {OUTPUT_DIR}")
    return all_results


if __name__ == "__main__":
    results = main()
