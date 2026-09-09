## Chunk.gd — Nodo lógico de un chunk del mundo.
## NEXUS Protocol · Fase 1 · Arquitectura por chunks
class_name Chunk
extends Node3D

## Identificador único del chunk en coordenadas de chunk.
var chunk_id := Vector2i.ZERO
## Bandera de carga completa (malla + datos).
var is_loaded := false

func _ready() -> void:
	process_mode = Node.PROCESS_MODE_INHERIT

## Marca el chunk como totalmente cargado.
func mark_loaded() -> void:
	is_loaded = true

## Devuelve la caja AABB aproximada del chunk en espacio local.
func get_bounds() -> AABB:
	var cs := NexusConfig.CHUNK_SIZE
	return AABB(Vector3.ZERO, Vector3(cs.x, NexusConfig.TERRAIN_MAX_HEIGHT, cs.y))
