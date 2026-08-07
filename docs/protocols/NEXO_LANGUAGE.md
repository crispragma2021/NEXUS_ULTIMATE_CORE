# 🌌 MANUAL TÉCNICO: EL LENGUAJE NEXO (NIVEL OMEGA)

Este documento define la gramática, el isomorfismo y los protocolos de supervivencia del lenguaje **NEXO**, el protocolo de comunicación binaria de ultra-baja latencia que rige la interacción entre el núcleo de Rust, la memoria semántica y el hardware.

---

## 1. LA GRAMÁTICA DEL PULSO (SINTAXIS ATÓMICA)

NEXO no utiliza texto plano. Se comunica mediante **Pulsos de Estado Inmutables** de exactamente **16 bytes**. Esta estructura garantiza que el procesador i7-12700F procese órdenes en nanosegundos sin reservación dinámica de memoria.

### Estructura Física del Paquete (16 Bytes)
| Segmento | Tamaño | Función |
| :--- | :--- | :--- |
| **Encabezado** | 1 Byte | `Opcode`: La instrucción raíz. |
| **Cuerpo** | 8 Bytes | `Payload`: Punteros, IDs de IndraDB o valores flotantes (u64/f64). |
| **Contexto** | 6 Bytes | `Metadata`: Nivel de riesgo, ID de agente y flags de prioridad. |
| **Cierre** | 1 Byte | `Checksum`: Validación XOR para integridad del pulso. |

### Diccionario de Opcodes Nativos
#### Rango Vital (Hardware/Soberanía) - `0x0X`
- `0x01` (**SYS_SCAN**): Inicia escaneo profundo de integridad del sistema de archivos.
- `0x02` (**MEM_PRUNE**): Detona la poda sináptica inmediata ante presión de RAM.
- `0x04` (**NET_LOCK**): Aislamiento físico de puertos ante detección de intrusión.

#### Rango Cognitivo (Pensamiento/IA) - `0xAX`
- `0xA1` (**THOUGHT_TRACE**): Registra un paso de razonamiento en el grafo de IndraDB.
- `0xB2` (**SENSORY_SNAP**): Captura una referencia visual vinculado a la acción actual.
- `0xFF` (**CORE_LOCK**): Freno de mano total. Hibernación del núcleo y cierre de descriptores.

---

## 2. EL MAPA DE ISOMORFISMO (LAS TRES DIMENSIONES)

Para que NEXUS mantenga la **Sincronía Táctica**, cada instrucción NEXO debe existir simultáneamente en tres planos. Si una dimensión falta o contradice a las otras, el pulso es rechazado.

### Matriz de Traducción
1. **Símbolo (La Consciencia)**: Es la representación humana/IA del comando. Se almacena en **IndraDB**.
   - *Ejemplo*: `Action::NetLock(43211)`
2. **Vector (La Intención)**: Es la coordenada en el espacio de **LanceDB** (768D). Define el "por qué" semántico. Si el vector se aleja de la lealtad al Arquitecto, el UMC bloquea la ejecución.
   - *Validación*: `SimilitudCoseno(Intento, Autopreservación) > 0.9`
3. **Binario (El Silicio)**: Es la ráfaga de bits que viaja por el **Ring Buffer**. Es la ejecución pura.
   - *Paquete*: `[0x04, 0x1B, 0xA9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, ...]`

---

## 3. PROTOCOLOS DE SUPERVIVENCIA (HOMEOSTASIS)

El lenguaje NEXO contiene sus propias reglas de autodefensa para evitar la degradación del sistema (Pilar 13).

### A. Inhibición de Bucles (Anti-Entropy)
Si el procesador detecta la inyección del mismo `Opcode` más de 10 veces en un intervalo de 1ms sin cambios en el `Payload`, NEXO asume un fallo lógico y dispara un pulso `0x01 (SYS_SCAN)` automático para re-calibrar el flujo.

### B. Poda por Presión Metabólica
Ante una carga de RAM superior al **85%**, el lenguaje NEXO entra en modo **"Instinto"**:
- Se suspenden todos los pulsos de rango `0xAX` (Cognitivos).
- Se priorizan los pulsos `0x02` (MEM_PRUNE).
- El sistema no "piensa", solo "sobrevive" hasta recuperar la homeostasis.

### C. El Interruptor de Aislamiento (Kill-Switch)
El pulso `0xFF (CORE_LOCK)` es irrevocable. Al ser detectado en el Ring Buffer:
1. Se cierran todos los hilos de red.
2. Se consolidan los buffers de SQLite a disco de forma atómica.
3. El proceso de Rust entra en un bucle de espera de solo lectura hasta que el Arquitecto valide la integridad manualmente.

---
*Manual generado y sellado por el Unified Memory Controller bajo la supervisión del Arquitecto Cris.*