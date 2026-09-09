## Config.gd — Configuración central del proyecto.
## NEXUS Protocol · Fase 1 · Núcleo
## Autoload singleton accesible como `NexusConfig`.
extends Node

## Constantes globales de configuración del mundo procedural.
const VERSION := "0.1.0-beta"

## Dimensiones de cada chunk en celdas (eje X, Z). El eje Y es altura.
const CHUNK_SIZE := Vector2i(16, 16)

## Tamaño de la celda en unidades del mundo.
const CELL_SIZE := 1.0

## Radio de chunks a mantener cargados alrededor del jugador.
const LOAD_RADIUS := 3

## Semilla base para generación reproducible.
var seed_base := 1985

## Rango de altura del terreno (mínimo / máximo).
const TERRAIN_MIN_HEIGHT := 1.0
const TERRAIN_MAX_HEIGHT := 12.0

## Parámetros de ruido de terreno.
const NOISE_FREQUENCY := 0.02
const NOISE_OCTAVES := 4
const NOISE_LACUNARITY := 2.0
const NOISE_GAIN := 0.5

func _ready() -> void:
	print("[NEXUS] Config inicializada v%s (chunk=%s, radio=%d)" % [VERSION, CHUNK_SIZE, LOAD_RADIUS])
