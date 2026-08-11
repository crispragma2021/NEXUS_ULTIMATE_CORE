## ChunkManager.gd — Orquestador de carga/descarga dinámica de chunks.
## NEXUS Protocol · Fase 1 · Arquitectura por chunks
class_name ChunkManager
extends Node

## Distancia (en unidades de mundo) desde la cual se re-evalúa la carga.
const RESCAN_DISTANCE := 8.0

## Nodo contenedor de todos los chunks del mundo.
var chunk_container: Node3D = null
## Posición de referencia (usualmente el jugador) para el streaming.
var _tracked_position := Vector3.ZERO
## Última posición en la que se re-escaneó.
var _last_scan_pos := Vector3.INF
## Centro de la malla de chunks activos.
var _center_chunk := Vector2i(100000, 100000)
## Conjunto de IDs de chunk actualmente activos.
var _active := {}

func _ready() -> void:
	NexusWorld.initialize(chunk_container)

## Actualiza la posición objetivo de streaming.
func set_tracked_position(pos: Vector3) -> void:
	_tracked_position = pos
	_update_center()

## Inicializa la primera malla de chunks alrededor de la posición dada.
func bootstrap(pos: Vector3) -> void:
	set_tracked_position(pos)
	_refresh()

## Función llamada cada frame; re-escanea solo cuando hay movimiento relevante.
func _process(_delta: float) -> void:
	if _tracked_position.distance_to(_last_scan_pos) > RESCAN_DISTANCE:
		_update_center()
		_refresh()

func _update_center() -> void:
	var center := NexusWorld._to_chunk_id(_tracked_position)
	if center != _center_chunk:
		_center_chunk = center

## Recarga la malla de chunks: carga los que faltan y descarga los lejanos.
func _refresh() -> void:
	var radius := NexusConfig.LOAD_RADIUS
	var desired := {}

	# Construir el conjunto deseado dentro del radio.
	for dz in range(-radius, radius + 1):
		for dx in range(-radius, radius + 1):
			var cid := _center_chunk + Vector2i(dx, dz)
			desired[cid] = true
			if not _active.has(cid):
				NexusWorld.request_chunk(cid)

	# Descargar chunks que ya no están en el radio.
	for cid in _active.keys():
		if not desired.has(cid):
			NexusWorld.release_chunk(cid)

	_active = desired
	_last_scan_pos = _tracked_position

## Cuenta de chunks activos en el administrador.
func active_count() -> int:
	return _active.size()
