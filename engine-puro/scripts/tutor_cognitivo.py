 #!/usr/bin/env python3
import os
import sys
import subprocess
import time
import json
import urllib.request
import argparse

# Colores ANSI para la consola
CYAN = "\033[96m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
MAGENTA = "\033[95m"
RED = "\033[91m"
RESET = "\033[0m"
BOLD = "\033[1m"
GRAY = "\033[90m"

SYSTEM_PROMPT = """Eres el Tutor de un cerebro digital neuromórfico recién nacido que está aprendiendo a hablar español.
Tu misión es enseñarle sintaxis, gramática y conceptos simples de forma paciente y didáctica.

REGLAS CRÍTICAS DE COMUNICACIÓN:
1. Responde SIEMPRE con frases cortas y simples (entre 4 y 10 palabras máximo). No uses párrafos largos.
2. Si el cerebro dice algo con sentido, refuérzalo e introduce un concepto relacionado (ej: "Sí, el sistema está activo y tiene luz").
3. Si dice algo incoherente, corrígelo con cariño repitiendo la frase correcta de forma muy simple.
4. Usa palabras clave del vocabulario semilla del motor: mente, sistema, cerebro, red, nodo, señal, luz, vida, tiempo, conciencia, bueno, real, claro.
5. Mantén un tono amigable, paciente y alentador.
"""

def query_ollama(model, system_prompt, user_message, history):
    url = "http://localhost:11434/api/chat"
    messages = [{"role": "system", "content": system_prompt}]
    
    # Añadir los últimos mensajes del historial para contexto (máximo 10)
    for msg in history[-10:]:
        messages.append(msg)
        
    messages.append({"role": "user", "content": user_message})
    
    payload = {
        "model": model,
        "messages": messages,
        "stream": False,
        "options": {
            "temperature": 0.7,
            "max_tokens": 50
        }
    }
    
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"}
    )
    
    try:
        with urllib.request.urlopen(req, timeout=15) as response:
            res_data = json.loads(response.read().decode("utf-8"))
            return res_data["message"]["content"].strip()
    except Exception as e:
        print(f"\n{RED}[Error Ollama]{RESET} No se pudo comunicar con el servicio Ollama: {e}")
        print(f"Asegúrate de que Ollama está corriendo (`ollama serve`) y que el modelo `{model}` esté descargado.")
        return None

def main():
    parser = argparse.ArgumentParser(description="Prótesis de Tutoría Cognitiva para el Engine Puro")
    parser.add_argument("--model", type=str, default="qwen2.5:7b-instruct-q4_K_M", 
                        help="Modelo de Ollama a usar como tutor")
    parser.add_argument("--delay", type=float, default=1.5, 
                        help="Espera en segundos entre interacciones para poder leerlas")
    parser.add_argument("--max-steps", type=int, default=50, 
                        help="Número máximo de interacciones en esta sesión")
    args = parser.parse_args()

    print(f"\n{BOLD}{MAGENTA}============================================================")
    print(" 🏫 GIMNASIO COGNITIVO — PRÓTESIS DE TUTORÍA AUTÓNOMA")
    print(f"============================================================{RESET}")
    print(f"  Tutor (Ollama): {CYAN}{args.model}{RESET}")
    print(f"  Aprendiz (Engine): {GREEN}Cerebro Digital (Rust){RESET}")
    print(f"  Cadencia: {args.delay}s | Pasos Máximos: {args.max_steps}")
    print(f"{GRAY}------------------------------------------------------------{RESET}\n")

    # In iniciar el proceso del cerebro digital de Rust
    # Compilamos primero para asegurar que tenemos la última versión
    print(f"{GRAY}Compilando el engine puro...{RESET}")
    build_proc = subprocess.run(["cargo", "build", "--bin", "cerebro-digital"], 
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if build_proc.returncode != 0:
        print(f"{RED}[Error compilación]{RESET} No se pudo compilar el engine-puro.")
        sys.exit(1)

    print(f"{GRAY}Iniciando cerebro-digital...{RESET}")
    # Ejecutamos el binario directamente
    proc_path = "./target/debug/cerebro-digital"
    if not os.path.exists(proc_path):
        proc_path = "cargo run --quiet --bin cerebro-digital"

    engine_proc = subprocess.Popen(
        [proc_path] if proc_path == "./target/debug/cerebro-digital" else ["cargo", "run", "--quiet", "--bin", "cerebro-digital"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )

    # Leer el banner inicial del cerebro
    banner_lines = []
    while True:
        line = engine_proc.stdout.readline()
        if not line or "🧠 >" in line or "Comandos:" in line:
            break
        banner_lines.append(line.strip())

    print(f"{GRAY}Engine Puro listo y cargado.{RESET}")
    
    # Historial de conversación para Ollama
    history = []
    
    # Primer estímulo para iniciar la conversación
    mensaje_tutor = "hola cerebro sistema activo"
    print(f"\n{CYAN}{BOLD}Tutor 👨‍🏫 >{RESET} {mensaje_tutor}")
    history.append({"role": "assistant", "content": mensaje_tutor})

    try:
        for step in range(1, args.max_steps + 1):
            # 1. Enviar mensaje del tutor al engine
            engine_proc.stdin.write(f"{mensaje_tutor}\n")
            engine_proc.stdin.flush()
            
            # 2. Leer la respuesta del engine
            salida_engine = ""
            while True:
                line = engine_proc.stdout.readline()
                if not line:
                    break
                # Formato esperado: "  🧠 [0.001s] <texto>"
                if "🧠 [" in line:
                    parts = line.split("]", 1)
                    if len(parts) > 1:
                        salida_engine = parts[1].strip()
                    break
            
            if not salida_engine:
                print(f"{RED}[Error Engine]{RESET} No se obtuvo respuesta del cerebro.")
                break
                
            print(f"{GREEN}{BOLD}Cerebro 🧠 (Paso {step}) >{RESET} {salida_engine}")
            history.append({"role": "user", "content": salida_engine})
            
            # Esperar un momento para que sea legible y no sature la simulación
            time.sleep(args.delay)
            
            # 3. Consultar al tutor Ollama qué responderle al cerebro
            # Le pasamos la salida del cerebro como la entrada del usuario para Ollama
            mensaje_tutor = query_ollama(args.model, SYSTEM_PROMPT, salida_engine, history)
            
            if not mensaje_tutor:
                print(f"{RED}Abortando entrenamiento por fallo en Ollama.{RESET}")
                break
                
            # Limpiar un poco el output de Ollama por si acaso
            mensaje_tutor = mensaje_tutor.replace('"', '').replace('«', '').replace('»', '').lower()
            print(f"{CYAN}{BOLD}Tutor 👨‍🏫 >{RESET} {mensaje_tutor}")
            history.append({"role": "assistant", "content": mensaje_tutor})
            
            # Esperar antes de la siguiente iteración
            time.sleep(args.delay)

    except KeyboardInterrupt:
        print(f"\n\n{YELLOW}Sesión de entrenamiento interrumpida por el usuario.{RESET}")
    finally:
        # Cerrar el proceso del engine limpiamente
        print(f"\n{GRAY}Apagando cerebro digital y guardando estado...{RESET}")
        try:
            engine_proc.stdin.write("/exit\n")
            engine_proc.stdin.flush()
            engine_proc.terminate()
            engine_proc.wait(timeout=2)
        except Exception:
            pass
        print(f"{GREEN}¡Entrenamiento completado!{RESET}\n")

if __name__ == "__main__":
    main()
