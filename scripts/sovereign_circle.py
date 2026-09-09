import time
import math
from evdev import UInput, ecodes as e

# 🧬 NEXUS OMEGA - Nervio Motor Digital (uinput)
# Replicación del gesto físico para dibujar un círculo.

def dibujar_circulo_soberano():
    # Capacidades del ratón: Ejes Relativos + Botón Izquierdo
    cap = {
        e.EV_REL: (e.REL_X, e.REL_Y),
        e.EV_KEY: (e.BTN_LEFT, e.BTN_RIGHT)
    }

    print("🦾 [SNC] Despertando Nervio Motor en /dev/uinput...")
    
    try:
        with UInput(cap, name='NEXUS-OMEGA-HAND', version=0x1) as ui:
            # 1. Bajando el dedo (Click)
            print("🦾 [SNC] Dedo DIGITAL bajando (MouseDown)...")
            ui.write(e.EV_KEY, e.BTN_LEFT, 1)
            ui.syn()
            time.sleep(0.5)

            # 2. Trazando el Círculo (Matemática OMEGA)
            steps = 40
            radius = 15
            for i in range(steps + 1):
                angle = (2 * math.pi * i) / steps
                # Diferencial de movimiento para simular velocidad constante
                # dx = r * (cos(a) - cos(a-da))
                # dy = r * (sin(a) - sin(a-da))
                # Pero REL_X/REL_Y son diferenciales acumulados en cada syn.
                
                dx = int(radius * math.cos(angle))
                dy = int(radius * math.sin(angle))
                
                ui.write(e.EV_REL, e.REL_X, dx)
                ui.write(e.EV_REL, e.REL_Y, dy)
                ui.syn()
                time.sleep(0.05)

            # 3. Levantando el dedo (Release)
            time.sleep(0.5)
            ui.write(e.EV_KEY, e.BTN_LEFT, 0)
            ui.syn()
            print("✅ [SNC] Círculo OMEGA completado con éxito.")

    except Exception as ex:
        print(f"⚠️ [SNC] Error en la inyección de ráfaga táctil: {ex}")

if __name__ == "__main__":
    dibujar_circulo_soberano()
