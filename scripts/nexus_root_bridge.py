import os
import subprocess
import time

FIFO_PATH = "/home/soberano/NEXUS_ULTIMATE_CORE/data/vault/nexus_root_bridge.fifo"
OUTPUT_PATH = "/home/soberano/NEXUS_ULTIMATE_CORE/data/vault/bridge_output.log"

def main():
    if os.path.exists(FIFO_PATH):
        os.remove(FIFO_PATH)
    
    os.mkfifo(FIFO_PATH)
    os.chmod(FIFO_PATH, 0o666)
    
    print(f"🌉 Puente Soberano V2 (Con Feedback) Activado.")
    print(f"Esperando órdenes en {FIFO_PATH}...")

    while True:
        try:
            with open(FIFO_PATH, "r") as fifo:
                for line in fifo:
                    command = line.strip()
                    if not command:
                        continue
                    
                    print(f"🚀 Ejecutando: {command}")
                    try:
                        result = subprocess.run(command, shell=True, capture_output=True, text=True)
                        with open(OUTPUT_PATH, "w") as out:
                            out.write(f"STDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}")
                        print(f"✅ Salida guardada en {OUTPUT_PATH}")
                    except Exception as e:
                        with open(OUTPUT_PATH, "w") as out:
                            out.write(f"ERROR CRITICO: {e}")
                        print(f"🔥 Fallo: {e}")
        except Exception as e:
            time.sleep(1)

if __name__ == "__main__":
    main()
