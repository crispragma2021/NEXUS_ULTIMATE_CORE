# Cambio de Orden en la Cascada de Proveedores NEXUS

## Resumen

Este cambio modifica el orden por defecto de selección de proveedores en NEXUS y agrega soporte para la variable de entorno `NEXUS_CASCADE_ORDER` para restringir la lista exacta de proveedores activos.

## Cambios Realizados

### 1. Modificación de `agents/nexo_proveedores.py`

- **ORDEN_DEFAULT** actualizada a: `["gemini", "deepseek", "openrouter", "groq", "github", "cerebras", "ollama"]`

- **Modelos por proveedor definidos:**
  - `gemini`: `["gemini-3.8-flash", "gemini-3.7-flash"]`
  - `deepseek`: `["deepseek-chat"]`
  - `openrouter`: `["meta-llama/llama-3.3-70b-instruct"]`
  - `groq`: `["openai/gpt-oss-20b"]`
  - `github`: `["gpt-4.1-mini"]`
  - `cerebras`: `[]` (vacío, proveedor soportado pero sin modelos definidos)
  - `ollama`: `[]` (vacío, proveedor soportado pero sin modelos definidos)

- **Soporte para `NEXUS_CASCADE_ORDER`**: Variable de entorno que permite restringir la lista exacta de proveedores activos. Si está definida, sobrescribe el `ORDEN_DEFAULT`.

- **Guarda `RuntimeError`**: Si `ORDEN_DEFAULT` nombra proveedores no registrados, se lanza un error en tiempo de importación.

### 2. Actualización de `tests/test_cascada_proveedores.py`

- 70 pruebas unitarias que validan:
  - El orden por defecto de 7 proveedores
  - Los modelos disponibles por cada proveedor
  - La función `obtener_modelos()` con y sin parámetros de proveedor u orden
  - La variable de entorno `NEXUS_CASCADE_ORDER` para restringir proveedores
  - Validación de `RuntimeError` para proveedores no registrados
  - Órdenes personalizadas y combinaciones de proveedores
  - Pruebas parametrizadas con 19 escenarios de orden diferentes

### 3. Actualización de `.env.example`

- Agregada la variable `NEXUS_CASCADE_ORDER` con descripción de su formato y uso.

### 4. Creación de `README.md`

- Documentación del sistema NEXUS OMEGA
- Documentación de la variable `NEXUS_CASCADE_ORDER`
- Instrucciones para ejecutar las pruebas

## Uso de NEXUS_CASCADE_ORDER

Para restringir la cascada a proveedores específicos:

```bash
export NEXUS_CASCADE_ORDER="gemini,deepseek,openrouter"
```

Esto limitará la selección solo a esos tres proveedores, en el orden especificado, ignorando el `ORDEN_DEFAULT`.

## Ejemplos

```bash
# Usar orden por defecto
python -c "from agents.nexo_proveedores import obtener_modelos; print(obtener_modelos())"

# Usar orden personalizado
python -c "from agents.nexo_proveedores import obtener_modelos; print(obtener_modelos(orden=['groq', 'github']))"

# Usar variable de entorno
export NEXUS_CASCADE_ORDER="gemini,groq"
python -c "from agents.nexo_proveedores import obtener_modelos; print(obtener_modelos())"
```