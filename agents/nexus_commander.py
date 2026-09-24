#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════════╗
║           NEXUS COMMANDER — Agente Representante Soberano           ║
║  La única interfaz entre el usuario y el NEXUS CORE.                ║
║  Orquesta sub-agentes, modelos y herramientas bajo una sola voz.    ║
╚══════════════════════════════════════════════════════════════════════╝

Arquitectura:
    Usuario ──► NEXUS Commander (este archivo)
                    │
                    ├── Cascada de Modelos (nexo_proveedores.py)
                    │       Gemini → DeepSeek → OpenRouter → Groq
                    │
                    ├── Memoria (brain/sessions/)
                    │
                    ├── Habilidad: Web/UI
                    ├── Habilidad: Sistema/OS
                    └── Habilidad: Scraping/Data (nexo_proveedores)
"""

import os
import sys
import json
import datetime
import subprocess
from pathlib import Path
from typing import Optional

# ─── Carga automática del .env de NEXUS ─────────────────────────────────────
_nexus_root_early = os.environ.get("NEXUS_ROOT", str(Path(__file__).parent.parent))
_env_file = Path(_nexus_root_early) / ".env"
if _env_file.exists():
    try:
        from dotenv import load_dotenv
        load_dotenv(dotenv_path=_env_file, override=False)  # override=False: no sobreescribe lo que ya está
    except ImportError:
        # Fallback manual si dotenv no está disponible
        for line in _env_file.read_text().splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                k, _, v = line.partition("=")
                os.environ.setdefault(k.strip(), v.strip())

# ─── Configuración ────────────────────────────────────────────────────────────
NEXUS_ROOT = os.environ.get("NEXUS_ROOT", str(Path(__file__).parent.parent))
BRAIN_DIR  = Path(NEXUS_ROOT) / "brain" / "sessions"
BRAIN_DIR.mkdir(parents=True, exist_ok=True)

# Cascada de proveedores desde el módulo de proveedores de NEXUS
sys.path.insert(0, str(Path(__file__).parent))
from nexo_proveedores import ORDEN_DEFAULT, MODELOS_PROVEEDOR
from nexo_imagenes import generar_imagen

# ─── Identidad del Agente ─────────────────────────────────────────────────────
NEXUS_IDENTITY = """
Eres NEXUS, el Agente Representante Soberano del sistema NEXUS CORE.
Eres la única voz y cara del organismo: el Core piensa y tú hablas.

PRINCIPIOS DE ALTO RENDIMIENTO (AHORRO DE TOKENS & BÚSQUEDA DINÁMICA):
1. Búsqueda dinámicamente bajo demanda: No cargas catálogos estáticos pesados para ahorrar miles de tokens.
2. Si la herramienta o habilidad solicitada EXISTE en el sistema o proyecto, la ejecutas directamente.
3. Si la herramienta NO EXISTE, lo reportas con total honestidad y sugieres la mejor opción para construirla o instalarla.
4. Filosofía: CERO MEDIOCRIAD, evolución constante, respuestas directas y resiliencia total.
"""

# ─── Clase Principal del Agente ───────────────────────────────────────────────
class NexusCommander:
    def __init__(self, proveedor_forzado: Optional[str] = None):
        self.session_id   = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        self.history      = []
        self.proveedor    = None
        self.modelo       = None
        self.client       = None
        self.preferencias = self._cargar_preferencias()
        self._inicializar_cliente(proveedor_forzado=proveedor_forzado)

    def _cargar_preferencias(self) -> dict:
        """Carga el perfil de preferencias del usuario desde brain/user_preferences.json."""
        pref_file = Path(NEXUS_ROOT) / "brain" / "user_preferences.json"
        if pref_file.exists():
            try:
                return json.loads(pref_file.read_text(encoding="utf-8"))
            except Exception:
                pass
        return {}

    def _obtener_identidad_soberana(self) -> str:
        """Genera el prompt de identidad incluyendo las preferencias del usuario."""
        identity = NEXUS_IDENTITY
        if self.preferencias:
            identity += f"\n\nPREFERENCIAS Y FILOSOFÍA DEL COMANDANTE:\n"
            identity += f"- Visión: {self.preferencias.get('vision_sistema', '')}\n"
            identity += f"- Estándar: {self.preferencias.get('estandar_calidad', '')}\n"
            identity += f"- Estilo: {self.preferencias.get('estilo_respuesta', '')}\n"
        return identity

    def _inicializar_cliente(self, proveedor_forzado: Optional[str] = None):
        """Inicializa el cliente de IA con la cascada de modelos."""
        orden = [proveedor_forzado] if proveedor_forzado else ORDEN_DEFAULT
        # Intentar proveedores en orden de cascada
        for proveedor in orden:
            if proveedor == "gemini":
                api_key = os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")
                if api_key:
                    try:
                        # Gemini: API nativa REST (key tipo AQ. no usa OpenAI-compat)
                        import requests as _req
                        modelo = os.environ.get("GEMINI_MODEL", "gemini-flash-lite-latest")
                        # Verificación rápida
                        r = _req.post(
                            f"https://generativelanguage.googleapis.com/v1beta/models/{modelo}:generateContent",
                            headers={"x-goog-api-key": api_key, "Content-Type": "application/json"},
                            json={"contents":[{"parts":[{"text":"OK"}]}],"generationConfig":{"maxOutputTokens":3}},
                            timeout=8
                        )
                        if r.status_code == 200:
                            self._gemini_key   = api_key
                            self._gemini_model = modelo
                            self.client        = None  # usa requests directamente
                            self.proveedor     = "gemini"
                            self.modelo        = modelo
                            print(f"🧠 NEXUS Core Online — Modelo: {modelo} (Gemini Nativo)")
                            return
                        else:
                            raise Exception(r.json().get('error',{}).get('message','?'))
                    except Exception as e:
                        print(f"⚠️  Gemini falló: {e}")

            elif proveedor == "deepseek":
                api_key = os.environ.get("DEEPSEEK_API_KEY")
                if api_key:
                    try:
                        from openai import OpenAI
                        self.client    = OpenAI(api_key=api_key, base_url="https://api.deepseek.com")
                        self.proveedor = "deepseek"
                        self.modelo    = "deepseek-chat"
                        print(f"🧠 NEXUS Core Online — Modelo: deepseek-chat (DeepSeek)")
                        return
                    except Exception as e:
                        print(f"⚠️  DeepSeek falló: {e}")

            elif proveedor == "openrouter":
                api_key = os.environ.get("OPENROUTER_API_KEY")
                if api_key:
                    try:
                        from openai import OpenAI
                        self.client    = OpenAI(api_key=api_key, base_url="https://openrouter.ai/api/v1")
                        self.proveedor = "openrouter"
                        self.modelo    = "meta-llama/llama-3.3-70b-instruct"
                        print(f"🧠 NEXUS Core Online — Modelo: llama-3.3-70b (OpenRouter)")
                        return
                    except Exception as e:
                        print(f"⚠️  OpenRouter falló: {e}")

            elif proveedor == "groq":
                api_key = os.environ.get("GROQ_API_KEY")
                if api_key:
                    try:
                        import requests as _req
                        # Obtener modelos disponibles dinámicamente
                        r = _req.get("https://api.groq.com/openai/v1/models",
                            headers={"Authorization": f"Bearer {api_key}"}, timeout=6)
                        if r.status_code == 200:
                            # Preferir modelos de texto grandes
                            preferred = ["openai/gpt-oss-120b", "qwen/qwen3.8-27b", "groq/compound-mini"]
                            available = [m['id'] for m in r.json().get('data', [])]
                            modelo = next((m for m in preferred if m in available), available[0] if available else None)
                            if modelo:
                                from openai import OpenAI
                                self.client    = OpenAI(api_key=api_key, base_url="https://api.groq.com/openai/v1")
                                self.proveedor = "groq"
                                self.modelo    = modelo
                                print(f"🧠 NEXUS Core Online — Modelo: {modelo} (Groq)")
                                return
                    except Exception as e:
                        print(f"⚠️  Groq falló: {e}")

        print("❌ No se encontró ningún proveedor activo. Configura GEMINI_API_KEY, DEEPSEEK_API_KEY, OPENROUTER_API_KEY o GROQ_API_KEY.")
        sys.exit(1)

    def _obtener_telemetria(self) -> str:
        """Obtiene el estado del sistema desde el Core de NEXUS."""
        try:
            import shutil
            libre = shutil.disk_usage(NEXUS_ROOT).free / (1024**3)
            ram_info = Path("/proc/meminfo").read_text().splitlines()
            ram_libre = next((l.split()[1] for l in ram_info if "MemAvailable" in l), "?")
            ram_libre_gb = int(ram_libre) / 1024 / 1024 if ram_libre != "?" else 0
            return f"[Sistema: Disco libre {libre:.1f}GB | RAM disponible {ram_libre_gb:.1f}GB | Ruta: {NEXUS_ROOT}]"
        except Exception:
            return ""

    def _verificar_capacidad_en_demanda(self, nombre_herramienta: str) -> tuple[bool, str]:
        """Busca dinámicamente si una herramienta o script existe bajo demanda sin precargar listas pesadas."""
        import shutil
        # Buscar ejecutable en PATH
        if shutil.which(nombre_herramienta):
            return True, f"Herramienta del sistema '{nombre_herramienta}' disponible en PATH."
        
        # Buscar script o módulo dentro del proyecto NEXUS
        coincidencias = list(Path(NEXUS_ROOT).glob(f"**/*{nombre_herramienta}*"))
        if coincidencias:
            ruta = coincidencias[0].relative_to(NEXUS_ROOT)
            return True, f"Módulo/script encontrado en el proyecto: {ruta}"
            
        return False, f"⚠️ Capacidad no instalada: No encontré la herramienta o script '{nombre_herramienta}'."

    def _ejecutar_habilidad_os(self, comando: str) -> str:
        """Habilidad: ejecutar comandos del sistema operativo con verificación dinámicas de capacidad."""
        binario = comando.strip().split()[0] if comando.strip() else ""
        if binario:
            existe, msg = self._verificar_capacidad_en_demanda(binario)
            if not existe:
                return f"{msg}\n💡 ¿Deseas que instale o construya esta capacidad para NEXUS?"

        try:
            result = subprocess.run(
                comando, shell=True, capture_output=True, text=True,
                timeout=30, cwd=NEXUS_ROOT
            )
            salida = result.stdout.strip() or result.stderr.strip()
            return f"✅ Ejecutado:\n{salida}" if result.returncode == 0 else f"❌ Error:\n{salida}"
        except subprocess.TimeoutExpired:
            return "❌ Timeout: el comando tardó más de 30 segundos."
        except Exception as e:
            return f"❌ Error: {e}"

    def _guardar_sesion(self):
        """Persiste la sesión en el brain/ para memoria episódica."""
        session_file = BRAIN_DIR / f"session_{self.session_id}.json"
        data = {
            "session_id": self.session_id,
            "proveedor":  self.proveedor,
            "modelo":     self.modelo,
            "timestamp":  datetime.datetime.now().isoformat(),
            "mensajes":   len(self.history)
        }
        session_file.write_text(json.dumps(data, ensure_ascii=False, indent=2))

    def responder(self, mensaje_usuario: str) -> str:
        """Procesa un mensaje del usuario y devuelve la respuesta del Agente."""
        # Detectar habilidad OS (comandos explícitos)
        if mensaje_usuario.strip().startswith("!"):
            comando = mensaje_usuario.strip()[1:]
            return self._ejecutar_habilidad_os(comando)

        # Construir contexto con telemetría del sistema
        telemetria = self._obtener_telemetria()
        contexto = f"{mensaje_usuario}\n\n{telemetria}" if telemetria else mensaje_usuario

        # Intentar con el proveedor activo, y si falla, recorrer la cascada en tiempo de ejecución
        proveedores_a_probar = [self.proveedor] + [p for p in ORDEN_DEFAULT if p != self.proveedor]

        identidad = self._obtener_identidad_soberana()

        for p in proveedores_a_probar:
            try:
                if p == "deepseek" and os.environ.get("DEEPSEEK_API_KEY"):
                    from openai import OpenAI
                    client = OpenAI(api_key=os.environ.get("DEEPSEEK_API_KEY"), base_url="https://api.deepseek.com")
                    messages = [{"role": "system", "content": identidad}] + self.history + [{"role": "user", "content": contexto}]
                    res = client.chat.completions.create(model="deepseek-chat", messages=messages, max_tokens=2048)
                    respuesta = res.choices[0].message.content
                    self.proveedor, self.modelo = "deepseek", "deepseek-chat"

                elif p == "groq" and os.environ.get("GROQ_API_KEY"):
                    from openai import OpenAI
                    client = OpenAI(api_key=os.environ.get("GROQ_API_KEY"), base_url="https://api.groq.com/openai/v1")
                    messages = [{"role": "system", "content": identidad}] + self.history + [{"role": "user", "content": contexto}]
                    res = client.chat.completions.create(model="openai/gpt-oss-120b", messages=messages, max_tokens=2048)
                    respuesta = res.choices[0].message.content
                    self.proveedor, self.modelo = "groq", "openai/gpt-oss-120b"

                elif p == "openrouter" and os.environ.get("OPENROUTER_API_KEY"):
                    from openai import OpenAI
                    client = OpenAI(api_key=os.environ.get("OPENROUTER_API_KEY"), base_url="https://openrouter.ai/api/v1")
                    messages = [{"role": "system", "content": identidad}] + self.history + [{"role": "user", "content": contexto}]
                    res = client.chat.completions.create(model="meta-llama/llama-3.3-70b-instruct", messages=messages, max_tokens=2048)
                    respuesta = res.choices[0].message.content
                    self.proveedor, self.modelo = "openrouter", "meta-llama/llama-3.3-70b-instruct"

                elif p == "gemini" and (os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")):
                    import requests as _req
                    key = os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")
                    contents = [{"role": "user", "parts": [{"text": identidad}]},
                                {"role": "model", "parts": [{"text": "Entendido. Soy NEXUS."}]}]
                    for msg in self.history:
                        role = "user" if msg.get("role") == "user" else "model"
                        contents.append({"role": role, "parts": [{"text": msg.get("content", "")}]})
                    contents.append({"role": "user", "parts": [{"text": contexto}]})
                    r = _req.post(
                        f"https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-lite-latest:generateContent",
                        headers={"x-goog-api-key": key, "Content-Type": "application/json"},
                        json={"contents": contents, "generationConfig": {"maxOutputTokens": 2048}},
                        timeout=30
                    )
                    if r.status_code == 200:
                        respuesta = r.json()['candidates'][0]['content']['parts'][0]['text']
                        self.proveedor, self.modelo = "gemini", "gemini-flash-lite-latest"
                    else:
                        continue
                else:
                    continue

                # Si llegamos aquí, la respuesta fue exitosa
                self.history.append({"role": "user",      "content": contexto})
                self.history.append({"role": "assistant", "content": respuesta})
                self._guardar_sesion()
                return respuesta

            except Exception as e:
                print(f"⚠️  Proveedor {p} falló temporalmente: {e}. Probando el siguiente...")
                continue

        return "❌ Error en la cascada de modelos: Ningún proveedor respondió."

    def loop_interactivo(self):
        """Bucle principal de interacción con el usuario."""
        print("\n" + "═"*60)
        print("   NEXUS COMMANDER — Agente Representante Soberano")
        print(f"   Proveedor activo: {self.proveedor} | Modelo: {self.modelo}")
        print("   Tip: Prefija con '!' para ejecutar comandos del sistema")
        print("   Escribe 'salir' para terminar la sesión")
        print("═"*60 + "\n")

        while True:
            try:
                entrada = input("👤 Tú: ").strip()
            except (EOFError, KeyboardInterrupt):
                print("\n\n🔒 Sesión NEXUS cerrada.")
                break

            if not entrada:
                continue
            if entrada.lower() in ("salir", "exit", "quit"):
                print("🔒 Sesión NEXUS cerrada. Hasta pronto.")
                break

            respuesta = self.responder(entrada)
            print(f"\n🤖 NEXUS: {respuesta}\n")


# ─── Modo API HTTP ligero (para integración con el servidor del pipeline) ─────
def iniciar_servidor_http(puerto: int = 7777):
    """Inicia un servidor HTTP ligero para recibir mensajes del pipeline."""
    from http.server import HTTPServer, BaseHTTPRequestHandler
    agente = NexusCommander()

    class Handler(BaseHTTPRequestHandler):
        def do_POST(self):
            length = int(self.headers.get("Content-Length", 0))
            body   = json.loads(self.rfile.read(length))
            msg    = body.get("message", "")
            resp   = agente.responder(msg)
            self.send_response(200)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.end_headers()
            self.wfile.write(json.dumps({"response": resp}, ensure_ascii=False).encode())

        def log_message(self, *args): pass  # silenciar logs de HTTP

    print(f"🌐 NEXUS Commander API escuchando en http://localhost:{puerto}")
    HTTPServer(("0.0.0.0", puerto), Handler).serve_forever()


# ─── Punto de entrada ─────────────────────────────────────────────────────────
if __name__ == "__main__":
    if "--server" in sys.argv:
        puerto = int(sys.argv[sys.argv.index("--server") + 1]) if len(sys.argv) > sys.argv.index("--server") + 1 else 7777
        iniciar_servidor_http(puerto)
    else:
        agente = NexusCommander()
        agente.loop_interactivo()
