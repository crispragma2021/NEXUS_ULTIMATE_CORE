# 🔱 NEXUS OMEGA

NEXUS OMEGA es una plataforma de orquestación de modelos de lenguaje que soporta múltiples proveedores de IA a través de un sistema de cascada.

## Configuración

### Variables de Entorno

#### NEXUS_CASCADE_ORDER

Lista ordenada de proveedores activos, separados por comas. Ejemplo:

```
NEXUS_CASCADE_ORDER=gemini,deepseek,openrouter,groq,github,cerebras,ollama
```

Si no está configurada, el sistema usa el orden por defecto definido en `ORDEN_DEFAULT`.

Los proveedores disponibles son:
- **gemini**: modelos Gemini 3.8 Flash y 3.7 Flash
- **deepseek**: modelo DeepSeek Chat
- **openrouter**: modelo meta-llama/llama-3.3-70b-instruct
- **groq**: modelo openai/gpt-oss-20b
- **github**: modelo gpt-4.1-mini

### ORDEN_DEFAULT

El orden por defecto de proveedores es:

```
ORDEN_DEFAULT = ["gemini", "deepseek", "openrouter", "groq", "github", "cerebras", "ollama"]
```

## Uso

El sistema selecciona proveedores en el orden especificado. Cuando un proveedor falla o no tiene modelos disponibles, pasa al siguiente en la cascada.

## Tests

Ejecutar las pruebas unitarias:

```bash
pytest tests/test_cascada_proveedores.py -v
```