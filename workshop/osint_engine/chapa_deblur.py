#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════╗
║  NEXUS OSINT ENGINE — Chapa Deblur + Super Resolution           ║
║  Pipeline de deblurring por movimiento + Wiener filter          ║
║  © 2026 NEXUS — Soberanía Técnica                               ║
╚══════════════════════════════════════════════════════════════════╝
"""
import cv2
import numpy as np
import subprocess
import json
import os
import re
from pathlib import Path

BASE_DIR = Path("/home/soberano/NEXUS_ULTIMATE_CORE")
VIDEOS_DIR = BASE_DIR / "downloads" / "videos"
OUTPUT_DIR = VIDEOS_DIR / "chapa_deblur"
VIDEO_PATH = VIDEOS_DIR / "aldovideo.mp4"

os.makedirs(OUTPUT_DIR, exist_ok=True)

# Patrones de chapa paraguaya
CHAPA_PATTERNS = [
    (r'[A-Z]{3}\s*\d{4}', 'ABC 1234'),
    (r'[A-Z]{3}\s*\d{3}', 'ABC 123'),
    (r'\d{3}\s*[A-Z]{3}', '123 ABC'),
    (r'[A-Z]{2}\s*\d{4}', 'AB 1234'),
    (r'[A-Z]{1}\s*\d{4}', 'A 1234'),
]


def extract_frames_dense(video_path, intervals, output_prefix='dense'):
    """Extrae frames en intervalos específicos (cada 0.5s)."""
    frames = []
    duration = 164  # duración conocida
    
    for start, end in intervals:
        for t in range(start, end + 1):
            out_path = OUTPUT_DIR / f"{output_prefix}_{t:03d}.jpg"
            if not out_path.exists():
                subprocess.run([
                    'bin/ffmpeg', '-i', str(video_path),
                    '-ss', str(t),
                    '-vframes', '1',
                    '-q:v', '1',  # máxima calidad
                    str(out_path)
                ], capture_output=True)
            frames.append((t, out_path))
    
    return frames


def motion_deblur(image, kernel_size=15, angle=None):
    """
    Deblurring Wiener filter para motion blur.
    Si angle es None, prueba múltiples ángulos.
    """
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY) if len(image.shape) == 3 else image
    
    results = []
    angles_to_try = [angle] if angle else [0, 15, 30, 45, 60, 75, 90, 105, 120, 135, 150, 165]
    
    for ang in angles_to_try:
        # Crear kernel de motion blur
        kernel = np.zeros((kernel_size, kernel_size))
        center = kernel_size // 2
        for i in range(kernel_size):
            x = int(center + (i - center) * np.cos(np.radians(ang)))
            y = int(center + (i - center) * np.sin(np.radians(ang)))
            if 0 <= x < kernel_size and 0 <= y < kernel_size:
                kernel[y, x] = 1
        kernel = kernel / kernel.sum()
        
        # Wiener deconvolution
        try:
            # FFT
            img_fft = np.fft.fft2(gray)
            kernel_fft = np.fft.fft2(kernel, s=gray.shape)
            
            # Wiener filter
            K = 0.01  # relación señal/ruido
            kernel_fft_conj = np.conj(kernel_fft)
            wiener = kernel_fft_conj / (kernel_fft * kernel_fft_conj + K)
            
            result_fft = img_fft * wiener
            result = np.abs(np.fft.ifft2(result_fft))
            result = np.clip(result, 0, 255).astype(np.uint8)
            
            results.append((ang, result))
        except:
            continue
    
    return results


def lucy_richardson_deconvolution(image, kernel_size=15, iterations=10):
    """Algoritmo Lucy-Richardson para deblurring."""
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY) if len(image.shape) == 3 else image
    
    # Kernel Gaussiano estimado
    kernel = cv2.getGaussianKernel(kernel_size, -1)
    kernel = kernel @ kernel.T
    
    # Lucy-Richardson
    img = gray.astype(np.float64)
    estimate = img.copy()
    
    for _ in range(iterations):
        # Simular blur
        blurred = cv2.filter2D(estimate, -1, kernel)
        blurred = np.maximum(blurred, 1e-8)
        
        # Ratio
        ratio = img / blurred
        
        # Actualizar estimación
        estimate = estimate * cv2.filter2D(ratio, -1, kernel)
    
    estimate = np.clip(estimate, 0, 255).astype(np.uint8)
    return estimate


def sharpen_image(image):
    """Múltiples técnicas de sharpening."""
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY) if len(image.shape) == 3 else image
    
    results = {}
    
    # 1. Unsharp masking
    blurred = cv2.GaussianBlur(gray, (0, 0), 3)
    unsharp = cv2.addWeighted(gray, 1.5, blurred, -0.5, 0)
    results['unsharp'] = unsharp
    
    # 2. Laplacian sharpening
    laplacian = cv2.Laplacian(gray, cv2.CV_64F)
    lap_sharp = cv2.subtract(gray.astype(np.float64), laplacian * 0.5)
    results['laplacian'] = np.clip(lap_sharp, 0, 255).astype(np.uint8)
    
    # 3. Kernel sharpening fuerte
    kernel = np.array([
        [-2, -2, -2],
        [-2, 17, -2],
        [-2, -2, -2]
    ]) / 9
    strong_sharp = cv2.filter2D(gray, -1, kernel)
    results['strong'] = strong_sharp
    
    return results


def detect_license_plate_region(image):
    """Detecta región de chapa usando contornos y ratio de aspecto."""
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY) if len(image.shape) == 3 else image
    
    # Escalar 4x para mejor detección
    h, w = gray.shape
    big = cv2.resize(gray, (w*4, h*4), interpolation=cv2.INTER_CUBIC)
    
    results = []
    
    # Probar múltiples técnicas de detección
    for method in ['sobel', 'canny', 'morph']:
        if method == 'sobel':
            grad_x = cv2.Sobel(big, cv2.CV_64F, 1, 0, ksize=3)
            grad_y = cv2.Sobel(big, cv2.CV_64F, 0, 1, ksize=3)
            edges = cv2.magnitude(grad_x, grad_y)
            edges = np.uint8(np.clip(edges, 0, 255))
        elif method == 'canny':
            edges = cv2.Canny(big, 30, 100)
        else:
            edges = cv2.Canny(big, 50, 150)
            kernel = np.ones((5,5), np.uint8)
            edges = cv2.morphologyEx(edges, cv2.MORPH_CLOSE, kernel)
        
        # Encontrar contornos
        contours, _ = cv2.findContours(edges, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        
        for cnt in contours:
            x, y, cw, ch = cv2.boundingRect(cnt)
            aspect = cw / ch if ch > 0 else 0
            
            if 2.0 < aspect < 7.0 and cw > 80 and ch > 20:
                # Extraer ROI
                pad = 10
                x1, y1 = max(0, x-pad), max(0, y-pad)
                x2, y2 = min(big.shape[1], x+cw+pad), min(big.shape[0], y+ch+pad)
                roi = big[y1:y2, x1:x2]
                
                # Aumentar aún más resolución
                roi_big = cv2.resize(roi, None, fx=2, fy=2, interpolation=cv2.INTER_CUBIC)
                
                results.append({
                    'roi': roi_big,
                    'bbox': (x//4, y//4, cw//4, ch//4),
                    'method': method,
                })
    
    return results


def ocr_with_tesseract(image, psm=7):
    """OCR con Tesseract."""
    env = os.environ.copy()
    env['TESSDATA_PREFIX'] = '/usr/share/tesseract-ocr/5/tessdata'
    
    # Guardar temporal
    temp_path = OUTPUT_DIR / '_temp_ocr.jpg'
    cv2.imwrite(str(temp_path), image)
    
    try:
        result = subprocess.run(
            ['/usr/bin/tesseract', str(temp_path), 'stdout',
             '--psm', str(psm),
             '-l', 'spa+eng',
             '-c', 'tessedit_char_whitelist=ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789- ',
             '--oem', '3'],
            capture_output=True, text=True, timeout=10, env=env
        )
        return result.stdout.strip()
    except Exception as e:
        return f"OCR_ERROR: {e}"


def main():
    print("╔══════════════════════════════════════════════════╗")
    print("║   NEXUS — Chapa Deblur + Super Resolution       ║")
    print("║   Motion deblur · Lucy-Richardson · Wiener      ║")
    print("╚══════════════════════════════════════════════════╝")
    
    # ─── 1. Extraer frames densos en zona crítica ──────────
    # La chapa se menciona al final (02:38-02:43)
    # También la cara del conductor se ve en frames 3 y 5
    print("\n[1] Extrayendo frames densos en zona del vehículo...")
    
    # Zona donde la chapa podría ser visible (todo el video, cada 0.5s en últimos 30s)
    intervals = [
        (130, 164),  # últimos 34 segundos (alta probabilidad de chapa)
        (0, 10),      # inicio
        (60, 70),     # mitad
    ]
    
    frames = extract_frames_dense(VIDEO_PATH, intervals, 'deblur')
    print(f"    {len(frames)} frames extraídos en zonas críticas")
    
    # ─── 2. Procesar cada frame ────────────────────────────
    print("\n[2] Aplicando deblurring + detección de chapa...")
    
    all_detections = []
    
    for timestamp, frame_path in frames:
        img = cv2.imread(str(frame_path))
        if img is None:
            continue
        
        h, w = img.shape[:2]
        
        # 2a. Detectar regiones de chapa
        plate_regions = detect_license_plate_region(img)
        
        if plate_regions:
            print(f"\n[🔍] Frame t={timestamp}s: {len(plate_regions)} regiones candidatas")
            
            for i, region in enumerate(plate_regions):
                roi = region['roi']
                bbox = region['bbox']
                
                # Guardar ROI
                roi_path = OUTPUT_DIR / f"roi_{timestamp:03d}_{i}_{region['method']}.jpg"
                cv2.imwrite(str(roi_path), roi)
                
                # OCR directo
                text = ocr_with_tesseract(roi, psm=7)
                
                # Verificar si es chapa
                found_chapa = None
                for pattern, desc in CHAPA_PATTERNS:
                    match = re.search(pattern, text.upper())
                    if match:
                        found_chapa = match.group()
                        break
                
                if found_chapa:
                    print(f"    🚗 CHAPA ENCONTRADA: {found_chapa} ({desc})")
                    print(f"       Frame: t={timestamp}s, ROI: {bbox}")
                    all_detections.append({
                        'chapa': found_chapa,
                        'timestamp': timestamp,
                        'frame': frame_path.name,
                        'roi': str(roi_path),
                        'bbox': bbox,
                        'text': text,
                    })
                else:
                    clean = re.sub(r'[^A-Z0-9]', '', text.upper())
                    if 5 <= len(clean) <= 8 and text:
                        print(f"    ⚠️ Posible: '{text}' (clean: {clean})")
        
        # 2b. Deblurring de frame completo
        print(f"\n[📷] Frame t={timestamp}s: Deblurring ({w}x{h})")
        
        # Motion deblur (Wiener)
        deblurred_results = motion_deblur(img, kernel_size=15)
        
        for angle, deblurred in deblurred_results:
            # OCR en frame deblurreado (región central inferior - donde suele estar la chapa)
            h_db, w_db = deblurred.shape
            roi_bottom = deblurred[h_db//2:h_db, :]
            
            # Redimensionar para mejor OCR
            roi_big = cv2.resize(roi_bottom, None, fx=2, fy=2, interpolation=cv2.INTER_CUBIC)
            
            text = ocr_with_tesseract(roi_big, psm=7)
            if text:
                clean = re.sub(r'[^A-Z0-9]', '', text.upper())
                for pattern, desc in CHAPA_PATTERNS:
                    match = re.search(pattern, text.upper())
                    if match:
                        print(f"    🚗 CHAPA (deblur {angle}°): {match.group()}")
                        all_detections.append({
                            'chapa': match.group(),
                            'timestamp': timestamp,
                            'method': f'wiener_{angle}',
                            'text': text,
                        })
        
        # Lucy-Richardson deconvolution
        lr_result = lucy_richardson_deconvolution(img, kernel_size=15, iterations=10)
        lr_path = OUTPUT_DIR / f"lr_{timestamp:03d}.jpg"
        cv2.imwrite(str(lr_path), lr_result)
        
        text = ocr_with_tesseract(lr_result, psm=7)
        if text:
            for pattern, desc in CHAPA_PATTERNS:
                match = re.search(pattern, text.upper())
                if match:
                    print(f"    🚗 CHAPA (LR): {match.group()}")
                    all_detections.append({
                        'chapa': match.group(),
                        'timestamp': timestamp,
                        'method': 'lucy_richardson',
                        'text': text,
                    })
    
    # ─── 3. REPORTE FINAL ──────────────────────────────────
    print(f"\n{'='*60}")
    print("📊 REPORTE FINAL DE CHAPA")
    print(f"{'='*60}")
    
    if all_detections:
        # Agrupar por chapa
        chapa_groups = {}
        for d in all_detections:
            chapa = d['chapa']
            if chapa not in chapa_groups:
                chapa_groups[chapa] = []
            chapa_groups[chapa].append(d)
        
        print(f"\n🚗 CHAPAS DETECTADAS:")
        for chapa, detections in chapa_groups.items():
            print(f"\n  [{chapa}] — {len(detections)} detecciones")
            for d in detections[:3]:
                print(f"     ├─ Frame: {d.get('timestamp','?')}s")
                print(f"     └─ Método: {d.get('method','direct')}")
                if d.get('roi'):
                    print(f"        ROI: {d['roi']}")
    else:
        print("\n⚠️  No se detectaron chapas válidas.")
        print("   Posibles causas:")
        print("   1. El video no muestra la chapa con suficiente claridad")
        print("   2. La chapa se muestra en un frame no capturado")
        print("   3. El OCR no puede resolver el texto por calidad extrema")
        print("\n   Recomendación: Usar Grok API para analizar frames por visión")
    
    # Guardar reporte
    report = {
        'detections': all_detections,
        'total_detections': len(all_detections),
        'output_dir': str(OUTPUT_DIR),
    }
    
    report_path = OUTPUT_DIR / 'chapa_deblur_report.json'
    with open(report_path, 'w') as f:
        json.dump(report, f, indent=2)
    
    print(f"\n[💾] Reporte guardado: {report_path}")
    return all_detections


if __name__ == "__main__":
    results = main()
