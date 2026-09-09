## World.gd — Autoload global que orquesta el mundo procedural.
## NEXUS Protocol · Fase 1 · Núcleo
## Autoload singleton accesible como `NexusWorld`.
extends Node

## Señal emitida cuando un chunk ha sido generado y añadido al mundo.
signal chunk_loaded(chunk_id: Vector2i, chunk: Node3D)
## Señal emitida cuando un chunk es descargado.
signal chunk_unloaded(chunk_id: Vector2i)

## Referencia al contenedor de chunks en el árbol de escena.
var _chunk_container: Node3D = null
## Diccionario de chunks activos indexados por coordenadas de chunk.
var _chunks: Dictionary = {}
## Instancia del generador procedural de terreno.
var _generator: Node = null

func _ready() -> void:
	_generator = load("res://scripts/world/TerrainGenerator.gd").new()
	add_child(_generator)
	print("[NEXUS] World autoload listo")

## Inicializa el contenedor de chunks y el generador.
func initialize(container: Node3D) -> void:
	_chunk_container = container
	print("[NEXUS] Contenedor de chunks inicializado")

## Genera un chunk en las coordenadas dadas y lo añade al mundo.
## @param chunk_id Coordenadas del chunk (en unidades de chunk).
## @returns El nodo Chunk generado, o null si ya existía.
func request_chunk(chunk_id: Vector2i) -> Node3D:
	if _chunks.has(chunk_id):
		return _chunks[chunk_id]
	if _generator == null:
		return null

	var chunk: Node3D = _generator.generate_chunk(chunk_id, _chunk_container)
	_chunks[chunk_id] = chunk
	chunk_loaded.emit(chunk_id, chunk)
	return chunk

## Elimina un chunk del mundo y libera su memoria.
func release_chunk(chunk_id: Vector2i) -> void:
	if not _chunks.has(chunk_id):
		return
	var chunk: Node3D = _chunks[chunk_id]
	_chunks.erase(chunk_id)
	if chunk != null and is_instance_valid(chunk):
		chunk.queue_free()
	chunk_unloaded.emit(chunk_id)

## Devuelve el chunk en la posición mundial dada (o null).
func get_chunk_at_world(world_pos: Vector3) -> Node3D:
	var chunk_id := _to_chunk_id(world_pos)
	return _chunks.get(chunk_id)

## Punto de entrada para el ChunkManager: reenvía la posición del jugador.
func _notify_player_pos(pos: Vector3) -> void:
	pass  # el ChunkManager en escena maneja el streaming directamente

## Convierte una posición mundial a coordenadas de chunk.
func _to_chunk_id(world_pos: Vector3) -> Vector2i:
	var cs := NexusConfig.CHUNK_SIZE
	return Vector2i(
		floori(world_pos.x / cs.x),
		floori(world_pos.z / cs.y)
	)

## Convierte una coordenada de chunk a la posición mundial de su origen.
func chunk_to_world_origin(chunk_id: Vector2i) -> Vector3:
	var cs := NexusConfig.CHUNK_SIZE
	return Vector3(chunk_id.x * cs.x, 0.0, chunk_id.y * cs.y)

## Número de chunks activos actualmente.
func active_chunk_count() -> int:
	return _chunks.size()
