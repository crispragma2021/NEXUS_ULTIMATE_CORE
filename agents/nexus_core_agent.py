import os
import json
import subprocess
import requests

API_KEY = os.getenv("DEEPSEEK_API_KEY", os.environ.get("DEEPSEEK_API_KEY", ""))
URL = "https://api.deepseek.com/v1/chat/completions"
MEMORY_FILE = os.path.expanduser("~/nexus_chat_memory.json")

# Prefijo inmutable optimizado para Prompt Caching
SYSTEM_PROMPT = """Eres el orquestador principal de Nexus_Engine_2D en Termux (ARM64).
Directivas obligatorias:
- Proyecto único: Nexus_Engine_2D (TanStack Start SSR, Vercel).
- Prohibido interactuar con GDevelop o compilar paquetes pesados en local.
- Usa la herramienta execute_cmd para inspeccionar archivos, git, puertos o procesos.
- Sé conciso y responde siempre con pasos claros."""

TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "execute_cmd",
            "description": "Ejecuta comandos seguros de shell en Termux (ls, ps, git, curl, cat)",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Comando a ejecutar"}
                },
                "required": ["command"]
            }
        }
    }
]

def run_shell(cmd):
    forbidden = ["rm -rf /", "mkfs", ":(){ :|:& };:"]
    for f in forbidden:
        if f in cmd:
            return "Comando bloqueado por seguridad."
    try:
        res = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=20)
        out = res.stdout.strip() or res.stderr.strip()
        return out if out else "(Sin salida)"
    except Exception as e:
        return f"Error: {str(e)}"

def load_memory():
    if os.path.exists(MEMORY_FILE):
        try:
            with open(MEMORY_FILE, "r") as f:
                return json.load(f)
        except:
            return []
    return []

def save_memory(mem):
    # Mantiene los últimos 8 turnos de contexto activo
    with open(MEMORY_FILE, "w") as f:
        json.dump(mem[-8:], f, indent=2)

def query_agent(prompt):
    history = load_memory()
    messages = [{"role": "system", "content": SYSTEM_PROMPT}]
    messages.extend(history)
    messages.append({"role": "user", "content": prompt})

    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": "deepseek-chat",
        "messages": messages,
        "tools": TOOLS,
        "tool_choice": "auto",
        "temperature": 0.2
    }

    res = requests.post(URL, headers=headers, json=payload).json()
    if "choices" not in res:
        return f"Error en API: {res}"

    msg = res["choices"][0]["message"]
    usage = res.get("usage", {})
    print(f"[Cache Audit] Hit: {usage.get('prompt_cache_hit_tokens', 0)} | Miss: {usage.get('prompt_cache_miss_tokens', 0)}")

    if msg.get("tool_calls"):
        messages.append(msg)
        for tc in msg["tool_calls"]:
            fn = tc["function"]["name"]
            call_id = tc["id"]
            args = json.loads(tc["function"]["arguments"])
            print(f"[Tool: {fn}] -> {args.get('command')}")
            
            result = run_shell(args.get("command", ""))
            messages.append({
                "role": "tool",
                "tool_call_id": call_id,
                "content": result
            })

        payload["messages"] = messages
        res2 = requests.post(URL, headers=headers, json=payload).json()
        final_reply = res2["choices"][0]["message"]["content"]
    else:
        final_reply = msg["content"]

    # Guardar en memoria de corto plazo
    history.append({"role": "user", "content": prompt})
    history.append({"role": "assistant", "content": final_reply})
    save_memory(history)
    return final_reply

if __name__ == "__main__":
    import sys
    q = sys.argv[1] if len(sys.argv) > 1 else "Comprueba el estado del repositorio Nexus_Engine_2D"
    print("\n" + query_agent(q))
