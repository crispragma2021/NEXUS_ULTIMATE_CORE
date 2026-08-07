import os
import requests
import google.auth
from google.auth.transport.requests import Request

def test_agent_builder():
    print("🤖 Conectando con tu Agente en Google Cloud (Gemini Flash)...")
    
    # 1. Obtener credenciales configuradas
    credentials, project = google.auth.default()
    credentials.refresh(Request())
    token = credentials.token

    # 2. Configurar el endpoint de la API usando tu Deployment ID
    # El usuario proporcionó: projects/1075726813576/locations/us/apps/4911c883-ac3d-477b-aee4-cbf2e75a8b3d/deployments/9c1fc5c7-de24-4113-8713-83d569cc3624
    url = "https://us-discoveryengine.googleapis.com/v1alpha/projects/1075726813576/locations/us/apps/4911c883-ac3d-477b-aee4-cbf2e75a8b3d/deployments/9c1fc5c7-de24-4113-8713-83d569cc3624:query"
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }

    # 3. La pregunta que le haremos al Agente
    payload = {
        "query": "Hola, ¿puedes confirmar que estás operando con Gemini Flash y decirme de qué eres capaz?"
    }

    print(f"👉 Enviando mensaje: '{payload['query']}'")
    
    # 4. Llamar a la API
    try:
        response = requests.post(url, headers=headers, json=payload)
        
        if response.status_code == 200:
            data = response.json()
            print("\n✅ RESPUESTA DEL AGENTE:")
            # Agent Builder retorna la respuesta normalmente en 'reply' o 'summary'
            if 'reply' in data:
                print(data['reply']['reply'])
            elif 'summary' in data:
                print(data['summary']['summaryText'])
            else:
                print(data)
        else:
            print(f"\n❌ Error {response.status_code}: {response.text}")
    except Exception as e:
        print(f"\n❌ Error de conexión: {e}")

if __name__ == "__main__":
    test_agent_builder()
