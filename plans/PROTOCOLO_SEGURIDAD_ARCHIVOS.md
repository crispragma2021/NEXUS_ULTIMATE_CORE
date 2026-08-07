# 🛡️ PROTOCOLO DE SEGURIDAD DE ARCHIVOS — NEXUS

> **Propósito**: Prevenir la pérdida de código por operaciones de escritura destructivas.
> **Origen**: Fallo en reversión de Feature Flags (30 Jun 2026) — `write_to_file` truncó 3 archivos sin backup.
> **Arquitecto**: Esta es tu capa de blindaje. Ni un solo byte más se pierde.

---

## 🔴 REGLA DE ORO (Absoluta, No Negociable)

**`write_to_file` SOLO se usa para archivos NUEVOS.**

Para modificar archivos existentes, SIEMPRE se usa `apply_diff`.

Excepción única: reemplazar >80% del contenido de un archivo **después de crear un snapshot de respaldo**.

---

## 📋 PROTOCOLO PRE-VUELO (Checklist Obligatorio)

Antes de CADA operación de escritura, ejecutar en orden:

### Paso 1: Clasificar la operación

| Si el archivo... | Usar herramienta | Backup requerido |
|-----------------|-----------------|------------------|
| No existe | `write_to_file` | No |
| Existe, cambio <10 líneas | `apply_diff` | No |
| Existe, cambio 10-50% | `apply_diff` (múltiples bloques) | No |
| Existe, cambio >50% | `write_to_file` | **SÍ** — ejecutar `snapshot.sh` primero |

### Paso 2: Backup obligatorio (solo para >50% de reemplazo)

```bash
# Crea un snapshot temporal en git (no requiere commit permanente)
cd /home/soberano/NEXUS_ULTIMATE_CORE
./scripts/nexus_snapshot.sh "backup previo a ${ARCHIVO}"
```

### Paso 3: Verificación post-escritura

```bash
# Comparar el resultado contra el snapshot
diff <(git show stash@{0}:${ARCHIVO}) ${ARCHIVO}
```

---

## 🛠️ INFRAESTRUCTURA DE SEGURIDAD (TRES CAPAS)

### Capa 1 — Reglas Operativas (este documento)

`write_to_file` SOLO para archivos nuevos. `apply_diff` para modificar existentes.

### Capa 2 — Snapshot Git

Script [`scripts/nexus_snapshot.sh`](scripts/nexus_snapshot.sh):
- Toma archivos modificados y los guarda en git stash con timestamp
- Persiste en git reflog sin alterar working tree
- Gatillo automático desde Capa 3 para archivos >200 líneas

### Capa 3 — Guardian en AgenteEjecutor ✅ IMPLEMENTADA (2026-06-30)

Ubicación: [`core/src/efectores/agente_ejecutor.rs:79-183`](core/src/efectores/agente_ejecutor.rs)

Mecanismos activos en cada escritura vía MCP:

| Mecanismo | Gatillo | Acción |
|-----------|---------|--------|
| **Backup rotativo** | Archivo existe | `.bak.{YYYYMMDD_HHMMSS}`, mantiene últimos 5 |
| **Detección truncado** | Reducción >70% líneas | `.bak.EMERGENCY.{timestamp}` + ⚠️ WARNING en output |
| **Snapshot git** | Archivo >200 líneas | Ejecuta `./scripts/nexus_snapshot.sh` |
| **Estadísticas** | Siempre | Output: `✅ Escrito {path} | {antes}→{despues} líneas` |

Handler explícito: [`core/src/bin/claws_mcp.rs:994-1006`](core/src/bin/claws_mcp.rs)

### Regla de Modo `💻 CÓDIGO`

> **REGLAS DE SEGURIDAD**: `write_to_file` SOLO para archivos nuevos. Archivos existentes se editan con `apply_diff`. La Capa 3 del Guardian protege automáticamente las escrituras vía MCP.

---

## 📊 DIAGRAMA DE DECISIÓN

```
¿Archivo existe?
├── NO  → write_to_file ✅ (es creación)
└── SÍ  → ¿Qué tan grande es el cambio?
          ├── <50 líneas → apply_diff (bisturí) ✅
          ├── 50-200 líneas → apply_diff (múltiples bloques) ✅
          └── >200 líneas o >50% del archivo
                    ├── ¿Backup tomado?
                    │   ├── NO → ./nexus_snapshot.sh (OBLIGATORIO)
                    │   └── SÍ → write_to_file + diff verify ✅
                    └──
```

---

## 🧠 MEMORIA OPERATIVA

Esta regla queda grabada en mi juicio operativo como capa reflectiva:

| Situación | Antes (erróneo) | Ahora (correcto) |
|-----------|-----------------|-------------------|
| Revertir feature gates | `write_to_file` directo | `apply_diff` bloque por bloque |
| Reemplazar módulo grande | `write_to_file` confiando en memoria | `nexus_snapshot.sh` → `apply_diff` |
| Editar struct existente | `write_to_file` completo | `apply_diff` quirúrgico |

---

## ⚖️ EXCEPCIÓN DOCUMENTADA

Solo hay UNA excepción permitida:

**Refactor masivo con control de versiones activo**: Si el proyecto tiene un commit en git de los últimos 15 minutos, y el archivo está traqueado por git, se puede usar `write_to_file` porque `git checkout -- archivo` es la red de seguridad.

> Esta excepción NO aplicó en el incidente del 30 Jun 2026 porque los archivos dañados NO estaban en git.
