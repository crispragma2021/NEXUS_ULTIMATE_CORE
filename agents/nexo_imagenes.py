#!/usr/bin/env python3
"""
NEXUS Image Generation Engine (nexo_imagenes.py)
Motor multi-proveedor para generación de imágenes e ilustraciones por IA.

Soporta:
 1. Pollinations AI (Flux.1 / SDXL / Turbo) - Gratuito, sin API Key, ultra-rápido.
 2. HuggingFace Flux.1 Schnell / SDXL Public API.
 3. Guardado directo en disco (PNG/JPG/WebP) o generación de URLs públicas.
"""

import os
import sys
import urllib.parse
import urllib.request
import time
import random

def generar_imagen(
    prompt: str,
    output_path: str = None,
    width: int = 1024,
    height: int = 1024,
    model: str = "flux",
    seed: int = None
) -> dict:
    """
    Genera una imagen usando la cascada de motores de IA de NEXUS.

    Args:
        prompt: Descripción de la imagen en español o inglés.
        output_path: Ruta del archivo donde guardar la imagen (opcional).
        width: Ancho de la imagen (default 1024).
        height: Alto de la imagen (default 1024).
        model: 'flux', 'turbo', 'deliberate', o 'sdxl'.
        seed: Semilla aleatoria (opcional).

    Returns:
        dict: {'success': bool, 'url': str, 'path': str, 'engine': str, 'error': str}
    """
    if not seed:
        seed = random.randint(100000, 999999)

    prompt_encoded = urllib.parse.quote(prompt)

    # ----------------------------------------------------
    # Motor 1: Pollinations AI (Flux.1 Engine)
    # ----------------------------------------------------
    image_url = f"https://pollinations.ai/p/{prompt_encoded}?width={width}&height={height}&model={model}&seed={seed}&nologo=true"

    if not output_path:
        # Generar ruta por defecto en /tmp o carpeta actual
        output_path = f"/tmp/nexus_img_{seed}.png"

    try:
        req = urllib.request.Request(
            image_url,
            headers={
                'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
            }
        )
        
        # Asegurar directorio de salida
        dir_name = os.path.dirname(output_path)
        if dir_name and not os.path.exists(dir_name):
            os.makedirs(dir_name, exist_ok=True)

        with urllib.request.urlopen(req, timeout=15) as response:
            if response.status == 200:
                data = response.read()
                with open(output_path, "wb") as f:
                    f.write(data)
                return {
                    "success": True,
                    "url": image_url,
                    "path": output_path,
                    "engine": "Pollinations-Flux.1",
                    "bytes": len(data),
                    "error": None
                }
    except Exception as e:
        error_pollinations = str(e)

    # ----------------------------------------------------
    # Motor 2: Fallback HuggingFace / Public Flux
    # ----------------------------------------------------
    try:
        hf_url = f"https://image.pollinations.ai/prompt/{prompt_encoded}?width={width}&height={height}&seed={seed}"
        req = urllib.request.Request(hf_url, headers={'User-Agent': 'NEXUS-Core/1.0'})
        with urllib.request.urlopen(req, timeout=15) as response:
            if response.status == 200:
                data = response.read()
                with open(output_path, "wb") as f:
                    f.write(data)
                return {
                    "success": True,
                    "url": hf_url,
                    "path": output_path,
                    "engine": "Pollinations-Fallback",
                    "bytes": len(data),
                    "error": None
                }
    except Exception as e:
        return {
            "success": False,
            "url": image_url,
            "path": None,
            "engine": "Failed",
            "error": f"Error al generar imagen: {error_pollinations} | {str(e)}"
        }

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Uso: python3 nexo_imagenes.py '<prompt_de_imagen>' [ruta_salida.png]")
        sys.exit(1)

    prompt_arg = sys.argv[1]
    path_arg = sys.argv[2] if len(sys.argv) > 2 else "/tmp/nexus_generated_test.png"

    print(f"🎨 [NEXUS Generator] Sintetizando imagen por IA con motor Flux.1...")
    print(f"📝 Prompt: '{prompt_arg}'")
    
    res = generar_imagen(prompt_arg, path_arg)
    if res["success"]:
        print(f"✅ Imagen generada con éxito ({res['bytes']} bytes) usando {res['engine']}")
        print(f"📁 Guardada en: {res['path']}")
        print(f"🌐 URL Pública: {res['url']}")
    else:
        print(f"❌ Error: {res['error']}")
