# 🧠 DIARIO DE APRENDIZAJES Y EVOLUCIÓN (NEXUS OMEGA)

## [2026-03-20] ERROR: INVISIBILIDAD DEL AGENTE
- **Falla**: Intenté usar `browser_subagent` (Fantasma Interno) cuando el Arquitecto quería ver a NEXUS actuar en su propia pantalla.
- **Lección**: La visualización es el 80% de la confianza del Arquitecto. Priorizar X11 (`DISPLAY=:0`) y herramientas visibles.
- **Acción Correctiva**: No usar subagentes invisibles para demostraciones de mando directo.

## [2026-03-20] ERROR: BLOQUEO POR COMPILACIÓN
- **Falla**: Intenté lanzar `cargo run` tras un cambio masivo de `lib.rs`, causando una espera de 10 minutos improductiva.
- **Lección**: Rust es lento; el usuario es rápido. 
- **Acción Correctiva**: Protocolo de "Bypass" inmediato. Abrir la ventana primero, compilar en fondo después.

## [2026-03-20] ERROR: FALLA DE PUNTERÍA VISUAL (COORDENADA Y)
- **Falla**: Realicé un clic ciego en X11 a 700px de altura, fallando por completo la caja de Gemini.
- **Lección**: Las cajas de texto a 1080p en Gemini suelen residir en el epicentro (960x560). La ráfaga técnica debe ser calibrada con tu captura de pantalla actual.
- **Acción Correctiva**: Usar coordenadas dinámicas o esperar a que la 'Vigilancia Visual' de NEXUS confirme la posición antes de escribir.

---
*El Error de Puntería es la Base del Ojo de Águila. NEXUS ha aprendido a mirar antes de pulsar.* 🛡️🦾🧬

## [2026-03-20] LECCIÓN MAESTRA: LA TRINIDAD DEL EVENTO
- **Falla**: El "Enter" no enviaba el mensaje a pesar de estar escrito.
- **Solución**: Usar la secuencia biológica del teclado (keyDown -> char \r -> keyUp).
- **Refuerzo**: NEXUS sabe que Gemini no acepta pulsaciones muertas; necesita el pulso de la trinidad.

