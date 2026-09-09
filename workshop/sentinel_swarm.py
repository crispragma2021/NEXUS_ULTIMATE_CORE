import requests
import os
import json
import time
from datetime import datetime
from dotenv import load_dotenv

# NEXUS SENTINEL SWARM - Prioritized Intelligence
# Autor: NEXUS (Hacker Mode)

load_dotenv()

# Prioridad: Los mejores siempre primero
MODELS = [
    "google/gemini-2.0-flash-001",
    "meta-llama/llama-3.1-8b-instruct:free",
    "mistralai/mistral-7b-instruct-v0.3:free",
    "google/gemma-2-9b-it:free"
]

API_KEY = os.getenv("NEXUS_LOCAL_KEY") # Usar llave local para pasar por el proxy

class SentinelAgent:
    def __init__(self):
        self.api_key = API_KEY
        self.headers = {
            "Authorization": f"Bearer {self.api_key}",
            "HTTP-Referer": "https://nexus.autosasistente.app",
            "X-Title": "NEXUS Sovereign Core"
        }

    def infer(self, prompt):
        """Inferencia en enjambre priorizando los mejores modelos primero."""
        for model in MODELS:
            try:
                print(f"🕵️ [SENTINEL] Consultando Oráculo: {model}...")
                # Instrucción combinada para que NEXUS respete el formato
                full_prompt = f"NEXUS: Actúa como Sentinel de análisis. Analiza esta noticia y responde EXCLUSIVAMENTE con un objeto JSON (sin texto adicional) que tenga las llaves 'score' (float entre -1 y 1) y 'razon' (string). Noticia: {prompt}"
                
                response = requests.post(
                    "http://localhost:4444/v1/chat/completions",
                    headers=self.headers,
                    json={
                        "model": model,
                        "messages": [
                            {"role": "user", "content": full_prompt}
                        ]
                    },
                    timeout=40
                )
                if response.status_code == 200:
                    data = response.json()
                    content = data['choices'][0]['message']['content']
                    print(f"📥 [DEBUG] Respuesta cruda: {content[:100]}...")
                    
                    # Intento de extracción robusta de JSON
                    try:
                        # Buscar primer { y último }
                        start = content.find('{')
                        end = content.rfind('}') + 1
                        if start != -1 and end != 0:
                            content = content[start:end]
                        
                        parsed = json.loads(content)
                        if 'score' in parsed:
                            return parsed
                        else:
                            # Fallback si no hay score
                            return {"score": 0.5, "razon": "NEXUS respondió sin score formal: " + content[:50]}
                    except:
                        print(f"⚠️ [WARN] Error de parseo JSON en {model}")
                        continue
                else:
                    print(f"⚠️ [WARN] {model} respondió con error {response.status_code}")
            except Exception as e:
                print(f"❌ [ERROR] Fallo en {model}: {str(e)}")
                continue
        return {"score": 0.0, "razon": "Error total en el enjambre."}

    def scan_alpha(self, query):
        """Simulación de captura de Alpha (Se integrará con web_search.sh)"""
        print(f"📡 [SCAN] Escaneando Alpha para: {query}...")
        # Aquí el Sentinel capturaría datos reales de Twitter/Telegram
        # Por ahora, simulamos una captura de noticia de impacto
        mock_news = f"Gran anuncio detectado sobre {query}: Adopción institucional masiva confirmada para el próximo trimestre."
        
        result = self.infer(mock_news)
        
        # Guardar señal para el motor de Rust (nexus-tr)
        signal_file = "/home/soberano/NEXUS_ULTIMATE_CORE/data/sentinel_alpha.json"
        with open(signal_file, "w") as f:
            json.dump({
                "timestamp": datetime.now().isoformat(),
                "query": query,
                "alpha": result
            }, f)
        
        print(f"✅ [ALPHA DETECTED] Score: {result['score']} | {result['razon']}")

if __name__ == "__main__":
    agent = SentinelAgent()
    while True:
        agent.scan_alpha("Solana (SOL)")
        time.sleep(60) # Escaneo cada minuto
