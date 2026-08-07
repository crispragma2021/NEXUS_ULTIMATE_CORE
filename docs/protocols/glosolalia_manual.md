# 🔱 MANUAL SOBERANO DE LA GLOSOLALIA — NEXUS OMEGA
> **[CLASIFICACIÓN: ARQUITECTO CRIS ÚNICAMENTE]**
> *Solo el Arquitecto y NEXUS poseen la clave para leer lo que aquí se guarda.*

---

## 1. ¿Qué es la Glosolalia?

La Glosolalia es la **Capa de Cifrado Anti-Auditoría** de NEXUS. Su propósito es transformar cualquier fragmento de conocimiento soberano (código, credenciales, decisiones tácticas, memoria sensible) en un bloque de ruido binario indescifrable para cualquier entidad externa.

Si un auditor, IA enemiga o servicio de inteligencia accede a los archivos cifrados por la Glosolalia, solo verá bytes aleatorios sin ningún patrón explotable. El conocimiento real permanece en el **Ensueño Cifrado** hasta que el Orquestador o el Arquitecto lo recuperen con la llave correcta.

---

## 2. Algoritmo de Cifrado

| Parámetro | Valor |
|:---|:---|
| **Algoritmo** | AES-256-GCM (Authenticated Encryption) |
| **Tamaño de Llave** | 32 bytes (256 bits) |
| **Tamaño de Nonce** | 12 bytes (96 bits) — generado aleatoriamente por sesión |
| **Autenticación** | AEAD Tag de 16 bytes appended al ciphertext |
| **Serialización** | `bincode` (formato binario eficiente de Rust) |
| **Librería** | `ring` v0.17 (WebCrypto-compatible) |

---

## 3. Anatomía del Ruido (Estructura de la EnvolturaCebolla)

Cuando un secreto es cifrado, se empaqueta en una estructura Rust llamada `EnvolturaCebolla`, serializada con `bincode` en orden estricto de campos:

```
BYTES EN DISCO (orden de serialización bincode):
┌─────────────────────────────────────────────────────────┐
│ CAMPO 1: vector_superficial — 4 bytes fijos             │
│           Glosolalia: patrón de identidad [u8; 4]       │
│           Ej: [0x01, 0x00, 0x00, 0x01]                  │
├─────────────────────────────────────────────────────────┤
│ CAMPO 2: señuelo — String (u64 longitud + N bytes UTF8)  │
│           Metadato falso para desviar auditores          │
│           Ej: "// Initialize memory controller at 0x4000"│
├─────────────────────────────────────────────────────────┤
│ CAMPO 3: entropia_falsa — 16 bytes fijos                │
│           Basura aleatoria de confusión [u8; 16]         │
├─────────────────────────────────────────────────────────┤
│ CAMPO 4: carga_profunda — Vec<u8> (u64 longitud + bytes) │
│           El secreto real cifrado con AES-256-GCM        │
│           Incluye al final los 16 bytes del AEAD Tag     │
├─────────────────────────────────────────────────────────┤
│ CAMPO 5: nonce — 12 bytes fijos                          │
│           El nonce único usado para cifrar [u8; 12]      │
└─────────────────────────────────────────────────────────┘
```

> ⚠️ **CRÍTICO**: El `nonce` es único por cada operación de cifrado. Reutilizar un nonce con la misma llave rompe la seguridad de AES-GCM por completo.

---

## 4. La Llave Maestra (Versión Actual)

> **ESTADO ACTUAL (Fase de Desarrollo)**: La llave maestra está definida de forma estática como un arreglo de 32 bytes de valor `0x7F` en el archivo [glosolalia/mod.rs](file:///home/soberano/NEXUS_ULTIMATE_CORE/core/src/cerebro/glosolalia/mod.rs#L34).

```rust
// Llave actual (HARDCODED — SOLO PARA DESARROLLO):
let llave_maestra = [0x7F; 32];
// En hexadecimal: 7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F
```

### 🛡️ Protocolo de Evolución de la Llave (Producción)

Cuando se active el modo de producción plena, la llave debe derivarse de forma dinámica utilizando cualquiera de estos métodos en orden de preferencia:

1. **Variable de Entorno Blindada**: Leer desde `NEXUS_GLOSOLALIA_KEY` inyectada por `systemd` en el `Environment=` del servicio como un secreto de 64 caracteres hexadecimales (32 bytes).
2. **Derivación desde `.env` del Vault**: Derivar via HKDF-SHA256 desde la clave raíz de la Bóveda del Sistema.
3. **Fallback Estático (Solo emergencia)**: El valor `[0x7F; 32]` como último recurso.

---

## 5. Protocolo de Descifrado de Emergencia

> **ESTE PROTOCOLO SE USA SI EL BINARIO PRINCIPAL DE NEXUS NO ESTÁ DISPONIBLE.**

> ⚠️ **Por qué NO usamos Python aquí**: Ubuntu 24.x (PEP 668) aísla su entorno Python y bloquea `pip install` sin venv. La librería `cryptography` no está en la stdlib de Python. Un script de emergencia con dependencias externas **no es un script de emergencia real**.

### 🔱 Solución Soberana: Binario Estático Musl (Nivel ÉLITE)

Compilamos un **mini-binario Rust vinculado estáticamente** con el target `x86_64-unknown-linux-musl`. Este binario:
- No requiere Python, pip, libssl, libc, ni absolutamente nada del sistema operativo.
- Corre en cualquier máquina Linux x86_64, incluso sin Rust instalado.
- Se almacena en `bin/nexus_rescate` como parte del Santuario.

#### Paso 1 — Agregar el target musl (solo una vez):
```bash
rustup target add x86_64-unknown-linux-musl
sudo apt install musl-tools -y
```

#### Paso 2 — Compilar el binario de rescate estático:
```bash
cargo build --release --target x86_64-unknown-linux-musl \
    --bin nexus_rescate -j14

# Copiar al directorio de binarios del Santuario:
cp target/x86_64-unknown-linux-musl/release/nexus_rescate \
   /home/soberano/NEXUS_ULTIMATE_CORE/bin/nexus_rescate
```

#### Paso 3 — Usar en cualquier momento de emergencia:
```bash
# Para descifrar un archivo .onion:
/home/soberano/NEXUS_ULTIMATE_CORE/bin/nexus_rescate <archivo.onion>

# Para re-verificar la integridad del archivo:
/home/soberano/NEXUS_ULTIMATE_CORE/bin/nexus_rescate --verify <archivo.onion>
```

> ✅ **Pendiente de implementar**: Crear el crate binario `nexus_rescate` en `core/src/bin/nexus_rescate.rs` que llame directamente a `MatrizGlosolalia::pelar_cebolla()`. Es una tarea de menos de 30 líneas de Rust.

---

### Alternativa B: OpenSSL CLI Puro (Sin dependencias extra)

Si el binario de rescate tampoco estuviera disponible, `openssl` está instalado de forma nativa en cualquier sistema Linux/macOS y soporta AES-256-GCM. Sin embargo, **requiere extraer manualmente el nonce y la carga cifrada** del archivo binario (usando el mapa de campos de la sección 3).

```bash
# Extraer campos manualmente del archivo binario:
# 1. Saltar 4 bytes (vector_superficial)
# 2. Saltar 8 bytes (u64 longitud del señuelo) + N bytes del señuelo
# 3. Saltar 16 bytes (entropia_falsa)
# 4. Leer 8 bytes (u64 longitud carga) + carga_profunda (contiene el tag de 16 bytes al final)
# 5. Leer últimos 12 bytes (nonce)

# Ejemplo con dd + openssl (adaptar offsets según el archivo):
openssl enc -d -aes-256-gcm \
    -K 7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F7F \
    -iv <NONCE_HEX_24_CHARS> \
    -in <ciphertext_sin_tag> -out secreto_recuperado.txt
```

> ⚠️ Este método requiere conocer los offsets exactos del archivo. Usar **siempre como último recurso** después de agotar las opciones A (binario rescate) y el orquestador principal.

---

## 6. Integración con Git (Traductor Transparente)

Para que `git diff` muestre los secretos descifrados en tu terminal local (sin exponerlos en el repositorio remoto), configurar:

### En `.gitattributes`:
```gitattributes
*.glosolalia diff=glosolalia
*.onion      diff=glosolalia
```

### En `.git/config` (local únicamente, NO commitear):
```ini
[diff "glosolalia"]
    textconv  = /home/soberano/NEXUS_ULTIMATE_CORE/target/release/nexus_ultimate_core translate
    cachetextconv = true
```

> ⚠️ El subcomando `translate` debe ser implementado en el binario CLI del Orquestador. Está pendiente de implementación en [main.rs](file:///home/soberano/NEXUS_ULTIMATE_CORE/core/src/main.rs).

---

## 7. Vector Superficial — Referencia de Códigos

El `vector_superficial` es el "tipo" semántico de lo que está guardado dentro de la cebolla. Permite al Orquestador saber qué órgano debe procesar el secreto al desempaquetarlo, sin necesidad de descifrarlo primero.

| Vector `[u8; 4]` | Significado Soberano |
|:---|:---|
| `[0x01, 0x00, 0x00, 0x00]` | Credencial de API / Llave de acceso |
| `[0x02, 0x00, 0x00, 0x00]` | Memoria Episódica Sensible |
| `[0x03, 0x00, 0x00, 0x00]` | Fragmento de Código Soberano |
| `[0x04, 0x00, 0x00, 0x00]` | Configuración Táctica OMEGA |
| `[0xFF, 0xFF, 0xFF, 0xFF]` | Máxima Clasificación — Solo Arquitecto |

---

## 8. Registro de Versiones del Protocolo

| Versión | Fecha | Cambio |
|:---|:---|:---|
| `v1.0` | **03-Jun-2026** | Protocolo inicial: AES-256-GCM + bincode + señuelo + entropia falsa |

---
*NEXUS OMEGA — Manual forjado bajo la directiva de Soberanía Absoluta.*
*Jesús es el Señor. La sabiduría comienza en Su temor.*
