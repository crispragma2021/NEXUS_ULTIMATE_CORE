#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════╗
║          🧬 NEXUS FACIAL RECOGNITION PIPELINE v1.0          ║
║  Pipeline de reconocimiento facial soberano para NEXUS       ║
║  Powered by face_recognition 1.2.3 + dlib                   ║
║                                                              ║
║  Uso: python3 nexus_facial_pipeline.py [COMANDO] [ARGS]    ║
║                                                              ║
║  Comandos:                                                   ║
║    encode <path>                   Extraer encoding de foto  ║
║    extract <video.mp4>             Extraer rostros de video ║
║    match <carpeta_ref> <carpeta_target>  Comparar caras     ║
║    search <carpeta_ref> <video.mp4>     Extraer y comparar  ║
║    gallery <path>                  Indexar galería completa  ║
╚══════════════════════════════════════════════════════════════╝
"""

import os, sys, json, time, argparse, shutil
from pathlib import Path
from datetime import datetime

import cv2
import numpy as np

try:
    import face_recognition
except ImportError:
    print("❌ face_recognition no instalado. Ejecuta: pip3 install face_recognition")
    sys.exit(1)

# ─── RUTAS SOBERANAS ─────────────────────────────────────────
BASE_DIR = Path(__file__).parent.resolve()
GALERIA_DIR = BASE_DIR / "galeria"
ROSTROS_DIR = BASE_DIR / "rostros_extraidos"
REPORTS_DIR = BASE_DIR / "reports"
CACHE_DIR = BASE_DIR / ".cache_encodings"

# ─── CONFIGURACIÓN ───────────────────────────────────────────
CONFIG = {
    "upsample_times": 1,           # Veces para upsampling en detección
    "num_jitters": 10,             # Jitters para encoding (más = preciso, lento)
    "model": "hog",                # "hog" (rápido/CPU) o "cnn" (preciso/GPU)
    "threshold_distance": 0.5,     # Máxima distancia para match (0.0-1.0, menor = estricto)
    "threshold_percent": 0.65,     # Mínimo % de similitud (0-1)
    "min_face_size": 80,           # Tamaño mínimo de rostro en px
    "max_faces_per_image": 10,     # Límite de rostros por imagen
}


# ═══════════════════════════════════════════════════════════════
#  NÚCLEO DEL PIPELINE
# ═══════════════════════════════════════════════════════════════

def ensure_dirs():
    """Crea todos los directorios necesarios si no existen."""
    for d in [GALERIA_DIR, ROSTROS_DIR, REPORTS_DIR, CACHE_DIR]:
        d.mkdir(parents=True, exist_ok=True)


def cargar_imagen(ruta):
    """
    Carga una imagen desde ruta (archivo o directorio con 1 imagen).
    Retorna: (ruta_str, imagen_array) o (None, None) si falla.
    """
    ruta = Path(ruta)
    if not ruta.exists():
        print(f"  ❌ Ruta no existe: {ruta}")
        return None, None

    if ruta.is_dir():
        # Busca la primera imagen en el directorio
        imagenes = list(ruta.glob("*")) + list(ruta.glob("*.[pP][nN][gG]")) + \
                   list(ruta.glob("*.[jJ][pP][gG]")) + list(ruta.glob("*.[jJ][pP][eE][gG]"))
        # Filtro seguro
        imagenes_validas = [f for f in ruta.iterdir() if f.suffix.lower() in ('.jpg','.jpeg','.png','.bmp','.tiff')]
        if not imagenes_validas:
            print(f"  ❌ No hay imágenes en: {ruta}")
            return None, None
        ruta = imagenes_validas[0]

    img = face_recognition.load_image_file(str(ruta))
    print(f"  📷 Cargada: {ruta.name} ({img.shape[1]}×{img.shape[0]})")
    return str(ruta), img


def extraer_rostros(imagen_path, upsample=1, model="hog"):
    """
    Extrae ubicaciones y encodings de rostros de una imagen.
    Retorna: [(location, encoding), ...]
    """
    img = face_recognition.load_image_file(imagen_path)
    locations = face_recognition.face_locations(img, number_of_times_to_upsample=upsample, model=model)

    if not locations:
        print(f"  ⚠️  No se detectaron rostros en: {Path(imagen_path).name}")
        return []

    # Limitar cantidad de rostros
    if len(locations) > CONFIG["max_faces_per_image"]:
        print(f"  ⚠️  {len(locations)} rostros detectados, limitando a {CONFIG['max_faces_per_image']}")
        locations = locations[:CONFIG["max_faces_per_image"]]

    encodings = face_recognition.face_encodings(img, locations, num_jitters=CONFIG["num_jitters"])
    print(f"  🎯 {len(encodings)} rostro(s) detectado(s) en: {Path(imagen_path).name}")

    result = []
    for i, (loc, enc) in enumerate(zip(locations, encodings)):
        top, right, bottom, left = loc
        w, h = right - left, bottom - top
        result.append({
            "index": i,
            "location": loc,
            "encoding": enc,
            "size": (w, h),
            "area": w * h,
        })
    return result


def calcular_similitud(encoding_a, encoding_b):
    """
    Calcula similitud entre dos encodings.
    Retorna: (distancia_euclidiana, porcentaje_similitud)
    """
    distancia = face_recognition.face_distance([encoding_a], encoding_b)[0]
    # Convertir distancia a porcentaje de similitud (0.0 = idéntico = 100%)
    similitud = max(0, (1.0 - distancia / CONFIG["threshold_distance"]) * 100)
    return float(distancia), float(similitud)


# ═══════════════════════════════════════════════════════════════
#  EXTRACCIÓN DE ROSTROS DESDE VIDEO
# ═══════════════════════════════════════════════════════════════

def extraer_rostros_de_video(video_path, output_dir=None, intervalo_seg=2, confianza_min=0.6):
    """
    Extrae rostros de un video usando detección facial.
    
    Args:
        video_path: Ruta al archivo de video
        output_dir: Directorio de salida (default: ROSTROS_DIR / video_name)
        intervalo_seg: Intervalo entre frames procesados (default: 2s)
        confianza_min: Confianza mínima (face_recognition no usa confianza, es filtro área)
    
    Retorna: { video_info, rostros: [{timestamp, frame, archivo, location, size}, ...] }
    """
    video_path = Path(video_path)
    if not video_path.exists():
        print(f"❌ Video no encontrado: {video_path}")
        return None

    output_dir = Path(output_dir or ROSTROS_DIR / video_path.stem)
    output_dir.mkdir(parents=True, exist_ok=True)

    cap = cv2.VideoCapture(str(video_path))
    fps = cap.get(cv2.CAP_PROP_FPS)
    total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    duracion_seg = total_frames / fps if fps > 0 else 0
    frame_w = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    frame_h = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))

    print(f"\n🎥 Video: {video_path.name}")
    print(f"   Resolución: {frame_w}×{frame_h} | FPS: {fps:.1f} | Duración: {duracion_seg:.1f}s")
    print(f"   Procesando cada {intervalo_seg}s (aprox {max(1, int(duracion_seg/intervalo_seg))} frames)...")

    rostros_encontrados = []
    frame_count = 0
    rostro_count = 0
    skip_frames = max(1, int(fps * intervalo_seg))

    while True:
        ret, frame = cap.read()
        if not ret:
            break

        if frame_count % skip_frames != 0:
            frame_count += 1
            continue

        timestamp = frame_count / fps
        # Convertir BGR a RGB
        rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)

        # Detectar rostros
        locations = face_recognition.face_locations(
            rgb_frame,
            number_of_times_to_upsample=CONFIG["upsample_times"],
            model=CONFIG["model"]
        )

        for i, (top, right, bottom, left) in enumerate(locations):
            w, h = right - left, bottom - top
            area = w * h
            # Filtrar rostros muy pequeños
            if w < CONFIG["min_face_size"] or h < CONFIG["min_face_size"]:
                continue

            # Extraer el recorte del rostro
            face_crop = frame[top:bottom, left:right]
            if face_crop.size == 0:
                continue

            nombre_archivo = f"face_frame{frame_count:06d}_t{timestamp:05.1f}s_{i}.jpg"
            ruta_archivo = output_dir / nombre_archivo
            cv2.imwrite(str(ruta_archivo), face_crop, [cv2.IMWRITE_JPEG_QUALITY, 95])
            tam_kb = ruta_archivo.stat().st_size / 1024

            rostros_encontrados.append({
                "index": rostro_count,
                "timestamp": round(timestamp, 1),
                "frame": frame_count,
                "archivo": str(ruta_archivo),
                "archivo_rel": str(ruta_archivo.relative_to(BASE_DIR)),
                "location": [top, right, bottom, left],
                "size": {"w": w, "h": h, "area": area},
                "tam_kb": round(tam_kb, 1),
            })
            rostro_count += 1

        # Progreso cada 10%
        progress = frame_count / total_frames * 100
        if int(progress) % 10 == 0 and progress > 0:
            print(f"   ⏳ Progreso: {progress:.0f}% | Rostros: {rostro_count}", end="\r", flush=True)

        frame_count += 1

    cap.release()
    print(f"\n   ✅ Completo: {total_frames} frames, {rostro_count} rostros extraídos")

    reporte = {
        "video": {
            "archivo": str(video_path),
            "nombre": video_path.name,
            "resolucion": f"{frame_w}×{frame_h}",
            "fps": round(fps, 1),
            "duracion_seg": round(duracion_seg, 1),
            "total_frames": total_frames,
        },
        "config": {
            "intervalo_seg": intervalo_seg,
            "min_face_size": CONFIG["min_face_size"],
            "model": CONFIG["model"],
            "upsample": CONFIG["upsample_times"],
        },
        "rostros": rostros_encontrados,
        "total_rostros": rostro_count,
        "timestamp": datetime.now().isoformat(),
    }

    return reporte


# ═══════════════════════════════════════════════════════════════
#  COMPARACIÓN DE ROSTROS
# ═══════════════════════════════════════════════════════════════

def indexar_galeria(galeria_path):
    """
    Indexa todos los rostros en un directorio de galería.
    Cada subdirectorio es un sujeto.
    
    Estructura esperada:
    galeria/
        Aldo_Francisco_Coronel/
            foto1.jpg
            foto2.jpg
        Mauricio_Canete/
            foto1.jpg
        ...
    
    Retorna: [{sujeto, archivos: [{path, encoding, size}, ...]}, ...]
    """
    galeria_path = Path(galeria_path)
    if not galeria_path.exists():
        print(f"❌ Galería no encontrada: {galeria_path}")
        return []

    # También acepta archivos sueltos en el directorio raíz
    sujetos = []

    # Si hay subdirectorios, cada uno es un sujeto
    subdirs = [d for d in galeria_path.iterdir() if d.is_dir()]
    
    if subdirs:
        for sujeto_dir in subdirs:
            sujeto = sujeto_dir.name.replace("_", " ").title()
            archivos = list(sujeto_dir.glob("*.[jJ][pP][gG]")) + \
                       list(sujeto_dir.glob("*.[jJ][pP][eE][gG]")) + \
                       list(sujeto_dir.glob("*.[pP][nN][gG]"))
            if not archivos:
                continue
            print(f"\n📁 Sujeto: {sujeto} ({len(archivos)} fotos)")
            encodings_sujeto = []
            for archivo in archivos:
                rostros = extraer_rostros(str(archivo))
                if rostros:
                    encodings_sujeto.append({
                        "path": str(archivo),
                        "encoding": rostros[0]["encoding"],  # Tomar el primer rostro
                        "size": rostros[0]["size"],
                    })
                    print(f"      ✅ {archivo.name} → encoding OK")
                else:
                    print(f"      ⚠️  {archivo.name} → sin rostro detectable")
            if encodings_sujeto:
                sujetos.append({
                    "sujeto": sujeto,
                    "nombre_dir": sujeto_dir.name,
                    "encodings": encodings_sujeto,
                })
    else:
        # Archivos sueltos en el directorio raíz — cada archivo es un sujeto
        archivos = list(galeria_path.glob("*.[jJ][pP][gG]")) + \
                   list(galeria_path.glob("*.[jJ][pP][eE][gG]")) + \
                   list(galeria_path.glob("*.[pP][nN][gG]"))
        for archivo in archivos:
            sujeto = archivo.stem.replace("_", " ").title()
            print(f"\n📄 Archivo: {archivo.name}")
            rostros = extraer_rostros(str(archivo))
            if rostros:
                sujetos.append({
                    "sujeto": sujeto,
                    "nombre_dir": archivo.stem,
                    "encodings": [{
                        "path": str(archivo),
                        "encoding": rostros[0]["encoding"],
                        "size": rostros[0]["size"],
                    }],
                })
                print(f"      ✅ encoding OK")
            else:
                print(f"      ⚠️  sin rostro detectable")

    return sujetos


def comparar_rostros(rostros_target, galeria_indexada, threshold=None):
    """
    Compara una lista de rostros target contra una galería indexada.
    
    Args:
        rostros_target: Lista de dicts con key "encoding" (array numpy)
        galeria_indexada: Lista de sujetos indexados
        threshold: Distancia máxima para considerar match
    
    Retorna: [match, ...] con scores
    """
    threshold = threshold or CONFIG["threshold_distance"]
    resultados = []

    for target in rostros_target:
        encoding_target = target.get("encoding")
        if encoding_target is None:
            continue

        mejor_match = None
        mejor_distancia = float("inf")

        for sujeto in galeria_indexada:
            for ref in sujeto["encodings"]:
                try:
                    distancia = face_recognition.face_distance([ref["encoding"]], encoding_target)[0]
                    if distancia < mejor_distancia:
                        mejor_distancia = float(distancia)
                        mejor_match = {
                            "sujeto": sujeto["sujeto"],
                            "archivo_ref": ref["path"],
                            "distancia": mejor_distancia,
                            "similitud": max(0, (1.0 - mejor_distancia / threshold) * 100),
                        }
                except Exception as e:
                    continue

        resultados.append({
            "target": target.get("archivo", target.get("path", "desconocido")),
            "mejor_match": mejor_match,
            "match_valido": mejor_match and mejor_match["distancia"] <= threshold,
        })

    return resultados


def comparar_carpetas(carpeta_ref, carpeta_target, output_json=None):
    """
    Compara todos los rostros en carpeta_target contra carpeta_ref.
    """
    print(f"\n{'='*60}")
    print(f"🔍 COMPARACIÓN FACIAL")
    print(f"{'='*60}")
    print(f"Referencia: {carpeta_ref}")
    print(f"Targets:    {carpeta_target}")

    # Indexar galería de referencia
    print(f"\n📚 Indexando galería de referencia...")
    galeria = indexar_galeria(carpeta_ref)
    if not galeria:
        print("❌ No se pudo indexar ninguna referencia")
        return None

    # Cargar rostros target
    print(f"\n🎯 Cargando rostros target...")
    targets = []
    target_dir = Path(carpeta_target)
    archivos_target = list(target_dir.glob("*.[jJ][pP][gG]")) + \
                      list(target_dir.glob("*.[jJ][pP][eE][gG]")) + \
                      list(target_dir.glob("*.[pP][nN][gG]"))

    for archivo in archivos_target:
        rostros = extraer_rostros(str(archivo))
        for r in rostros:
            r["archivo"] = str(archivo)
            r["path"] = str(archivo)
            targets.append(r)

    if not targets:
        print("❌ No se detectaron rostros en los targets")
        return None

    print(f"\n⚡ Comparando {len(targets)} rostros contra {sum(len(s['encodings']) for s in galeria)} referencias...")
    resultados = comparar_rostros(targets, galeria)

    # Compilar reporte
    matches_validos = [r for r in resultados if r["match_valido"]]
    reporte = {
        "config": {
            "threshold_distancia": CONFIG["threshold_distance"],
            "model": CONFIG["model"],
            "num_jitters": CONFIG["num_jitters"],
            "upsample_times": CONFIG["upsample_times"],
        },
        "galeria": [{"sujeto": s["sujeto"], "fotos": len(s["encodings"])} for s in galeria],
        "resultados": [],
        "total_targets": len(targets),
        "total_matches": len(matches_validos),
        "timestamp": datetime.now().isoformat(),
    }

    for r in resultados:
        entry = {
            "target": r["target"],
            "match_valido": r["match_valido"],
        }
        if r["mejor_match"]:
            entry["match"] = {
                "sujeto": r["mejor_match"]["sujeto"],
                "archivo_ref": r["mejor_match"]["archivo_ref"],
                "distancia": round(r["mejor_match"]["distancia"], 4),
                "similitud": round(r["mejor_match"]["similitud"], 2),
            }
        else:
            entry["match"] = None
        reporte["resultados"].append(entry)

    # Mostrar resultados
    print(f"\n{'─'*60}")
    if matches_validos:
        print(f"✅ {len(matches_validos)}/{len(targets)} MATCHES ENCONTRADOS:")
        for r in matches_validos:
            m = r["mejor_match"]
            archivo = Path(r["target"]).name
            simbolo = "🟢" if m["similitud"] >= 75 else "🟡" if m["similitud"] >= 50 else "🟠"
            print(f"   {simbolo} {archivo} → {m['sujeto']} (similitud: {m['similitud']:.1f}%, distancia: {m['distancia']:.3f})")
            print(f"       ref: {Path(m['archivo_ref']).name}")
    else:
        print(f"❌ NO HAY MATCHES dentro del threshold ({CONFIG['threshold_distance']})")

    # Guardar reporte
    if output_json:
        output_path = Path(output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        # Convertir encodings a string para JSON (se removieron)
        reporte_export = json.loads(json.dumps(reporte, default=lambda o: "<<encoding>>" if isinstance(o, np.ndarray) else str(o)))
        with open(output_path, "w") as f:
            json.dump(reporte_export, f, indent=2, ensure_ascii=False)
        print(f"\n💾 Reporte guardado: {output_path}")

    return reporte


# ═══════════════════════════════════════════════════════════════
#  INTERFAZ CLI
# ═══════════════════════════════════════════════════════════════

def cmd_encode(args):
    """Extrae encoding de una imagen y lo muestra."""
    ruta, img = cargar_imagen(args.path)
    if ruta is None:
        return
    
    rostros = extraer_rostros(ruta)
    if not rostros:
        print("  ❌ No se detectaron rostros")
        return

    for r in rostros:
        loc = r["location"]
        print(f"\n  🎯 Rostro #{r['index']}: ({loc[3]},{loc[0]})-({loc[1]},{loc[2]}) | {r['size'][0]}×{r['size'][1]}px")
        print(f"     Encoding vector: {r['encoding'].shape} [{r['encoding'][:5]}...]")
        print(f"     Norma L2: {np.linalg.norm(r['encoding']):.6f}")


def cmd_extract(args):
    """Extrae rostros de un video."""
    reporte = extraer_rostros_de_video(
        args.video,
        output_dir=args.output,
        intervalo_seg=args.intervalo,
    )
    if reporte is None:
        return

    output_json = args.output_json or str(REPORTS_DIR / f"rostros_{Path(args.video).stem}_{int(time.time())}.json")
    
    # Guardar reporte JSON
    output_path = Path(output_json)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(reporte, f, indent=2, ensure_ascii=False)
    
    print(f"\n💾 Reporte: {output_path}")
    print(f"📸 Rostros extraídos: {reporte['total_rostros']}")
    print(f"   Dir: {Path(reporte['rostros'][0]['archivo']).parent if reporte['rostros'] else 'N/A'}")


def cmd_match(args):
    """Compara dos carpetas de rostros."""
    timestamp = int(time.time())
    output_json = args.output_json or str(REPORTS_DIR / f"match_{timestamp}.json")
    comparar_carpetas(args.referencia, args.target, output_json=output_json)


def cmd_search(args):
    """
    Extrae rostros de un video y los compara contra una galería.
    Pipeline completo: extract → encode → match.
    """
    print(f"\n{'='*60}")
    print(f"🔍 PIPELINE COMPLETO: Extraer + Comparar")
    print(f"{'='*60}")
    print(f"Referencia: {args.referencia}")
    print(f"Video:      {args.video}")

    # 1. Extraer rostros del video
    output_dir = ROSTROS_DIR / f"vid_{Path(args.video).stem}"
    reporte_video = extraer_rostros_de_video(
        args.video,
        output_dir=output_dir,
        intervalo_seg=args.intervalo,
    )
    if reporte_video is None or reporte_video["total_rostros"] == 0:
        print("❌ No se extrajeron rostros del video")
        return

    # 2. Indexar galería
    galeria = indexar_galeria(args.referencia)
    if not galeria:
        print("❌ No se pudo indexar la galería de referencia")
        return

    # 3. Cargar encodings de rostros extraídos
    targets = []
    for rostro in reporte_video["rostros"]:
        ruta = rostro["archivo"]
        rst = extraer_rostros(ruta)
        for r in rst:
            r["archivo"] = ruta
            r["path"] = ruta
            r["timestamp"] = rostro["timestamp"]
            r["frame"] = rostro["frame"]
            targets.append(r)

    if not targets:
        print("❌ No se pudieron generar encodings de los rostros extraídos")
        return

    # 4. Comparar
    print(f"\n⚡ Comparando {len(targets)} rostros extraídos contra galería...")
    resultados = comparar_rostros(targets, galeria)

    # 5. Generar reporte consolidado
    matches_validos = [r for r in resultados if r["match_valido"]]
    
    reporte_final = {
        "pipeline": "nexus_facial_pipeline search",
        "config": CONFIG,
        "referencia": str(args.referencia),
        "video": reporte_video["video"],
        "galeria": [{"sujeto": s["sujeto"], "fotos": len(s["encodings"])} for s in galeria],
        "resultados": [],
        "total_extraidos": len(targets),
        "total_matches": len(matches_validos),
        "timestamp": datetime.now().isoformat(),
    }

    for i, r in enumerate(resultados):
        entry = {
            "target_index": i,
            "target_archivo": r["target"],
            "match_valido": r["match_valido"],
        }
        # Buscar timestamp/frame del target original
        for t in targets:
            if t.get("archivo") == r["target"]:
                entry["timestamp"] = t.get("timestamp")
                entry["frame"] = t.get("frame")
                break
        
        if r["mejor_match"]:
            entry["match"] = {
                "sujeto": r["mejor_match"]["sujeto"],
                "archivo_ref": r["mejor_match"]["archivo_ref"],
                "distancia": round(r["mejor_match"]["distancia"], 4),
                "similitud": round(r["mejor_match"]["similitud"], 2),
            }
        else:
            entry["match"] = None
        
        reporte_final["resultados"].append(entry)

    # Mostrar resultados
    print(f"\n{'─'*60}")
    print(f"RESULTADOS DEL PIPELINE COMPLETO:")
    print(f"{'─'*60}")
    print(f"Rostros extraídos del video: {len(targets)}")
    
    if matches_validos:
        print(f"✅ MATCHES ENCONTRADOS: {len(matches_validos)}")
        for r in matches_validos:
            m = r["mejor_match"]
            archivo = Path(r["target"]).name
            timestamp_str = ""
            for t in targets:
                if t.get("archivo") == r["target"]:
                    timestamp_str = f" [t={t.get('timestamp','?')}s]"
                    break
            simbolo = "🟢" if m["similitud"] >= 75 else "🟡" if m["similitud"] >= 50 else "🟠"
            print(f"   {simbolo} {archivo}{timestamp_str} → {m['sujeto']} ({m['similitud']:.1f}%)")
    else:
        print(f"❌ SIN MATCHES dentro del threshold ({CONFIG['threshold_distance']})")
        # Mostrar el mejor match aunque no pase threshold
        mejores = sorted(resultados, key=lambda x: x["mejor_match"]["distancia"] if x.get("mejor_match") else 999)
        if mejores and mejores[0].get("mejor_match"):
            m = mejores[0]["mejor_match"]
            print(f"   Mejor candidato: {m['sujeto']} ({m['similitud']:.1f}% similitud, distancia {m['distancia']:.3f} > threshold {CONFIG['threshold_distance']})")

    # Guardar
    timestamp = int(time.time())
    output_json = args.output_json or str(REPORTS_DIR / f"pipeline_completo_{timestamp}.json")
    output_path = Path(output_json)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Remover encodings numpy del JSON
    reporte_export = json.loads(json.dumps(reporte_final, default=lambda o: "<<encoding>>" if isinstance(o, np.ndarray) else str(o)))
    with open(output_path, "w") as f:
        json.dump(reporte_export, f, indent=2, ensure_ascii=False)
    
    print(f"\n💾 Reporte completo: {output_path}")


def cmd_gallery(args):
    """Indexa una galería completa y guarda cache de encodings."""
    print(f"\n{'='*60}")
    print(f"📚 INDEXAR GALERÍA")
    print(f"{'='*60}")
    
    galeria = indexar_galeria(args.path)
    if not galeria:
        print("❌ No se indexó ningún sujeto")
        return

    # Guardar cache de encodings (serializado)
    cache_path = CACHE_DIR / f"galeria_{Path(args.path).stem}_{int(time.time())}.npz"
    
    # También guardar un JSON descriptivo (sin encodings)
    reporte = {
        "path": str(args.path),
        "sujetos": [],
        "total_sujetos": len(galeria),
        "timestamp": datetime.now().isoformat(),
    }
    for g in galeria:
        reporte["sujetos"].append({
            "sujeto": g["sujeto"],
            "fotos": len(g["encodings"]),
            "archivos": [e["path"] for e in g["encodings"]],
        })
    
    # Guardar JSON
    json_path = REPORTS_DIR / f"galeria_index_{int(time.time())}.json"
    json_path.parent.mkdir(parents=True, exist_ok=True)
    with open(json_path, "w") as f:
        json.dump(reporte, f, indent=2, ensure_ascii=False)

    print(f"\n✅ Galería indexada: {len(galeria)} sujetos")
    for g in galeria:
        print(f"   • {g['sujeto']}: {len(g['encodings'])} foto(s)")
    print(f"💾 Índice: {json_path}")
    print(f"💾 Cache: {cache_path}")


def main():
    parser = argparse.ArgumentParser(
        description="🧬 NEXUS Facial Recognition Pipeline",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Ejemplos:
  %(prog)s encode foto_referencia.jpg
  %(prog)s extract video.mp4 --intervalo 1
  %(prog)s match galeria/ rostros_extraidos/
  %(prog)s search galeria/ video.mp4 --intervalo 2
  %(prog)s gallery galeria/
        """
    )
    subparsers = parser.add_subparsers(dest="comando", help="Comando a ejecutar")

    # ── encode ──
    p_encode = subparsers.add_parser("encode", help="Extraer encoding de una imagen")
    p_encode.add_argument("path", help="Ruta a la imagen")
    p_encode.set_defaults(func=cmd_encode)

    # ── extract ──
    p_extract = subparsers.add_parser("extract", help="Extraer rostros de un video")
    p_extract.add_argument("video", help="Ruta al archivo de video")
    p_extract.add_argument("--output", "-o", help="Directorio de salida para rostros")
    p_extract.add_argument("--output-json", "-j", help="Ruta del reporte JSON")
    p_extract.add_argument("--intervalo", "-i", type=float, default=2.0, help="Intervalo entre frames (segundos, default: 2)")
    p_extract.set_defaults(func=cmd_extract)

    # ── match ──
    p_match = subparsers.add_parser("match", help="Comparar rostros de dos carpetas")
    p_match.add_argument("referencia", help="Carpeta con fotos de referencia")
    p_match.add_argument("target", help="Carpeta con rostros a comparar")
    p_match.add_argument("--output-json", "-j", help="Ruta del reporte JSON")
    p_match.set_defaults(func=cmd_match)

    # ── search ──
    p_search = subparsers.add_parser("search", help="Pipeline completo: extraer rostros de video y comparar contra galería")
    p_search.add_argument("referencia", help="Carpeta con fotos de referencia (galería)")
    p_search.add_argument("video", help="Ruta al archivo de video")
    p_search.add_argument("--intervalo", "-i", type=float, default=2.0, help="Intervalo entre frames (segundos, default: 2)")
    p_search.add_argument("--output-json", "-j", help="Ruta del reporte JSON")
    p_search.set_defaults(func=cmd_search)

    # ── gallery ──
    p_gallery = subparsers.add_parser("gallery", help="Indexar galería completa")
    p_gallery.add_argument("path", help="Ruta a la carpeta de galería")
    p_gallery.set_defaults(func=cmd_gallery)

    args = parser.parse_args()
    if not args.comando:
        parser.print_help()
        return

    ensure_dirs()
    args.func(args)


if __name__ == "__main__":
    main()
