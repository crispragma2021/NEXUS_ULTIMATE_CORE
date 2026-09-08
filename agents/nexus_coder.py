import socket
import json
import requests
import random
import sys
# ==========================================
# 1. ZENIT POOL - CONFIGURACIÓN DE CLAVES
# ==========================================
# Agrega aquí tus API Keys de Google AI Studio:
ZENIT_POOL_KEYS = [
    "AIzaSyDkOKdbfFd04QYZotex9kpY3UL51mUF4B8",


    "sk-edd3c54c6a57488fbfda01a43a6b6b98",
    # Agrega el resto de tus 13 claves aquí...
]

def get_zenit_key():
    """Selecciona una clave de Zenit Pool para evitar rate-limits."""
    if not ZENIT_POOL_KEYS or ZENIT_POOL_KEYS[0] == "TU_API_KEY_1":
        print("⚠️ Advertencia: Reemplaza 'TU_API_KEY_1' en nexus_coder.py con tus llaves reales.")
        sys.exit(1)
    return random.choice(ZENIT_POOL_KEYS)

# ==========================================
# 2. GENERADOR DE CÓDIGO CON GEMINI API
# ==========================================
def ask_gemini(prompt_text):
    api_key = get_zenit_key()
    url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={api_key}"
    
    headers = {"Content-Type": "application/json"}
    payload = {
        "contents": [{
            "parts": [{
                "text": f"Eres un asistente experto en Godot 4 (GDScript). Genera únicamente un objeto JSON con el comando a ejecutar en el juego. Estructura esperada: {{\"action\": \"create_node\", \"type\": \"Sprite2D\", \"name\": \"Jugador\"}}. Petición: {prompt_text}"
            }]
        }]
    }
    
    try:
        response = requests.post(url, json=payload, headers=headers, timeout=10)
        response.json()
        # Para esta demo, enviamos un comando formateado
        return {"action": "eval", "code": prompt_text}
    except Exception as e:
        print(f"❌ Error al consultar Zenit Pool / Gemini API: {e}")
        return None

# ==========================================
# 3. ENVIAR COMANDO A GODOT VIA TCP
# ==========================================
def send_to_godot(command_dict, host="127.0.0.1", port=8900):
    print(f"🧠 Conectando con Godot en {host}:{port}...")
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.settimeout(5)
        s.connect((host, port))
        
        # Convertir a cadena JSON y enviar
        json_data = json.dumps(command_dict) + "\n"
        s.sendall(json_data.encode('utf-8'))
        
        # Leer respuesta de Godot
        response = s.recv(1024).decode('utf-8')
        print(f"📥 Respuesta de Godot: {response.strip()}")
        s.close()
        return True
    except Exception as e:
        print(f"❌ Error de conexión con Godot: {e}")
        return False

# ==========================================
# 4. EJECUCIÓN PRINCIPAL
# ==========================================
if __name__ == "__main__":
    prompt = "Crea un nodo de prueba en la escena activa"
    if len(sys.argv) > 1:
        prompt = " ".join(sys.argv[1:])
        
    print(f"🚀 Procesando orden: '{prompt}'")
    
    # 1. Comando directo a enviar
    payload = {
        "action": "log",
        "message": f"Orden recibida desde Termux: {prompt}"
    }
    
    # 2. Despachar a Godot
    send_to_godot(payload)
