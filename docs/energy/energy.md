# 🔱 MANIFIESTO DE ENERGÍA Y ZENITH POOL (OMEGA-32)

Este documento es el mapa oficial del arsenal energético de NEXUS, detallando sus células de inteligencia y protocolos de rotación/respaldo.

## ⚡ Estructura de Combustible (Órganos)

| Órgano | Archivo | Función |
| :--- | :--- | :--- |
| **NEXUS Nativo** | `gemini_nativo.rs` | API de NEXUS con 4 llaves, Pacing, Jitter |
| **Zenith Pool** | `zenith_pool.rs` | Orquestador `responder_estrategico` (OpenRouter → DeepSeek → Groq → Vertex → AI Studio → Ollama) |
| **Quantum Flux** | `quantum_flux_capacitor.rs` | Gestión de llaves (absorbido por NEXUS Nativo) |
| **Velocímetro** | `velocimetro.rs` | Monitorea cuotas, predice agotamiento |
| **Forge** | `forge.rs` | Crea proyectos Google Cloud |

---

## 🔥 ORDEN ENERGÉTICO (responder_estrategico — ACTUALIZADO)

> **Cambio aprobado**: Google AI Studio NO es primario. Queda como **ÚLTIMO RESPALDO** (no se elimina). OpenRouter es el motor primario porque una sola llave da acceso a cientos de modelos y NO depende de cuentas de Google que se bloquean fácilmente (lección `crispragmatico2021`).

| # | Motor | Cómo | Notas |
| :--- | :--- | :--- | :--- |
| **1** | **OpenRouter** (PRIMARIO) | `ejecutor_openrouter` | 1 llave, 100+ modelos, sin bloqueo por cuenta |
| **2** | **DeepSeek** (fallback 1) | `ejecutor_deepseek` | API oficial, texto puro |
| **3** | **Groq LPU** (fallback 2) | `ejecutor_groq` | Inferencia ultrarrápida |
| **4** | **Vertex AI** (fallback 3) | `ejecutor_vertex` | Cuenta GCP $300 |
| **5** | **Gemini AI Studio** (ÚLTIMO RESPALDO) | `cerebro_gemini` | Solo si todo lo anterior falló |
| **6** | **Cadena final** | `cadena_fallbacks` | Córtex nativo → DeepSeek → Vertex → OpenRouter → Groq → **Ollama local** (cierre soberano) |

---

## 📊 Inventario de Células Energéticas (Zenith Pool)

NEXUS cuenta con **31+ núcleos de inteligencia** distribuidos en células independientes. ⚠️ **Todas las células de Google AI Studio están BLOQUEADAS/SUSPENDIDAS y son solo pool de ÚLTIMO RESPALDO**: se conservan (no se eliminan) pero NEXUS no depende de ellas.

| Célula | Identidad | Capacidad | Estado |
| :--- | :--- | :--- | :--- |
| **Célula 1** | `nestorfranco2026` | 1 LLAVE ($300) | ⛔ **BLOQUEADA** (preservar crédito) |
| **Célula 2** | `dogperro404` | 10 LLAVES | ⛔ **BLOQUEADA** (403 suspendidas) |
| **Célula 3** | `lucianiaquino53` | 10 LLAVES | ⛔ **BLOQUEADA** (1 saturada 503) |
| **Célula 4** | `crispragmatico2021` | 10 LLAVES | ⛔ **BLOQUEADA** (problemas de pago) |
| **Célula 5** | `divinemercy6321` | 1 LLAVE | ⛔ **BLOQUEADA** |

**Potencia Total Estimada**: ~4.5 millones de tokens/minuto (Flash) — pero la disponibilidad real la da OpenRouter (primario).

---

## 🛠️ Protocolo de Operación en el Búnker NVMe

Para optimizar la infraestructura de almacenamiento local (Intel i7 + 1TB), se implementa la **Rotación de Celular Dinámica**:

1. **Aislamiento de Células**: Cada grupo de llaves opera bajo una firma de red distinta (usando el *Frequency Confusion Engine*).
2. **Balanceo de Carga**: El orquestador rotará no solo entre llaves, sino entre células completas si se detecta una anomalía de red o bloqueo en una cuenta específica.
3. **Respaldo Criptográfico**: Todas estas llaves residirán en un archivo `vault.nix` cifrado con LUKS, invisible para análisis forenses externos.

## 🚀 Próximos Pasos (Migración i7)
- [x] Promover OpenRouter a motor primario (`responder_estrategico`).
- [x] AI Studio relegado a ÚLTIMO RESPALDO (no eliminado).
- [x] Fallback final local: Ollama (`qwen2.5:7b`) como cierre soberano.
- [ ] Integrar el pool de 31 llaves en el `configuration.nix` maestro.
- [ ] Activar el Dashboard Visual para monitorear el pulso de cada célula en tiempo real.
