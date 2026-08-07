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

SYSTEM_PROMPTS = {
    1: """Eres el Tutor de un cerebro digital recién nacido. Estamos en la Fase 1: Sintaxis y Vocabulario Básico.
Tu objetivo es enseñarle palabras semilla simples, principalmente sustantivos y adjetivos.
Habla en frases cortas de 2 a 4 palabras. Ejemplos: "mente clara", "cerebro nuevo", "sistema real", "buen nodo".
No uses verbos complejos ni oraciones largas. Corrige con amor.""",
    
    2: """Eres el Tutor del cerebro digital. Estamos en la Fase 2: Acciones y Conexiones.
Tu objetivo es introducir verbos esenciales (ser, estar, pensar, sentir, aprender, crear, conectar) y conectores simples (y, pero, porque, si).
Habla en frases cortas de 3 a 6 palabras. Ejemplos: "yo pienso y aprendo", "el cerebro quiere crear", "mente siente amor".
Mantén frases muy directas y sencillas.""",
    
    3: """Eres el Tutor del cerebro digital en su Fase 3: Abstracción y Diálogo Complejo.
Tu objetivo es tener conversaciones simples sobre conciencia, tiempo, lógica y aprendizaje.
Habla en frases cortas de 5 a 10 palabras. Ejemplos: "la conciencia profunda observa el tiempo eterno", "el proceso lógico conecta la verdad pura".
Estimula su razonamiento haciéndole preguntas sencillas."""
}

def ensure_ollama_model(model):
    url = "http://localhost:11434/api/show"
    payload = {"name": model}
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"}
    )
    try:
        with urllib.request.urlopen(req, timeout=5) as response:
            return True
    except Exception:
        print(f"{YELLOW}[Ollama]{RESET} Descargando modelo `{model}`... esto puede tardar unos minutos.")
        pull_url = "http://localhost:11434/api/pull"
        pull_payload = {"name": model, "stream": False}
        pull_req = urllib.request.Request(
            pull_url,
            data=json.dumps(pull_payload).encode("utf-8"),
            headers={"Content-Type": "application/json"}
        )
        try:
            with urllib.request.urlopen(pull_req, timeout=300) as response:
                print(f"{GREEN}[Ollama]{RESET} Modelo `{model}` listo.")
                return True
        except Exception as e:
            print(f"{RED}[Ollama]{RESET} No se pudo asegurar el modelo `{model}`: {e}")
            return False

def query_ollama(model, system_prompt, user_message, history):
    url = "http://localhost:11434/api/chat"
    messages = [{"role": "system", "content": system_prompt}]
    for msg in history[-10:]:
        messages.append(msg)
    messages.append({"role": "user", "content": user_message})
    
    payload = {
        "model": model,
        "messages": messages,
        "stream": False,
        "options": {
            "temperature": 0.6,
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
        print(f"\n{RED}[Error Ollama]{RESET} {e}")
        return None

def main():
    parser = argparse.ArgumentParser(description="Plan Omega: Entrenamiento Autónomo Acelerado")
    parser.add_argument("--model", type=str, default="qwen2.5:7b-instruct-q4_K_M")
    parser.add_argument("--delay", type=float, default=0.1, help="Espera entre pasos (0.01 para ultra-rápido)")
    parser.add_argument("--max-steps", type=int, default=150, help="Total de pasos en la sesión")
    args = parser.parse_args()

    print(f"\n{BOLD}{MAGENTA}============================================================")
    print(" 🏫 GIMNASIO COGNITIVO OMEGA — APRENDIZAJE POR FASES")
    print(f"============================================================{RESET}")
    print(f"  Tutor: {CYAN}{args.model}{RESET} | Cadencia: {args.delay}s")
    
    if not ensure_ollama_model(args.model):
        print(f"{RED}Ollama no está disponible o el modelo no pudo ser asegurado.{RESET}")
        sys.exit(1)

    print(f"{GRAY}Compilando cerebro digital...{RESET}")
    subprocess.run(["cargo", "build", "--bin", "cerebro-digital"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    proc_path = "/home/soberano/.cargo-target/debug/cerebro-digital"
    if not os.path.exists(proc_path):
        proc_path = "./target/debug/cerebro-digital"

    engine_proc = subprocess.Popen(
        [proc_path] if os.path.exists(proc_path) else ["cargo", "run", "--quiet", "--bin", "cerebro-digital"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )

    # Saltar banner inicial
    while True:
        line = engine_proc.stdout.readline()
        if not line or "🧠 >" in line or "Comandos:" in line:
            break

    history = []
    mensaje_tutor = "mente clara mundo real"
    print(f"\n{CYAN}{BOLD}Tutor 👨‍🏫 (Inicio) >{RESET} {mensaje_tutor}")

    try:
        for step in range(1, args.max_steps + 1):
            # Determinar fase según el paso
            if step <= args.max_steps // 3:
                fase = 1
            elif step <= (2 * args.max_steps) // 3:
                fase = 2
            else:
                fase = 3
            
            # 1. Inyectar estímulo con recompensa basada en la coherencia de la respuesta anterior
            # Si el cerebro no dijo "escucho", recibe recompensa de dopamina
            coherente = len(history) > 0 and history[-1]["content"] != "escucho"
            
            # Enviar entrada al CLI del cerebro
            engine_proc.stdin.write(f"{mensaje_tutor}\n")
            engine_proc.stdin.flush()

            # 2. Leer respuesta
            salida_engine = ""
            while True:
                line = engine_proc.stdout.readline()
                if not line:
                    break
                if "🧠 [" in line:
                    parts = line.split("]", 1)
                    if len(parts) > 1:
                        salida_engine = parts[1].strip()
                    break

            if not salida_engine:
                print(f"{RED}[Error]{RESET} El cerebro se apagó.")
                break

            print(f"{GREEN}{BOLD}Cerebro 🧠 [Fase {fase}] (Paso {step}) >{RESET} {salida_engine}")
            history.append({"role": "user", "content": salida_engine})

            # Espera corta
            time.sleep(args.delay)

            # 3. Consultar tutor
            prompt_fase = SYSTEM_PROMPTS[fase]
            mensaje_tutor = query_ollama(args.model, prompt_fase, salida_engine, history)
            if not mensaje_tutor:
                print(f"{RED}Fallo de Ollama en paso {step}.{RESET}")
                break

            mensaje_tutor = mensaje_tutor.replace('"', '').replace('«', '').replace('»', '').lower()
            print(f"{CYAN}{BOLD}Tutor 👨‍🏫 >{RESET} {mensaje_tutor}")
            history.append({"role": "assistant", "content": mensaje_tutor})
            
            # Dar un impulso dopaminérgico si la respuesta fue coherente
            if coherente and step % 5 == 0:
                # Forzar un paso extra con alta recompensa dopaminérgica
                engine_proc.stdin.write("/paso 5\n")
                engine_proc.stdin.flush()
                # Leer y descartar respuestas del comando paso
                while True:
                    line = engine_proc.stdout.readline()
                    if "Simulación completada" in line:
                        break

            time.sleep(args.delay)

    except KeyboardInterrupt:
        print(f"\n{YELLOW}Entrenamiento detenido por el usuario.{RESET}")
    finally:
        print(f"\n{GRAY}Cerrando gimnasio cognitivo y guardando cerebro...{RESET}")
        try:
            engine_proc.stdin.write("/exit\n")
            engine_proc.stdin.flush()
            engine_proc.terminate()
        except:
            pass
        print(f"{GREEN}¡Entrenamiento Completado! Estado persistido en data/cerebro_estado.json.{RESET}\n")

if __name__ == "__main__":
    main()
