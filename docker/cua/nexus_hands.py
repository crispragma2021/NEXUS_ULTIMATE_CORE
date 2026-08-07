#!/usr/bin/env python3
"""
🖐️ NEXUS HANDS v1 — Dominio Real del Mouse y Teclado del Sistema (Python)
============================================================================
Permite a NEXUS moverse como humano REAL dentro del entorno CUA (Xvfb :99):
- Control de mouse/teclado vía xdotool (X server con -ac, sin auth)
- Movimiento de mouse con curva Bezier + jitter Perlin (indetectable)
- Tecleo biométrico con distribución normal de delays (Box-Muller)
- Errores tipográficos realistas + correcciones
- OCR de pantalla con tesseract (local, sin API key)
- Sin dependencia de Node ni de API keys — 100% soberano y local

MOTOR: xdotool (backend X11 nativo, funciona contra Xvfb -ac).
Interfaz: CLI + modo JSON para integración con NEXUS.
"""
import json
import math
import os
import random
import subprocess
import sys
import tempfile

# ─── Display ───
DISPLAY = os.environ.get("DISPLAY", ":99")
os.environ["DISPLAY"] = DISPLAY

# PyAutoGUI opcional (solo para screenshot; si Xlib falla, degradamos a xwd/import)
try:
    import pyautogui  # noqa: F401
    HAS_PYAUTOGUI = True
except Exception:  # noqa: BLE001
    HAS_PYAUTOGUI = False


# ═══════════════════════════════════════════════════════════════════════════
# UTILIDADES BIOMÉTRICAS
# ═══════════════════════════════════════════════════════════════════════════

def gaussian(mean=0.0, std=1.0):
    """Distribución normal (Box-Muller)."""
    u = random.random()
    v = random.random()
    z = math.sqrt(-2.0 * math.log(u)) * math.cos(2.0 * math.pi * v)
    return z * std + mean


def key_delay():
    """Delay entre teclas: media 80ms, std 25ms, clamp [30, 200]."""
    return max(30, min(200, round(gaussian(80, 25))))


def bezier_curve(x0, y0, x1, y1, steps=40):
    """Curva Bezier cúbica + jitter Perlin (más jitter en el medio)."""
    points = []
    dist = math.hypot(x1 - x0, y1 - y0)
    offset = dist * 0.25
    mx, my = (x0 + x1) / 2, (y0 + y1) / 2

    cp1 = (mx + (random.random() - 0.5) * offset,
           my + (random.random() - 0.5) * offset)
    cp2 = (mx + (random.random() - 0.5) * offset * 0.7,
           my + (random.random() - 0.5) * offset * 0.7)

    for i in range(steps + 1):
        t = i / steps
        u = 1 - t
        tt = t * t
        uu = u * u
        uuu = uu * u
        ttt = tt * t
        x = uuu * x0 + 3 * uu * t * cp1[0] + 3 * u * tt * cp2[0] + ttt * x1
        y = uuu * y0 + 3 * uu * t * cp1[1] + 3 * u * tt * cp2[1] + ttt * y1
        jf = math.sin(t * math.pi) * 2.5
        ang = random.random() * math.pi * 2
        x += math.cos(ang) * jf
        y += math.sin(ang) * jf
        points.append((round(x), round(y)))
    return points


# ═══════════════════════════════════════════════════════════════════════════
# MOTOR xdotool
# ═══════════════════════════════════════════════════════════════════════════

def xdo(args):
    """Ejecuta xdotool y devuelve stdout."""
    env = {**os.environ, "DISPLAY": DISPLAY}
    try:
        return subprocess.run(
            ["xdotool"] + args, capture_output=True, text=True,
            timeout=20, env=env,
        ).stdout.strip()
    except Exception:  # noqa: BLE001
        return ""


def screen_size():
    out = xdo(["getdisplaygeometry"])
    parts = out.split()
    return (int(parts[0]), int(parts[1])) if len(parts) >= 2 else (1920, 1080)


def mouse_pos():
    out = xdo(["getmouselocation", "--shell"])
    x = y = 0
    for line in out.splitlines():
        if line.startswith("X="):
            x = int(line.split("=")[1])
        elif line.startswith("Y="):
            y = int(line.split("=")[1])
    return (x, y)


# ═══════════════════════════════════════════════════════════════════════════
# MOUSE
# ═══════════════════════════════════════════════════════════════════════════

def mouse_move(to_x, to_y, duration_ms=None):
    """Mueve el mouse con trayectoria humana (Bezier + jitter)."""
    x0, y0 = mouse_pos()
    duration = duration_ms or random.randint(300, 600)
    steps = max(15, duration // 15)
    points = bezier_curve(x0, y0, to_x, to_y, steps)

    for px, py in points:
        xdo(["mousemove", str(px), str(py)])
        import time as _t
        _t.sleep(max(0.005, gaussian(0.012, 0.004)))
    return {"ok": True, "x": to_x, "y": to_y}


def mouse_click(button=1, times=1):
    """Clic con delay de decisión humana previo."""
    import time as _t
    btn_map = {1: "1", 2: "2", 3: "3"}
    btn = btn_map.get(button, "1")
    _t.sleep(random.uniform(0.08, 0.23))
    for _ in range(times):
        xdo(["click", btn])
        _t.sleep(random.uniform(0.05, 0.15))
    _t.sleep(random.uniform(0.05, 0.15))
    return {"ok": True, "button": btn, "times": times}


def click_at(x, y, button=1):
    """Click en coordenada con movimiento biométrico previo."""
    mouse_move(x, y)
    import time as _t
    _t.sleep(random.uniform(0.06, 0.18))
    mouse_click(button)
    return {"ok": True, "x": x, "y": y}


def scroll(distance, duration_ms=None):
    """Scroll con inercia (ease-in-out)."""
    import time as _t
    duration = duration_ms or random.randint(500, 1000)
    steps = max(8, duration // 30)
    direction = 1 if distance > 0 else -1
    total = abs(distance)

    for i in range(steps):
        t = i / steps
        eased = 2 * t * t if t < 0.5 else -1 + (4 - 2 * t) * t
        prev_t = (i - 1) / steps if i > 0 else 0
        prev_eased = 2 * prev_t * prev_t if prev_t < 0.5 else -1 + (4 - 2 * prev_t) * prev_t
        step_dist = int((eased - prev_eased) * total * direction)
        if step_dist > 0:
            xdo(["click", "5"])
        elif step_dist < 0:
            xdo(["click", "4"])
        _t.sleep(max(0.01, gaussian(0.03, 0.01)))
    return {"ok": True, "distance": distance}


# ═══════════════════════════════════════════════════════════════════════════
# TECLADO
# ═══════════════════════════════════════════════════════════════════════════

# Mapa de caracteres que requieren Shift (US layout)
_SHIFT_MAP = {
    "!": "1", "@": "2", "#": "3", "$": "4", "%": "5",
    "^": "6", "&": "7", "*": "8", "(": "9", ")": "0",
    "_": "minus", "+": "equal", "{": "bracketleft", "}": "bracketright",
    "|": "backslash", ":": "semicolon", '"': "apostrophe",
    "<": "comma", ">": "period", "?": "slash", "~": "grave",
}


def _type_char(char):
    """Teclea un carácter individual vía xdotool (escapa correctamente)."""
    import time as _t
    if char == " ":
        xdo(["key", "space"])
    elif char == "\n":
        xdo(["key", "Return"])
    elif char == "\t":
        xdo(["key", "Tab"])
    elif char in _SHIFT_MAP:
        xdo(["key", "shift+" + _SHIFT_MAP[char]])
    else:
        # xdotool type con delay 0; escribimos el char escapado
        xdo(["type", "--delay", "0", char])
    _t.sleep(key_delay() / 1000.0)


def type_text(text, clear_first=True, tab_to_focus=False):
    """Teclea texto simulando digitación humana real (nunca pega)."""
    import time as _t
    if tab_to_focus:
        _t.sleep(random.uniform(0.05, 0.15))
        xdo(["key", "Tab"])
        _t.sleep(random.uniform(0.08, 0.23))

    if clear_first:
        xdo(["key", "ctrl+a"])
        _t.sleep(random.uniform(0.05, 0.15))
        xdo(["key", "Delete"])
        _t.sleep(random.uniform(0.05, 0.16))

    errors = 0
    for i, char in enumerate(text):
        # Error tipográfico realista (2%, máx 2 por campo)
        if random.random() < 0.02 and i > 2 and errors < 2:
            errors += 1
            wrong = chr(ord(char) + random.choice([1, -1]))
            _type_char(wrong)
            _t.sleep(key_delay() * 1.5 / 1000.0)
            xdo(["key", "BackSpace"])
            _t.sleep(key_delay() * 1.2 / 1000.0)

        _type_char(char)

    return {"ok": True, "length": len(text), "errors": errors}


def press_keys(combo):
    """Pulsa una combinación de teclas (ej: ctrl+alt+t)."""
    keys = [k.strip() for k in combo.split("+")]
    if len(keys) == 1:
        xdo(["key", keys[0]])
    else:
        xdo(["key", "+".join(keys)])
    return {"ok": True, "combo": combo}


# ═══════════════════════════════════════════════════════════════════════════
# SCREEN — Captura + OCR
# ═══════════════════════════════════════════════════════════════════════════

def _capture_import(path, region):
    cmd = ["import", "-window", "root"]
    if region:
        x, y, w, h = region.get("x", 0), region.get("y", 0), region.get("w", 0), region.get("h", 0)
        cmd += ["-crop", f"{w}x{h}+{x}+{y}"]
    cmd.append(path)
    return subprocess.run(cmd, env={**os.environ, "DISPLAY": DISPLAY},
                          capture_output=True, timeout=20)


def _capture_xwd(path, region):
    env = {**os.environ, "DISPLAY": DISPLAY}
    if region:
        x, y, w, h = region.get("x", 0), region.get("y", 0), region.get("w", 0), region.get("h", 0)
        raw = subprocess.run(["xwd", "-root", "-silent"], env=env,
                             capture_output=True, timeout=20).stdout
        # convertir XWD → PNG y recortar con ImageMagick si está, si no devolver XWD crudo
        if raw:
            with open(path, "wb") as f:
                f.write(raw)
            subprocess.run(["convert", path, "-crop", f"{w}x{h}+{x}+{y}", path],
                           env=env, capture_output=True, timeout=20)
    else:
        raw = subprocess.run(["xwd", "-root", "-silent"], env=env,
                             capture_output=True, timeout=20).stdout
        if raw:
            with open(path, "wb") as f:
                f.write(raw)
    return None


def _capture_scrot(path, region):
    cmd = ["scrot", "-z"]
    if region:
        x, y, w, h = region.get("x", 0), region.get("y", 0), region.get("w", 0), region.get("h", 0)
        cmd += ["-a", f"{x},{y},{w},{h}"]
    cmd.append(path)
    return subprocess.run(cmd, env={**os.environ, "DISPLAY": DISPLAY},
                          capture_output=True, timeout=20)


def screenshot(out_path=None, region=None):
    """Captura pantalla (o región) a PNG. Encadena motores: import → xwd → scrot."""
    import time as _t
    path = out_path or os.path.join(tempfile.gettempdir(), f"nexus_shot_{int(_t.time()*1000)}.png")

    import shutil
    if shutil.which("import"):
        _capture_import(path, region)
    if not os.path.exists(path) and shutil.which("xwd"):
        _capture_xwd(path, region)
    if not os.path.exists(path) and shutil.which("scrot"):
        _capture_scrot(path, region)

    if not os.path.exists(path) or os.path.getsize(path) == 0:
        return {"ok": False, "error": "no se pudo capturar pantalla (sin import/xwd/scrot)"}
    return {"ok": True, "path": path}


def ocr_screen(region=None):
    """Extrae texto de la pantalla vía tesseract (local, sin API key)."""
    shot = screenshot(region=region)
    if not shot["ok"]:
        return shot
    try:
        result = subprocess.run(
            ["tesseract", shot["path"], "stdout", "--psm", "6"],
            capture_output=True, text=True, timeout=30,
        )
        text = result.stdout.strip()
    finally:
        try:
            os.unlink(shot["path"])
        except OSError:
            pass
    return {"ok": True, "text": text}


# ═══════════════════════════════════════════════════════════════════════════
# DISPATCH
# ═══════════════════════════════════════════════════════════════════════════

def handle(action, params):
    if action == "pos":
        x, y = mouse_pos()
        return {"ok": True, "x": x, "y": y}
    if action == "size":
        w, h = screen_size()
        return {"ok": True, "width": w, "height": h}
    if action == "move":
        return mouse_move(params.get("x", 0), params.get("y", 0), params.get("duration"))
    if action == "click":
        if params.get("x") is not None:
            return click_at(params["x"], params["y"], params.get("button", 1))
        return mouse_click(params.get("button", 1), params.get("times", 1))
    if action == "type":
        return type_text(params.get("text", ""), params.get("clearFirst", True), params.get("tabToFocus", False))
    if action == "key":
        return press_keys(params.get("combo", ""))
    if action == "scroll":
        return scroll(params.get("distance", 0), params.get("duration"))
    if action == "screenshot":
        return screenshot(params.get("path"), params.get("region"))
    if action == "ocr":
        return ocr_screen(params.get("region"))
    if action == "display":
        return {"ok": True, "display": DISPLAY}
    return {"ok": False, "error": f"acción desconocida: {action}"}


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "json":
        try:
            data = json.loads(sys.argv[2])
            result = handle(data.get("action"), data.get("params", {}))
        except Exception as e:  # noqa: BLE001
            result = {"ok": False, "error": str(e)}
        print(json.dumps(result, ensure_ascii=False))
        return

    # CLI simple
    if len(sys.argv) < 2:
        print("""
🖐️ NEXUS HANDS v1 — Dominio Real del Input (Python + xdotool)
Uso:
  python3 nexus_hands.py size                    → resolución del display
  python3 nexus_hands.py pos                     → posición del cursor
  python3 nexus_hands.py move <x> <y>            → mover mouse (biométrico)
  python3 nexus_hands.py click [1|2|3]           → clic (izq/medio/der)
  python3 nexus_hands.py type "<texto>"          → teclear (biométrico)
  python3 nexus_hands.py ocr                     → OCR de pantalla (tesseract)
  python3 nexus_hands.py json '{"action":"...","params":{...}}'
""")
        return

    cmd = sys.argv[1]
    if cmd == "size":
        w, h = screen_size()
        print(f"{w}x{h}")
    elif cmd == "pos":
        x, y = mouse_pos()
        print(json.dumps({"x": x, "y": y}))
    elif cmd == "move":
        print(json.dumps(mouse_move(int(sys.argv[2]), int(sys.argv[3]))))
    elif cmd == "click":
        print(json.dumps(mouse_click(int(sys.argv[2]) if len(sys.argv) > 2 else 1)))
    elif cmd == "type":
        print(json.dumps(type_text(" ".join(sys.argv[2:]))))
    elif cmd == "screenshot":
        r = screenshot()
        print(json.dumps(r))
    elif cmd == "ocr":
        print(json.dumps(ocr_screen()))
    elif cmd == "display":
        print(DISPLAY)
    else:
        print(f"comando desconocido: {cmd}")


if __name__ == "__main__":
    main()
