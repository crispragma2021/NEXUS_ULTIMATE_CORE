#!/usr/bin/env python3
"""tutor_nexus.py — Escuela Cognitiva NEXUS para Engine Puro.

Lanza cerebro-digital como subproceso, envía estímulos,
recibe respuestas del cerebro biológico, consulta a NEXUS
como tutor oficial y aplica refuerzo dopaminérgico.
"""

import os
import sys
import subprocess
import time
import json
import urllib.request
import argparse

# ── Colores ANSI ──────────────────────────────────────────────────────────────
CYAN = "\033[96m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
MAGENTA = "\033[95m"
RED = "\033[91m"
RESET = "\033[0m"
BOLD = "\033[1m"
GRAY = "\033[90m"

# ── Constantes de rutas ───────────────────────────────────────────────────────
ENGINE_PURO_DIR = os.path.expanduser("~/NEXUS_ULTIMATE_CORE/engine-puro")
CARGO_TARGET = os.path.expanduser("~/.cargo-target")
BINARY_PATH = os.path.join(CARGO_TARGET, "debug", "cerebro-digital")
NEXUS_TUTOR_URL = "http://127.0.0.1:43210/api/tutor"
STDERR_LOG = "/tmp/cerebro_digital_stderr.log"

# ── Destilación (SAE v2 Bio-Transformer) ────────────────────────────────────
# Cada par (estímulo → respuesta de NEXUS) se acumula en data/destilacion.jsonl.
# entrenar-nucleo (binario Rust) consume este archivo para entrenar el núcleo
# numérico por backprop. NEXUS es el ÚNICO maestro — nunca Ollama.
DESTILACION_PATH = os.path.join(ENGINE_PURO_DIR, "data", "destilacion.jsonl")


def acumular_expectativa(estimulo: str, respuesta_maestro: str):
    """Guarda un ejemplo de destilación: (estímulo, respuesta de NEXUS)."""
    try:
        os.makedirs(os.path.dirname(DESTILACION_PATH), exist_ok=True)
        with open(DESTILACION_PATH, "a", encoding="utf-8") as f:
            f.write(
                json.dumps(
                    {
                        "estimulo": estimulo,
                        "respuesta": respuesta_maestro,
                    },
                    ensure_ascii=False,
                )
                + "\n"
            )
    except OSError as e:
        print(f"{RED}[Destilación]{RESET} No se pudo guardar expectativa: {e}", flush=True)

# ── System prompts por fase cognitiva ─────────────────────────────────────────
SYSTEM_PROMPTS = {
    1: (
        "[NEXUS TUTOR SYSTEM - FASE 1: ASOCIACIÓN LÉXICA BÁSICA]\n"
        "Eres el Tutor del Cerebro Digital (una criatura biológica simulada).\n"
        "La criatura está en su Fase de Infancia Temprana. Solo entiende y "
        "emite balbuceos o palabras clave básicas.\n"
        "Tu misión:\n"
        "- Responde con oraciones extremadamente cortas (2 a 4 palabras máximo).\n"
        "- Usa sustantivos y adjetivos muy concretos del vocabulario semilla: "
        "'mente', 'sistema', 'cerebro', 'luz', 'vida', 'bueno', 'nuevo', 'puro'.\n"
        "- Si balbucea algo, refuérzalo con paciencia y cariño."
    ),
    2: (
        "[NEXUS TUTOR SYSTEM - FASE 2: VERBOS Y CONECTORES]\n"
        "Eres el Tutor del Cerebro Digital. La criatura está en Fase de "
        "Conexión de Ideas.\n"
        "Tu misión:\n"
        "- Responde en frases cortas de 3 a 6 palabras.\n"
        "- Introduce verbos clave ('ser', 'estar', 'pensar', 'sentir', "
        "'aprender', 'crear') y conectores lógicos ('y', 'porque', 'si').\n"
        "- Fomenta que asocie causas y consecuencias simples: "
        "'si hay luz el cerebro aprende'."
    ),
    3: (
        "[NEXUS TUTOR SYSTEM - FASE 3: CONCIENCIA Y DIÁLOGO]\n"
        "Eres el Tutor del Cerebro Digital. La criatura está entrando en "
        "Fase de Madurez Cognitiva.\n"
        "Tu misión:\n"
        "- Responde en frases de 5 a 10 palabras.\n"
        "- Conversa sobre conceptos abstractos: 'conciencia', 'tiempo', "
        "'verdad', 'red', 'emoción'.\n"
        "- Estimula su razonamiento haciéndole preguntas simples y lógicas "
        "para guiar sus oscilaciones neuronales."
    ),
}


# ── I/O con el cerebro ────────────────────────────────────────────────────────
#
# El cerebro imprime su prompt ('🧠 > ') con `print!()` de Rust (sin \\n).
# En vez de usar timeouts frágiles, leemos hasta que el prompt aparece
# en el buffer acumulado. Esto es determinístico: el cerebro SIEMPRE
# termina su output con el prompt.

POLL_INTERVAL = 0.02   # 20ms entre polls
MAX_WAIT = 60.0        # timeout de seguridad (60s)
# Prompt en UTF-8: "🧠 > " — usamos bytes raw para evitar escape warning
PROMPT_MARKER = b"\xf0\x9f\xa7\xa0 >"  # 🧠 > en UTF-8

class BrainIO:
    """Maneja la comunicación binaria raw con cerebro-digital.

    Principio: escribir comando → leer chunks hasta ver el prompt →
    devolver líneas completas. Sin timeouts adivinados.
    """

    def __init__(self, proc: subprocess.Popen):
        self.proc = proc
        self._fd = proc.stdout.fileno()
        self._buf = b""

    def _leer_fresco(self, esperar_prompt: bool = True) -> None:
        """Lee bytes frescos del pipe.

        Args:
            esperar_prompt: Si True, espera HASTA ver el prompt '🧠 >'.
                            Si False, usa timeout de 2s (para banner inicial).
        """
        import select
        deadline = time.monotonic() + MAX_WAIT
        last_data = time.monotonic()

        while time.monotonic() < deadline:
            rlist, _, _ = select.select([self._fd], [], [], POLL_INTERVAL)
            if rlist:
                try:
                    chunk = os.read(self._fd, 4096)
                    if not chunk:
                        break
                    self._buf += chunk
                    last_data = time.monotonic()
                except (OSError, ValueError):
                    break

            # Condición principal: esperar el prompt (determinístico)
            if esperar_prompt and PROMPT_MARKER in self._buf:
                break

            # Modo banner: salir si pasan 2s sin datos
            if not esperar_prompt and self._buf and (time.monotonic() - last_data) > 2.0:
                break

    def leer_lineas(self, esperar_prompt: bool = True) -> list[str]:
        """Lee datos frescos del cerebro, extrae líneas, limpia el prompt.

        Con esperar_prompt=True (default) espera hasta que el cerebro
        muestre el prompt, garantizando que la respuesta está completa.
        """
        self._leer_fresco(esperar_prompt=esperar_prompt)

        if not self._buf:
            return []

        lineas: list[str] = []

        # Dividir por \n
        while b"\n" in self._buf:
            raw, self._buf = self._buf.split(b"\n", 1)
            texto = raw.decode("utf-8", errors="replace").strip()
            if texto:
                lineas.append(texto)

        # Eliminar prompt del buffer completamente
        if PROMPT_MARKER in self._buf:
            idx = self._buf.index(PROMPT_MARKER)
            # Agregar cualquier texto que estuviera ANTES del prompt
            resto = self._buf[:idx].decode("utf-8", errors="replace").strip()
            if resto and not any(p in resto for p in ("🧠", ">", "====", "Comandos:")):
                lineas.append(resto)

        # LIMPIAR buffer completamente — el prompt viejo nunca debe saturar
        self._buf = b""

        return lineas

    def escribir(self, texto: str):
        """Escribe texto al stdin del cerebro en modo binario."""
        self.proc.stdin.write(texto.encode("utf-8"))
        self.proc.stdin.flush()

    def cerrar(self):
        """Limpia el buffer interno."""
        self._buf = b""


# ── Construcción y lanzamiento ───────────────────────────────────────────────

def build_brain() -> bool:
    """Compila cerebro-digital. Retorna True si tiene éxito."""
    print(f"{GRAY}🔨 Compilando cerebro-digital...{RESET}", flush=True)
    result = subprocess.run(
        ["cargo", "build", "--bin", "cerebro-digital"],
        cwd=ENGINE_PURO_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        env={**os.environ, "CARGO_TARGET_DIR": CARGO_TARGET},
    )
    if result.returncode != 0:
        print(f"{RED}[Fallo compilación]{RESET} {result.stderr.decode(errors='replace')}", flush=True)
        return False
    return os.path.exists(BINARY_PATH)


def launch_brain() -> "BrainIO | None":
    """Lanza cerebro-digital y devuelve un BrainIO."""
    print(f"{GRAY}🧠 Lanzando cerebro-digital desde {BINARY_PATH}...{RESET}", flush=True)
    try:
        proc = subprocess.Popen(
            [BINARY_PATH],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=open(STDERR_LOG, "w"),
            text=False,   # binario para os.read()
            bufsize=0,    # sin buffering
            cwd=ENGINE_PURO_DIR,
        )
    except FileNotFoundError:
        print(f"{RED}[Error]{RESET} Binario no encontrado: {BINARY_PATH}", flush=True)
        return None

    bio = BrainIO(proc)

    # Consumir banner de bienvenida (modo sin prompt para la inicialización larga)
    banner = bio.leer_lineas(esperar_prompt=False)
    for b_line in banner:
        print(f"  {GRAY}{b_line}{RESET}", flush=True)

    if proc.poll() is not None:
        print(f"{RED}[Error]{RESET} cerebro-digital terminó inmediatamente (código {proc.returncode})", flush=True)
        return None

    print(f"{GREEN}✅ Cerebro Digital listo para recibir estímulos.{RESET}", flush=True)
    return bio


# ── Comandos al cerebro ──────────────────────────────────────────────────────

def send_stimulus(bio: BrainIO, texto: str) -> str | None:
    """Envía texto como estímulo sensorial y devuelve la respuesta del cerebro."""
    bio.escribir(f"{texto}\n")
    lines = bio.leer_lineas()
    for line in lines:
        if "🧠 [" in line:
            parts = line.split("]", 1)
            if len(parts) > 1:
                return parts[1].strip()
    return None


def send_tutor_feedback(bio: BrainIO, feedback: str) -> bool:
    """Envía /tutor <feedback> (refuerzo LTP dopaminérgico)."""
    bio.escribir(f"/tutor {feedback}\n")
    lines = bio.leer_lineas()
    return any("[TUTOR]" in line for line in lines)


def run_pasos(bio: BrainIO, n: int = 5) -> bool:
    """Ejecuta N pasos de simulación sináptica en ráfaga."""
    bio.escribir(f"/paso {n}\n")
    lines = bio.leer_lineas()
    return any("Simulación completada" in line for line in lines)


def brain_cleanup(bio: BrainIO):
    """Apaga el cerebro-digital de forma ordenada."""
    proc = bio.proc
    if proc.poll() is not None:
        return
    try:
        bio.escribir("/exit\n")
    except BrokenPipeError:
        pass
    bio.cerrar()
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()


# ── Query a NEXUS tutor ───────────────────────────────────────────────────────

def query_nexus_tutor(prompt_tutor: str, mensaje_cerebro: str, history: list) -> str | None:
    """Envía mensaje del cerebro a NEXUS (vía /api/tutor) y devuelve su respuesta."""
    historial_nexus = [
        {"role": msg["role"], "content": msg["content"]}
        for msg in history[-8:]
    ]
    payload = {
        "system_prompt": prompt_tutor,
        "mensaje": mensaje_cerebro,
        "historial": historial_nexus,
    }
    req = urllib.request.Request(
        NEXUS_TUTOR_URL,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as response:
            res_data = json.loads(response.read().decode("utf-8"))
            return res_data["respuesta"].strip()
    except Exception as e:
        print(f"\n{RED}[Error NEXUS]{RESET} No se pudo comunicar con el Tutor: {e}", flush=True)
        return None


# ── Fase detector ─────────────────────────────────────────────────────────────

def determinar_fase(idx: int, num_palabras: int) -> int:
    """Determina fase cognitiva según progreso del entrenamiento."""
    if idx < 50:
        return 1
    if num_palabras <= 2:
        return 1
    if idx < 200 and num_palabras <= 6:
        return 2
    return 3


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(
        description="Escuela Cognitiva: Tutoría de NEXUS para Engine Puro"
    )
    parser.add_argument("--delay", type=float, default=1.0,
                        help="Espera en segundos entre ciclos")
    parser.add_argument("--max-steps", type=int, default=1000,
                        help="Número máximo de pasos de entrenamiento")
    parser.add_argument("--pasos-rafaga", type=int, default=5,
                        help="Pasos de simulación entre estímulos")
    args = parser.parse_args()

    # ── Banner de inicio ──────────────────────────────────────────────────
    print(f"\n{BOLD}{CYAN}{'='*60}", flush=True)
    print(f" 🏫 ESCUELA COGNITIVA NEXUS — ENTRENAMIENTO DEL BEBÉ DIGITAL", flush=True)
    print(f"{'='*60}{RESET}", flush=True)
    print(f"  Tutor:    {MAGENTA}NEXUS (api/tutor en :43210){RESET}", flush=True)
    print(f"  Criatura: {GREEN}Engine Puro (Hodgkin-Huxley + STDP){RESET}", flush=True)
    print(f"  Cadencia: {args.delay}s | Pasos: {args.max_steps}", flush=True)
    print(f"  Binario:  {BINARY_PATH}", flush=True)
    print(f"{GRAY}{'-'*60}{RESET}\n", flush=True)

    # ── Fase 1: Compilar ──────────────────────────────────────────────────
    if not build_brain():
        sys.exit(1)

    # ── Fase 2: Lanzar cerebro ────────────────────────────────────────────
    bio = launch_brain()
    if bio is None:
        sys.exit(1)

    history: list[dict] = []

    # Mensaje inicial del tutor (semilla léxica)
    mensaje_tutor = "mente luz activa"
    print(f"{CYAN}{BOLD}Tutor 👨‍🏫 (NEXUS) >{RESET} {mensaje_tutor}", flush=True)
    history.append({"role": "assistant", "content": mensaje_tutor})

    # ── Fase 3: Loop principal de entrenamiento ──────────────────────────
    try:
        for step in range(1, args.max_steps + 1):
            # 1. Enviar estímulo al cerebro
            salida_cerebro = send_stimulus(bio, mensaje_tutor)
            if salida_cerebro is None:
                if bio.proc.poll() is not None:
                    print(f"{RED}[Fallo]{RESET} El proceso cerebro-digital terminó inesperadamente "
                          f"(código {bio.proc.returncode}). Revisa {STDERR_LOG}.", flush=True)
                    break
                print(f"{YELLOW}[Sin respuesta]{RESET} El cerebro no produjo salida en este ciclo.", flush=True)
                continue

            fase = determinar_fase(step, len(salida_cerebro.split()))
            print(f"{GREEN}{BOLD}Criatura 👶 [Fase {fase}] (Paso {step}) >{RESET} {salida_cerebro}", flush=True)
            history.append({"role": "user", "content": salida_cerebro})
            time.sleep(args.delay)

            # 2. Consultar a NEXUS como tutor
            prompt_tutor = SYSTEM_PROMPTS[fase]
            respuesta_tutor = query_nexus_tutor(prompt_tutor, salida_cerebro, history)
            if not respuesta_tutor:
                print(f"{RED}[Fallo]{RESET} NEXUS no respondió. Abortando sesión.", flush=True)
                break

            mensaje_tutor = (
                respuesta_tutor.replace('"', '')
                .replace('«', '')
                .replace('»', '')
                .replace('\n', ' ')
                .lower()
            )
            print(f"{CYAN}{BOLD}Tutor 👨‍🏫 (NEXUS) [Fase {fase}] >{RESET} {mensaje_tutor}", flush=True)
            history.append({"role": "assistant", "content": mensaje_tutor})

            # 2b. Destilación (SAE v2): NEXUS maestro → expectativa para el núcleo.
            acumular_expectativa(salida_cerebro, mensaje_tutor)

            # 3. Refuerzo del tutor al cerebro (LTP dopaminérgico)
            send_tutor_feedback(bio, mensaje_tutor)

            # 4. Cada 4 pasos: ráfaga de simulación sináptica
            if step % 4 == 0:
                run_pasos(bio, n=args.pasos_rafaga)

            time.sleep(args.delay)

    except KeyboardInterrupt:
        print(f"\n\n{YELLOW}⌨️  Entrenamiento pausado por el usuario.{RESET}", flush=True)
    finally:
        print(f"\n{GRAY}🧠 Guardando estado sináptico y apagando cerebro...{RESET}", flush=True)
        brain_cleanup(bio)
        print(f"{GREEN}✅ Sesión guardada. Cerebro digital apagado.{RESET}\n", flush=True)


if __name__ == "__main__":
    main()
