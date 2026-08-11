#!/usr/bin/env -S godot --script
# SpawnZone — Zona de spawn de bestias configurable

class_name SpawnZone
extends Node3D

# Datos de la bestia a spawnear
@export var beast_data: BeastData = null

# Configuración de spawn
@export var max_concurrent: int = 5
@export var spawn_interval: float = 10.0
@export var spawn_radius: float = 20.0
@export var despawn_distance: float = 100.0
@export var only_spawn_when_player_near: bool = true
@export var player_detection_radius: float = 80.0

# Interno
var _active_beasts: Array[Beast] = []
var _spawn_timer: float = 0.0

func _ready() -> void:
	# Visualizar zona en editor
	#if TOOLS:
	#	update_gizmos()
	##endif
	
	_spawn_timer = randf_range(0.0, spawn_interval)

func _process(delta: float) -> void:
	if beast_data == null:
		return
	
	# Limpiar bestias muertas o inválidas
	_cleanup_dead_beasts()
	
	# Verificar si debemos spawnear
	if _should_spawn():
		_spawn_timer -= delta
		if _spawn_timer <= 0.0:
			_try_spawn()
			_spawn_timer = spawn_interval

func _should_spawn() -> bool:
	if _active_beasts.size() >= max_concurrent:
		return false
	
	if only_spawn_when_player_near:
		var player = _get_player()
		if player != null:
			var dist = global_position.distance_to(player.global_position)
			return dist <= player_detection_radius
		return false
	
	return true

func _cleanup_dead_beasts() -> void:
	var valid = []
	for b in _active_beasts:
		if is_instance_valid(b) and b.is_alive():
			# Verificar distancia de despawn
			if global_position.distance_to(b.global_position) <= despawn_distance:
				valid.append(b)
			else:
				b.queue_free()
		elif is_instance_valid(b):
			b.queue_free()
	_active_beasts = valid

func _get_player() -> Node3D:
	return get_tree().get_first_node_in_group("player") as Node3D

func _try_spawn() -> void:
	if beast_data == null:
		return
	
	var spawn_pos = _find_valid_spawn_position()
	
	var beast_scene = load("res://scenes/beasts/beast_base.tscn")
	var beast_instance: Beast = beast_scene.instantiate()
	beast_instance.global_position = spawn_pos
	beast_instance.beast_data = beast_data
	beast_instance.name = "%s_%d" % [beast_data.species_name, _active_beasts.size()]
	
	# Conectar señal de muerte
	beast_instance.beast_died.connect(_on_beast_died.bind(beast_instance))
	
	add_child(beast_instance)
	_active_beasts.append(beast_instance)
	
	print("[SPAWN] Spawneado %s en %s" % [beast_data.species_name, spawn_pos])

func _find_valid_spawn_position() -> Vector3:
	var attempts = 10
	for i in range(attempts):
		var angle = randf() * TAU
		var radius = randf_range(2.0, spawn_radius)
		var pos = global_position + Vector3(cos(angle) * radius, 0.0, sin(angle) * radius)
		
		# Verificar que no esté en zona segura
		var space_state = get_world_3d().direct_space_state
		var query = PhysicsRayQueryParameters3D.create(pos + Vector3.UP * 10.0, pos - Vector3.UP * 10.0)
		query.collision_mask = 1  # solo terreno
		var result = space_state.intersect_ray(query)
		
		if result:
			return result.position
	
	# Fallback: devolver la posición de la zona (no puede ser null en Vector3)
	return global_position

func _on_beast_died(beast: Beast, level: int) -> void:
	# La bestia ya se limpiará en _cleanup_dead_beasts
	pass

# Visualización en editor (Node3D no soporta _draw; se omite por claridad)