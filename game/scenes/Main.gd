## Main.gd — Escena principal: configuración del mundo y cámara isométrica.
## NEXUS Protocol · Fase 1 · Vista isométrica low-poly
class_name Main
extends Node3D

## Referencia al contenedor de chunks.
@onready var chunk_container: Node3D = $ChunkContainer
## Referencia al manager de chunks.
@onready var chunk_manager: ChunkManager = $ChunkManager
## Referencia al jugador.
@onready var player: PlayerController = $Player

func _ready() -> void:
	# Configurar la cámara isométrica.
	_setup_iso_camera()
	# Conectar el manager con el contenedor del mundo.
	chunk_manager.chunk_container = chunk_container
	# Inicializar el streaming de chunks alrededor del jugador.
	chunk_manager.bootstrap(player.global_position)
	print("[NEXUS] Mundo inicializado. Chunks activos: %d" % chunk_manager.active_count())

## Configura una cámara isométrica (rotación de 45° sobre los ejes Y y X).
func _setup_iso_camera() -> void:
	var cam := get_node_or_null("Camera3D") as Camera3D
	if cam == null:
		return
	# Proyección ortográfica para la estética isométrica low-poly.
	# Distancia y ángulos típicos isométricos.
	cam.rotation_degrees = Vector3(-45.0, 45.0, 0.0)
	cam.position = Vector3(0.0, 90.0, 90.0)
	cam.projection = Camera3D.PROJECTION_ORTHOGONAL
	cam.size = 60.0
	cam.make_current()
