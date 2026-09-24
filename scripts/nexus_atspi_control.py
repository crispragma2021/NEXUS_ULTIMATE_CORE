#!/usr/bin/env python3
"""
NEXUS Core - Motor de Percepción Semántica y Control AT-SPI2 / uinput
Permite al agente NEXUS inspeccionar la interfaz gráfica de cualquier aplicación
de Linux (GNOME, Chrome, GTK, Qt, Electron) vía el árbol de accesibilidad AT-SPI2
y ejecutar acciones semánticas (clics, texto, foco) de forma 100% precisa.
"""

import sys
import time
import os
import subprocess

try:
    import pyatspi
except ImportError:
    print("[ERROR] python3-pyatspi no está instalado.")
    sys.exit(1)

def get_desktop():
    reg = pyatspi.Registry
    return reg.getDesktop(0)

def find_elements(root, name_filter=None, role_filter=None):
    results = []
    if not root:
        return results
    
    def _traverse(node):
        if not node:
            return
        try:
            name = node.name or ""
            role = node.getRoleName() or ""
            
            match_name = (name_filter.lower() in name.lower()) if name_filter else True
            match_role = (role_filter.lower() in role.lower()) if role_filter else True
            
            if match_name and match_role and (name or role):
                bbox = None
                try:
                    comp = node.queryComponent()
                    rect = comp.getExtents(pyatspi.DESKTOP_COORDS)
                    bbox = (rect.x, rect.y, rect.width, rect.height)
                except:
                    pass
                results.append({"element": node, "name": name, "role": role, "bbox": bbox})
                
            for child in node:
                _traverse(child)
        except Exception:
            pass
            
    _traverse(root)
    return results

def find_app(app_name):
    desktop = get_desktop()
    for app in desktop:
        if app and app_name.lower() in (app.name or "").lower():
            return app
    return None

def click_element_by_name(app_name, element_name):
    app = find_app(app_name)
    if not app:
        print(f"[ERROR] Aplicación '{app_name}' no encontrada en el árbol AT-SPI2.")
        return False
        
    elements = find_elements(app, name_filter=element_name)
    if not elements:
        print(f"[ERROR] Elemento '{element_name}' no encontrado en '{app_name}'.")
        return False
        
    target = elements[0]
    el = target["element"]
    print(f"[AT-SPI2] Ejecutando acción en '{target['name']}' ({target['role']})...")
    
    # Intentar doAction primero (accion semántica directa)
    try:
        act = el.queryAction()
        if act.nActions > 0:
            act.doAction(0)
            print(f"[EXITO] Acción '{act.getName(0)}' ejecutada exitosamente vía AT-SPI2 DBus.")
            return True
    except Exception as e:
        print(f"[WARN] doAction falló: {e}. Intentando clic en coordenadas físicas...")
        
    # Fallback a coordenadas físicas con ydotool / xdotool
    if target["bbox"]:
        x, y, w, h = target["bbox"]
        cx = x + w // 2
        cy = y + h // 2
        print(f"[AT-SPI2] Coordenadas encontradas: ({cx}, {cy})")
        subprocess.run(["DISPLAY=:0 xdotool mousemove %d %d click 1" % (cx, cy)], shell=True)
        return True

    return False

def list_app_tree(app_name):
    app = find_app(app_name)
    if not app:
        print(f"[ERROR] Aplicación '{app_name}' no encontrada.")
        return
        
    print(f"=== ÁRBOL DE ACCESIBILIDAD PARA: {app.name} ===")
    elements = find_elements(app)
    for el in elements:
        bbox_str = f" Coordenadas: {el['bbox']}" if el['bbox'] else ""
        print(f"- Elemento: '{el['name']}' | Rol: {el['role']}{bbox_str}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Uso: nexus_atspi_control.py {list <app> | click <app> <elemento>}")
        sys.exit(1)
        
    cmd = sys.argv[1]
    if cmd == "list" and len(sys.argv) >= 3:
        list_app_tree(sys.argv[2])
    elif cmd == "click" and len(sys.argv) >= 4:
        click_element_by_name(sys.argv[2], sys.argv[3])
    else:
        print("Comando no reconocido.")
