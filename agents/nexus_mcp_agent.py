import os
import sys
import json
import random
import requests

# 1. RUTA AL PROYECTO GODOT
PROJECT_DIR = os.path.expanduser("~/storage/shared/GodotProjects")

# 2. ZENIT POOL - AGANGA AQUÍ TUS API KEYS DE GOOGLE AI STUDIO
ZENIT_POOL_KEYS = [
    "TU_API_KEY_1",
    "TU_API_KEY_2",
]

def get_zenit_key():
    valid_keys = [k for k in ZENIT_POOL_KEYS if not k.startswith("TU_API_KEY")]
    if not valid_keys:
        print("⚠️ Advertencia: Agrega tus llaves de Zenit Pool dentro de ZENIT_POOL_KEYS en ~/nexus_mcp_agent.py")
        sys.exit(1)
    return random.choice(valid_keys)

def execute_mcp_actions(actions):
    """Aplica los cambios directamente sobre la carpeta del proyecto de Godot."""
    for item in actions:
        file_name = item.get("filename")
        content = item.get("content", "")
        
        target_path = os.path.join(PROJECT_DIR, file_name)
        os.makedirs(os.path.dirname(target_path), exist_ok=True)
        
        with open(target_path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"✨ Archivo creado/actualizado en proyecto: {file_name}")

def ask_zenit_mcp(user_prompt):
    api_key = get_zenit_key()
    url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={api_key}"
    
    existing_files = os.listdir(PROJECT_DIR) if os.path.exists(PROJECT_DIR) else []
    
    system_instruction = f"""
    Eres el motor MCP autónomo de NEXUS para Godot 4.
    Archivos actuales en la raíz del proyecto: {existing_files}.
    Petición del usuario: {user_prompt}
    
    Responde EXCLUSIVAMENTE con un JSON válido con la siguiente estructura:
    {{
        "actions": [
            {{
                "type": "create_script",
                "filename": "nombre_archivo.gd",
                "content": "codigo GDScript completo para Godot 4"
            }}
        ]
    }}
    """
    
    payload = {
        "contents": [{"parts": [{"text": system_instruction}]}],
        "generationConfig": {"response_mime_type": "application/json"}
    }
    
    try:
        print("🧠 Consultando a Zenit Pool (Gemini MCP Engine)...")
        res = requests.post(url, json=payload, headers={"Content-Type": "application/json"}, timeout=15)
        res_data = res.json()
        raw_text = res_data['candidates'][0]['content']['parts'][0]['text']
        mcp_response = json.loads(raw_text)
        
        if "actions" in mcp_response:
            execute_mcp_actions(mcp_response["actions"])
            print("🚀 Cambios aplicados con éxito en la carpeta del juego.")
        else:
            print("⚠️ No se generaron acciones válidas.")
    except Exception as e:
        print(f"❌ Error durante la ejecución MCP: {e}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Uso: python3 ~/nexus_mcp_agent.py 'Tu orden'")
        sys.exit(1)
    ask_zenit_mcp(" ".join(sys.argv[1:]))
