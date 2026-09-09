#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════╗
║    👁️ NEXUS VISION GROK v1.0                                    ║
║    Análisis visual de frames/imágenes vía OpenRouter+IA          ║
║    Modelos disponibles: xai/grok-vision, google/gemini-pro-vision║
║    openai/gpt-4o, anthropic/claude-3-opus, etc.                  ║
║                                                                  ║
║  Uso: python3 nexus_vision_grok.py [COMANDO] [ARGS]              ║
║                                                                  ║
║  Comandos:                                                       ║
║    analyze <imagen> [--prompt TEXTO]     Analizar imagen única    ║
║    batch <directorio> [--prompt TEXTO]   Analizar directorio     ║
║    chapa <imagen>                        Detectar chapa/patente  ║
║    rostro <imagen>                       Describir rostro        ║
║    scene <imagen>                        Describir escena        ║
║    text <imagen>                         Extraer texto borroso   ║
║                                                                  ║
║  Prompt por defecto para frames de investigación:                ║
║    "Analiza esta imagen en detalle. Describe cualquier texto,    ║
║     persona, vehículo, patente, objeto o detalle relevante       ║
║     que puedas identificar."                                     ║
╚══════════════════════════════════════════════════════════════════╝
"""

import os, sys, json, time, base64, argparse, glob
from pathlib import Path
from datetime import datetime
import urllib.request
import urllib.error

# ─── RUTAS ──────────────────────────────────────────────────
BASE_DIR = Path(__file__).parent.resolve()
REPORTS_DIR = BASE_DIR / "reports"
IMAGES_DIR = BASE_DIR / "images_input"

# ─── CONFIG ─────────────────────────────────────────────────
CONFIG = {
    # Modelo por defecto: Grok Vision via OpenRouter
    "model": "xai/grok-vision-beta",
    "fallback_model": "openai/gpt-4o",
    "max_tokens": 4096,
    "temperature": 0.3,
    "max_image_size": 4 * 1024 * 1024,  # 4MB max para base64
    "max_images_per_batch": 20,          # Límite de imágenes por batch
    "timeout": 60,                        # Timeout API en segundos
}

# ─── PROMPTS PRE-ARMADOS ────────────────────────────────────
PROMPTS = {
    "default": (
        "Analiza esta imagen en detalle. Describe cualquier texto, persona, "
        "vehículo, patente, objeto o detalle relevante que puedas identificar. "
        "Responde en español. Sé específico y preciso."
    ),
    "chapa": (
        "Examina esta imagen con atención. Busca una patente/chapa/matrícula "
        "de vehículo. Si ves caracteres alfanuméricos, escríbelos EXACTAMENTE "
        "como aparecen. Si el texto está borroso, da tu mejor interpretación "
        "con nivel de confianza (alto/medio/bajo). Describe la posición de la "
        "chapa en el vehículo (delantera/trasera, color, formato). "
        "Responde en español."
    ),
    "rostro": (
        "Describe el rostro de la persona en esta imagen con detalles "
        "forenses: forma del rostro, color de ojos (si visible), nariz, "
        "cejas, barba/bigote, edad aproximada, marcas distintivas (cicatrices, "
        "lunares, tatuajes), peinado, expresión. Estima la probabilidad de que "
        "esta persona sea de Paraguay o región. Responde en español."
    ),
    "scene": (
        "Describe esta escena en detalle: ubicación aparente (interior/exterior, "
        "día/noche, urbano/rural), vehículos presentes (marca, modelo, color si "
        "identificable), personas, objetos, señales de tráfico, comercios, "
        "placas. Identifica cualquier elemento que pueda servir para geolocalizar. "
        "Responde en español."
    ),
    "text": (
        "Examina esta imagen buscando texto. Extrae cualquier texto visible "
        "aunque esté borroso, parcialmente oculto, o en baja resolución. Dame "
        "tu mejor interpretación para cada fragmento con nivel de confianza. "
        "Incluye texto en carteles, documentos, pantallas, patentes, etiquetas. "
        "Responde en español."
    ),
}


# ═══════════════════════════════════════════════════════════════
#  API OPENROUTER
# ═══════════════════════════════════════════════════════════════

def get_api_key():
    """Obtiene la API key de OpenRouter desde variables de entorno."""
    # Intentar desde el archivo .env en la raíz del proyecto
    env_path = BASE_DIR.parent / ".env"
    if env_path.exists():
        with open(env_path) as f:
            for line in f:
                line = line.strip()
                if line.startswith("OPENROUTER_API_KEY="):
                    key = line.split("=", 1)[1].strip().strip('"').strip("'")
                    if key:
                        return key

    # Fallback a variable de entorno
    key = os.environ.get("OPENROUTER_API_KEY")
    if key:
        return key

    print("❌ OPENROUTER_API_KEY no encontrada.")
    print("   Crea un archivo .env en la raíz con: OPENROUTER_API_KEY=sk-or-v1-...")
    return None


def encode_image(image_path, max_size=None):
    """
    Codifica una imagen a base64.
    Si max_size está definido, escala la imagen para no excederlo.
    """
    path = Path(image_path)
    if not path.exists():
        print(f"  ❌ Imagen no encontrada: {image_path}")
        return None

    size_kb = path.stat().st_size / 1024
    max_size = max_size or CONFIG["max_image_size"]

    # Si es muy grande, intentar reducir calidad via ImageMagick
    if path.stat().st_size > max_size:
        print(f"  ⚠️  Imagen muy grande ({size_kb:.0f}KB), redimensionando...")
        import subprocess
        temp_path = path.with_suffix(".tmp_resized.jpg")
        try:
            subprocess.run([
                "convert", str(path),
                "-resize", "50%",
                "-quality", "85",
                str(temp_path)
            ], check=True, capture_output=True, timeout=10)
            with open(temp_path, "rb") as f:
                encoded = base64.b64encode(f.read()).decode("utf-8")
            temp_path.unlink(missing_ok=True)
            print(f"  ✅ Redimensionada: {len(encoded)/1024:.0f}KB base64")
            return encoded
        except Exception as e:
            print(f"  ⚠️  No se pudo redimensionar: {e}")

    with open(image_path, "rb") as f:
        img_data = f.read()
    encoded = base64.b64encode(img_data).decode("utf-8")
    print(f"  📷 Codificada: {len(encoded)/1024:.0f}KB base64")
    return encoded


def query_openrouter_vision(image_base64, prompt, model=None, max_tokens=None, temperature=None):
    """
    Envía una imagen (base64) + prompt a OpenRouter con modelo de visión.
    Retorna: { success, text, model_used, tokens, error }
    """
    api_key = get_api_key()
    if not api_key:
        return {"success": False, "error": "No API key"}

    model = model or CONFIG["model"]
    max_tokens = max_tokens or CONFIG["max_tokens"]
    temperature = temperature or CONFIG["temperature"]

    # Formatear mensaje con imagen
    data = {
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": prompt},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": f"data:image/jpeg;base64,{image_base64}",
                            "detail": "high"
                        }
                    }
                ]
            }
        ],
        "max_tokens": max_tokens,
        "temperature": temperature,
    }

    payload = json.dumps(data).encode("utf-8")
    req = urllib.request.Request(
        "https://openrouter.ai/api/v1/chat/completions",
        data=payload,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            "HTTP-Referer": "https://nexus.sovereign",
            "X-Title": "NEXUS Vision Grok",
        },
        method="POST"
    )

    try:
        with urllib.request.urlopen(req, timeout=CONFIG["timeout"]) as resp:
            result = json.loads(resp.read().decode("utf-8"))

            if "error" in result:
                return {"success": False, "error": result["error"].get("message", str(result["error"]))}

            choice = result.get("choices", [{}])[0]
            text = choice.get("message", {}).get("content", "")

            usage = result.get("usage", {})
            return {
                "success": True,
                "text": text,
                "model_used": model,
                "prompt_tokens": usage.get("prompt_tokens", 0),
                "completion_tokens": usage.get("completion_tokens", 0),
                "total_tokens": usage.get("total_tokens", 0),
            }

    except urllib.error.HTTPError as e:
        error_body = e.read().decode("utf-8", errors="replace")[:500]
        return {"success": False, "error": f"HTTP {e.code}: {error_body}"}
    except urllib.error.URLError as e:
        return {"success": False, "error": f"URL Error: {e.reason}"}
    except Exception as e:
        return {"success": False, "error": str(e)}


def query_with_fallback(image_base64, prompt):
    """
    Intenta con el modelo principal, si falla con fallback.
    """
    # Intentar modelo principal
    result = query_openrouter_vision(image_base64, prompt, model=CONFIG["model"])
    
    if result["success"]:
        return result

    # Intentar fallback si el error es del modelo
    error_msg = result.get("error", "")
    if "model" in error_msg.lower() or "not found" in error_msg.lower() or "capacity" in error_msg.lower():
        print(f"  ⚠️  Modelo {CONFIG['model']} no disponible. Intentando fallback {CONFIG['fallback_model']}...")
        result = query_openrouter_vision(image_base64, prompt, model=CONFIG["fallback_model"])
        if result["success"]:
            return result

    return result


# ═══════════════════════════════════════════════════════════════
#  FUNCIONES DE ANÁLISIS
# ═══════════════════════════════════════════════════════════════

def analyze_image(image_path, prompt=None, prompt_type=None, save_report=True):
    """
    Analiza una imagen con IA visual.
    
    Args:
        image_path: Ruta a la imagen
        prompt: Prompt personalizado (opcional)
        prompt_type: Tipo de prompt predefinido (chapa, rostro, scene, text)
        save_report: Guardar reporte JSON
    
    Retorna: { success, text, model_used, tokens, error, archivo }
    """
    image_path = Path(image_path)
    if not image_path.exists():
        return {"success": False, "error": f"Archivo no encontrado: {image_path}"}

    # Determinar prompt
    if prompt:
        final_prompt = prompt
    elif prompt_type and prompt_type in PROMPTS:
        final_prompt = PROMPTS[prompt_type]
    else:
        final_prompt = PROMPTS["default"]

    print(f"\n{'─'*60}")
    print(f"👁️  Analizando: {image_path.name}")
    print(f"📝 Prompt: {final_prompt[:80]}...")
    print(f"📏 Tamaño: {image_path.stat().st_size / 1024:.0f}KB")
    print(f"{'─'*60}")

    # Codificar imagen
    encoded = encode_image(str(image_path))
    if encoded is None:
        return {"success": False, "error": "No se pudo codificar la imagen"}

    # Consultar API
    print(f"⏳ Consultando API ({CONFIG['model']})...")
    start_time = time.time()
    result = query_with_fallback(encoded, final_prompt)
    elapsed = time.time() - start_time

    if not result["success"]:
        print(f"  ❌ Error: {result.get('error', 'Desconocido')}")
        return result

    print(f"  ✅ Respuesta recibida en {elapsed:.1f}s")
    print(f"  📊 Tokens: {result.get('total_tokens', 'N/A')}")
    print(f"  🤖 Modelo: {result.get('model_used', 'N/A')}")
    print(f"\n{result['text']}\n")

    # Agregar metadatos
    result.update({
        "archivo": str(image_path),
        "archivo_nombre": image_path.name,
        "prompt": final_prompt,
        "elapsed_seconds": round(elapsed, 2),
        "timestamp": datetime.now().isoformat(),
    })

    # Guardar reporte
    if save_report:
        REPORTS_DIR.mkdir(parents=True, exist_ok=True)
        ts = int(time.time())
        report_path = REPORTS_DIR / f"vision_{image_path.stem}_{ts}.json"
        with open(report_path, "w") as f:
            json.dump(result, f, indent=2, ensure_ascii=False)
        print(f"💾 Reporte guardado: {report_path}")

    return result


def analyze_batch(directory, prompt=None, prompt_type=None, pattern="*.[jJ][pP][gG]"):
    """
    Analiza todas las imágenes de un directorio.
    """
    directory = Path(directory)
    if not directory.exists():
        print(f"❌ Directorio no encontrado: {directory}")
        return []

    # Encontrar imágenes
    images = []
    for ext in ["*.jpg", "*.jpeg", "*.png", "*.bmp", "*.tiff"]:
        images.extend(directory.glob(ext))
    
    images = sorted(set(images))  # Eliminar duplicados
    
    if not images:
        print(f"❌ No se encontraron imágenes en {directory}")
        return []

    # Limitar cantidad
    if len(images) > CONFIG["max_images_per_batch"]:
        print(f"⚠️  {len(images)} imágenes encontradas, limitando a {CONFIG['max_images_per_batch']}")
        images = images[:CONFIG["max_images_per_batch"]]

    print(f"\n{'='*60}")
    print(f"📦 BATCH: {len(images)} imágenes en {directory}")
    print(f"{'='*60}")

    results = []
    for i, img in enumerate(images, 1):
        print(f"\n[{i}/{len(images)}]")
        result = analyze_image(img, prompt=prompt, prompt_type=prompt_type, save_report=True)
        results.append(result)
        # Pequeña pausa entre requests para no rate-limit
        if i < len(images):
            time.sleep(1.5)

    # Reporte consolidado
    ts = int(time.time())
    batch_report = {
        "batch": {
            "directorio": str(directory),
            "total_images": len(images),
            "prompt": prompt or PROMPTS.get(prompt_type, PROMPTS["default"]),
        },
        "results": [
            {
                "archivo": r.get("archivo", "?"),
                "success": r.get("success", False),
                "model_used": r.get("model_used"),
                "total_tokens": r.get("total_tokens", 0),
                "text_preview": r.get("text", "")[:200] if r.get("success") else r.get("error", ""),
            }
            for r in results
        ],
        "timestamp": datetime.now().isoformat(),
    }

    batch_path = REPORTS_DIR / f"batch_{directory.name}_{ts}.json"
    with open(batch_path, "w") as f:
        json.dump(batch_report, f, indent=2, ensure_ascii=False)
    
    print(f"\n{'='*60}")
    print(f"✅ BATCH COMPLETO: {len(results)} imágenes analizadas")
    exitosos = sum(1 for r in results if r.get("success"))
    print(f"   Exitosas: {exitosos}/{len(results)}")
    print(f"💾 Reporte batch: {batch_path}")

    return results


# ═══════════════════════════════════════════════════════════════
#  CLI
# ═══════════════════════════════════════════════════════════════

def main():
    parser = argparse.ArgumentParser(
        description="👁️ NEXUS Vision Grok — Análisis visual con IA",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Ejemplos:
  %(prog)s analyze frame.jpg
  %(prog)s analyze frame.jpg --prompt-type chapa
  %(prog)s analyze frame.jpg --prompt "Describe este vehículo"
  %(prog)s batch ./frames/ --prompt-type scene
  %(prog)s chapa chapa_frame.jpg
  %(prog)s rostro face_extracted.jpg
  %(prog)s scene escena.jpg
  %(prog)s text texto_borroso.jpg
        """
    )
    subparsers = parser.add_subparsers(dest="comando", help="Comando")

    # ── analyze ──
    p_analyze = subparsers.add_parser("analyze", help="Analizar imagen con prompt personalizado")
    p_analyze.add_argument("imagen", help="Ruta a la imagen")
    p_analyze.add_argument("--prompt", "-p", help="Prompt personalizado")
    p_analyze.add_argument("--prompt-type", "-t", choices=PROMPTS.keys(), 
                          help="Tipo de prompt predefinido")
    p_analyze.set_defaults(func=lambda a: analyze_image(a.imagen, prompt=a.prompt, prompt_type=a.prompt_type))

    # ── batch ──
    p_batch = subparsers.add_parser("batch", help="Analizar todas las imágenes de un directorio")
    p_batch.add_argument("directorio", help="Directorio con imágenes")
    p_batch.add_argument("--prompt", "-p", help="Prompt personalizado")
    p_batch.add_argument("--prompt-type", "-t", choices=PROMPTS.keys(),
                        help="Tipo de prompt predefinido")
    p_batch.set_defaults(func=lambda a: analyze_batch(a.directorio, prompt=a.prompt, prompt_type=a.prompt_type))

    # ── Predefinidos (atajos) ──
    for cmd_name, prompt_type in [("chapa", "chapa"), ("rostro", "rostro"), 
                                    ("scene", "scene"), ("text", "text")]:
        p = subparsers.add_parser(cmd_name, help=f"Analizar imagen con prompt '{prompt_type}'")
        p.add_argument("imagen", help="Ruta a la imagen")
        p.set_defaults(func=lambda a, pt=prompt_type: analyze_image(a.imagen, prompt_type=pt))

    args = parser.parse_args()
    if not args.comando:
        parser.print_help()
        # Además mostrar balance
        verificar_balance()
        return

    # Crear directorios
    REPORTS_DIR.mkdir(parents=True, exist_ok=True)

    # Verificar API key antes de ejecutar
    if not get_api_key():
        print("\n⚠️  Para usar este módulo, configura tu OPENROUTER_API_KEY:")
        print("   echo 'OPENROUTER_API_KEY=sk-or-v1-tu-key' > " + str(BASE_DIR.parent / ".env"))
        return

    args.func(args)


def verificar_balance():
    """Verifica el balance de OpenRouter."""
    api_key = get_api_key()
    if not api_key:
        return
    
    try:
        req = urllib.request.Request(
            "https://openrouter.ai/api/v1/auth/key",
            headers={"Authorization": f"Bearer {api_key}"},
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            if "data" in data:
                d = data["data"]
                print(f"\n💰 OpenRouter Balance: {d.get('credits', 'N/A')} USD")
                print(f"   Límite: {d.get('limit', 'N/A')}")
                print(f"   Uso: {d.get('usage', 'N/A')}")
    except Exception as e:
        print(f"\n⚠️  No se pudo verificar balance: {e}")


if __name__ == "__main__":
    main()
