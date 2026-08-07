# 🔱 NEXUS GHOST-SHELL - Manual de Operaciones Soberanas

Este documento detalla las especificaciones técnicas y de interfaz del orbe de control periférico de NEXUS.

## 🛰️ Visión General
El **GHOST-SHELL** es la interfaz de control periférico de NEXUS. Un orbe minimalista que flota sobre el entorno de escritorio (Wayland) y permite la ejecución de comandos de sistema, interacción con el Orquestador y monitoreo ráfaga sin abandonar la tarea actual.

---

## 🛠️ Especificaciones Técnicas (Bajo Pilar 4: Rust Puro)

### 1. Núcleo del Sistema
- **Binario**: `nexus-ghost` (Rust v1.75+)
- **Motor UI**: Tauri v2 (Ligereza extrema + CSS Neon Reuse)
- **Protocolo de Ventana**: Wayland Support + Frameless Transparent Window.

### 2. Funcionalidades de "Soberanía de Escritorio"
- **Orb Mode**: Icono circular de 60px con logo NEXUS animado.
- **Chat Mode**: Ventana cuadrada minimalista (Glassmorphism).
- **Draggable**: Arrastre fluido mediante eventos nativos del sistema.
- **Intelli-Hide**: Ocultación en bordes con auto-emergencia al hover (Dash-to-Dock Style).
- **Direct Terminal**: El chat enviará las instrucciones directamente al `nexus-orquestador` para su ejecución en el hardware real.

---

## 📋 Lista de Tareas de Implementación

### Fase 1: Estructura y Vínculo
- [x] Crear directorio `nexus-ghost-shell`.
- [x] Inicializar proyecto Cargo + dependencias de Tauri.
- [x] Configurar `tauri.conf.json` para transparencia y modo `alwaysOnTop`.

### Fase 2: El Cuerpo del Orbe (UI/UX)
- [x] Implementar `index.html` con el Orbe Neón.
- [x] Clonar variables de color de `style.css` del HUD principal.
- [x] Crear lógica de transición Orb <-> Chat.

### Fase 3: El Motor Energético (Rust)
- [x] Implementar lógica de Draggable en `main.rs`.
- [x] Implementar el "Watcher" de bordes para auto-ocultación.
- [x] Configurar el WebSocket Client para hablar con el puerto `43211`.

### Fase 4: Despliegue y Pruebas
- [x] Compilación optimizada dinámica.
- [x] Verificación de Performance.
- [x] Auditoría de Seguridad (Aislamiento de procesos).

---
- **Status**: 🟢 COMPLETADO (Organismo consolidado en modo OMEGA)
