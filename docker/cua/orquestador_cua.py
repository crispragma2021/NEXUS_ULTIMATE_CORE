#!/usr/bin/env python3
# ═══════════════════════════════════════════════════════════════════════════
# 🔱 NEXUS CUA — ORQUESTADOR DE DECISIÓN
# Implementa la REGLA DE ORQUESTACIÓN del Computer-Using Agent:
#   - DOCKER HEADLESS (por defecto): interacción GUI diaria, navegador,
#     automatización de formularios, flujos rápidos, monitoreo noVNC.
#   - FIRECRACKER (aislamiento KVM): binarios no verificados, descargas
#     sospechosas, análisis/detonación, pruebas que exijan aislamiento HW.
#
# Uso:
#   orquestador_cua.py evaluar "<descripción de la tarea>"
#   orquestador_cua.py ejecutar "<comando>" --entorno docker|firecracker
#   orquestador_cua.py status
# ═══════════════════════════════════════════════════════════════════════════
import argparse
import os
import shutil
import subprocess
import sys
import time

# ─── Palabras clave que activan Firecracker (aislamiento KVM) ────────────
FIRE_CRACKER_PALABRAS = [
    "no verificado", "no confiable", "desconocido", "binario", "malware",
    "exploit", "detonar", "detonación", "pwn", "ransomware", "código hostil",
    "phishing", "payload", "shellcode", "dump", "crash", "kernel", "rootkit",
    "descarga sospechosa", "archivo sospechoso", "ejecutable desconocido",
    "ofensivo", "pentest", "inyección", "inyectar", "vulnerabilidad",
]

DOCKER_COMPOSE = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "docker-compose.cua.yml"
)
FIRECRACKER_DIR = "/home/soberano/NEXUS_ULTIMATE_CORE/firecracker_env"
GHOST_IGNITION = "/home/soberano/NEXUS_ULTIMATE_CORE/scripts/ghost_ignition.sh"


def clasificar_tarea(descripcion: str) -> str:
    """Devuelve 'firecracker' si la tarea exige aislamiento KVM, si no 'docker'."""
    d = descripcion.lower()
    for kw in FIRE_CRACKER_PALABRAS:
        if kw in d:
            return "firecracker"
    return "docker"


def estado_docker() -> str:
    try:
        out = subprocess.run(
            ["docker", "ps", "-a", "--filter", "name=nexus-cua-gui",
             "--format", "{{.Status}}"],
            capture_output=True, text=True, timeout=10,
        ).stdout.strip()
        return out or "sin contenedor"
    except Exception as e:
        return f"error: {e}"


def estado_firecracker() -> str:
    sock = "/tmp/firecracker.socket"
    if os.path.exists(sock):
        return "socket presente (posible MicroVM activa)"
    return "inactivo (sin socket)"


def arrancar_docker() -> bool:
    if not shutil.which("docker"):
        print("❌ docker no instalado")
        return False
    print("🐳 [CUA] Arrancando entorno Docker Headless GUI (Xvfb + noVNC)...")
    r = subprocess.run(
        ["docker", "compose", "-f", DOCKER_COMPOSE, "up", "-d", "--build"],
        capture_output=True, text=True,
    )
    if r.returncode != 0:
        print(r.stderr[-2000:])
        return False
    # Esperar healthcheck
    for _ in range(30):
        st = estado_docker()
        if "healthy" in st:
            break
        time.sleep(1)
    print(f"✅ [CUA] Docker: {estado_docker()}")
    print("🌐 [CUA] noVNC disponible en http://localhost:6080/vnc.html")
    return True


def arrancar_firecracker() -> bool:
    if not os.path.exists(GHOST_IGNITION):
        print("❌ ghost_ignition.sh no existe")
        return False
    if not os.path.exists("/dev/kvm"):
        print("❌ /dev/kvm no disponible (requiere KVM)")
        return False
    print("⚡ [CUA] Arrancando MicroVM Firecracker (aislamiento KVM)...")
    r = subprocess.run(["bash", GHOST_IGNITION], capture_output=True, text=True)
    print(r.stdout[-2000:])
    if r.stderr:
        print(r.stderr[-2000:])
    return r.returncode == 0


def main():
    parser = argparse.ArgumentParser(description="NEXUS CUA Orchestrator")
    sub = parser.add_subparsers(dest="cmd")

    ev = sub.add_parser("evaluar", help="Clasifica la tarea: docker o firecracker")
    ev.add_argument("descripcion", nargs="+")

    ex = sub.add_parser("ejecutar", help="Arranca el entorno seleccionado")
    ex.add_argument("--entorno", choices=["docker", "firecracker"], default="docker")

    sub.add_parser("status", help="Estado de ambos entornos")

    args = parser.parse_args()

    if args.cmd == "evaluar":
        desc = " ".join(args.descripcion)
        decision = clasificar_tarea(desc)
        print(f"🔍 [CUA] Tarea: {desc}")
        print(f"🎯 [CUA] Entorno elegido: {decision.upper()}")
        print(f"   → {'⚡ FIRECRACKER (aislamiento KVM)' if decision == 'firecracker' else '🐳 DOCKER HEADLESS (por defecto)'}")
        sys.exit(0)

    if args.cmd == "status":
        print("📊 [CUA] Estado de entornos:")
        print(f"  🐳 Docker Headless GUI : {estado_docker()}")
        print(f"  ⚡ Firecracker MicroVM : {estado_firecracker()}")
        sys.exit(0)

    if args.cmd == "ejecutar":
        if args.entorno == "docker":
            ok = arrancar_docker()
        else:
            ok = arrancar_firecracker()
        sys.exit(0 if ok else 1)

    parser.print_help()


if __name__ == "__main__":
    main()
