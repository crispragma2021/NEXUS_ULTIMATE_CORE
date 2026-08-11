# NEXUS Protocol — Game Client (Godot)

**Fase 1 · Núcleo y Estructura** — Mundo procedural low-poly / isométrico por chunks.

## Requisitos
- **Godot Engine 4.3+** (Renderizado Forward+)
- Proyecto pensado para ser exportado a **PC (Windows)** y **Android (APK)** en la Fase 4.

## Cómo ejecutar
Abrir el directorio `game/` con el editor Godot e importar el proyecto (`project.godot`), o ejecutar desde terminal:

```bash
godot --path ./game
```

## Estructura del proyecto
```
game/
├── project.godot              # Configuración del motor y autoloads
├── autoload/
│   ├── Config.gd              # Constantes y parámetros del mundo
│   └── World.gd               # Orquestador global del mundo procedural
├── scenes/
│   ├── Main.tscn / Main.gd    # Escena principal + cámara isométrica
│   └── Player.tscn            # Personaje de pruebas
├── scripts/
│   ├── world/
│   │   ├── Chunk.gd           # Nodo lógico de chunk
│   │   ├── ChunkManager.gd    # Streaming dinámico de chunks
│   │   └── TerrainGenerator.gd# Generación procedural (ruido Perlin)
│   └── player/
│       └── PlayerController.gd# Control de movimiento y salto
└── assets/
    └── icon.svg               # Ícono del proyecto
```

## Arquitectura modular por chunks
- **NexusConfig** (autoload): parámetros globales — tamaño de chunk, radio de carga, semilla, rangos de altura.
- **NexusWorld** (autoload): contiene el generador, indexa chunks activos por coordenadas `Vector2i` y emite señales `chunk_loaded` / `chunk_unloaded`.
- **ChunkManager**: re-escanea la malla de chunks alrededor del jugador, cargando los nuevos y descargando los lejanos (streaming).
- **TerrainGenerator**: genera mallas `ArrayMesh` por chunk con ruido Perlin fractal, colores por altura (gradiente de bioma low-poly) y colisión `StaticBody3D`.

## Fases siguientes (plan)
1. ✅ **Fase 1** — Núcleo, estructura modular y generador procedural.
2. ⬜ **Fase 2** — Asset Pipeline (conceptos con Gemini → `.glb` con Meshy → optimización en Blender).
3. ⬜ **Fase 3** — Servidor central en Rust + red (WebSockets/ENet).
4. ⬜ **Fase 4** — Exportación Beta para PC (Windows) y APK (Android).
