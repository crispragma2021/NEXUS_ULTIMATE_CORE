#!/usr/bin/env -S godot --script
# BeastAI — Máquina de estados para comportamiento de bestias
# Estados: IDLE, PATROL, CHASE, ATTACK, FLEE, RETURN, DEAD

class_name BeastAI
extends Node

enum State { IDLE, PATROL, CHASE, ATTACK, FLEE, RETURN, DEAD }
signal attack_request(target: Node)

# Referencias
var beast: Beast = null
var beast_data: BeastData = null

# Configuración
var state: int = State.IDLE
var patrol_points: Array[Vector3] = []
var current_patrol_index: int = 0
var target: Node3D = null
var last_attack_time: float = 0.0
var state_timer: float = 0.0

# Distancias
var detection_radius: float = 15.0
var chase_radius: float = 40.0
var attack_range: float = 2.0

func initialize(b: Beast, data: BeastData) -> void:
	beast = b
	beast_data = data
	
	if data != null:
		detection_radius = data.detection_radius
		chase_radius = data.chase_radius
		attack_range = data.attack_range
	
	# Generar puntos de patrulla alrededor de home
	_generate_patrol_points()
	
	# Timer para cambio de estado IDLE -> PATROL
	state_timer = randf_range(2.0, 5.0)

func _generate_patrol_points() -> void:
	if beast == null:
		return
	var home = beast.home_position
	patrol_points.clear()
	for i in range(4):
		var angle = (i / 4.0) * TAU
		var radius = randf_range(5.0, 15.0)
		var point = home + Vector3(cos(angle) * radius, 0.0, sin(angle) * radius)
		patrol_points.append(point)

func _process(delta: float) -> void:
	if beast == null or beast.is_dead:
		return
	
	state_timer -= delta
	
	match state:
		State.IDLE:
			_update_idle(delta)
		State.PATROL:
			_update_patrol(delta)
		State.CHASE:
			_update_chase(delta)
		State.ATTACK:
			_update_attack(delta)
		State.FLEE:
			_update_flee(delta)
		State.RETURN:
			_update_return(delta)
		State.DEAD:
			pass

func _update_idle(delta: float) -> void:
	# Detectar jugador
	var player = _detect_player()
	if player != null:
		set_state(State.CHASE)
		target = player
		return
	
	if state_timer <= 0.0:
		set_state(State.PATROL)

func _update_patrol(delta: float) -> void:
	# Detectar jugador
	var player = _detect_player()
	if player != null:
		set_state(State.CHASE)
		target = player
		return
	
	if patrol_points.is_empty():
		set_state(State.IDLE)
		return
	
	var target_point = patrol_points[current_patrol_index]
	var beast_pos = beast.global_position
	var to_target = target_point - beast_pos
	to_target.y = 0.0
	
	if to_target.length() < 2.0:
		# Llegó al punto, siguiente
		current_patrol_index = (current_patrol_index + 1) % patrol_points.size()
		state_timer = randf_range(1.0, 3.0)
		set_state(State.IDLE)
		return
	
	# Moverse hacia el punto
	var dir = to_target.normalized()
	var spd = beast_data.move_speed if beast_data != null else 3.0
	beast.velocity.x = dir.x * spd
	beast.velocity.z = dir.z * spd
	beast.move_and_slide()

func _update_chase(delta: float) -> void:
	if target == null or not is_instance_valid(target):
		set_state(State.RETURN)
		return
	
	var dist = beast.global_position.distance_to(target.global_position)
	
	# ¿Jugador muy lejos? Volver
	if dist > chase_radius:
		set_state(State.RETURN)
		return
	
	# ¿En rango de ataque?
	if dist <= attack_range:
		set_state(State.ATTACK)
		return
	
	# Perseguir
	var dir = (target.global_position - beast.global_position).normalized()
	dir.y = 0.0
	var spd = beast_data.move_speed if beast_data != null else 3.0
	beast.velocity.x = dir.x * spd
	beast.velocity.z = dir.z * spd
	beast.move_and_slide()

func _update_attack(delta: float) -> void:
	if target == null or not is_instance_valid(target):
		set_state(State.RETURN)
		return
	
	var dist = beast.global_position.distance_to(target.global_position)
	
	# Jugador se alejó
	if dist > attack_range + 1.0:
		set_state(State.CHASE)
		return
	
	# Cooldown de ataque
	var cd = beast_data.attack_cooldown if beast_data != null else 1.5
	if Time.get_ticks_msec() / 1000.0 - last_attack_time < cd:
		# Mantener posición, mirar al jugador
		beast.velocity.x = move_toward(beast.velocity.x, 0.0, 5.0)
		beast.velocity.z = move_toward(beast.velocity.z, 0.0, 5.0)
		beast.move_and_slide()
		return
	
	# Atacar
	last_attack_time = Time.get_ticks_msec() / 1000.0
	_perform_attack()
	
	# Verificar si debe huir (HP bajo)
	var flee_thresh = beast_data.flee_health_threshold if beast_data != null else 0.2
	if beast.health != null and beast.health.get_health_percentage() <= flee_thresh:
		set_state(State.FLEE)

func _update_flee(delta: float) -> void:
	if target == null:
		set_state(State.RETURN)
		return
	
	# Huir en dirección opuesta al jugador
	var away_dir = (beast.global_position - target.global_position).normalized()
	away_dir.y = 0.0
	var fspd = (beast_data.move_speed if beast_data != null else 3.0) * 1.5
	beast.velocity.x = away_dir.x * fspd
	beast.velocity.z = away_dir.z * fspd
	beast.move_and_slide()
	
	# Si está lo suficientemente lejos, volver
	var dist = beast.global_position.distance_to(target.global_position)
	if dist > chase_radius:
		set_state(State.RETURN)

func _update_return(delta: float) -> void:
	var home = beast.home_position
	var dist = beast.global_position.distance_to(home)
	
	if dist < 3.0:
		set_state(State.IDLE)
		return
	
	var dir = (home - beast.global_position).normalized()
	dir.y = 0.0
	var rspd = beast_data.move_speed if beast_data != null else 3.0
	beast.velocity.x = dir.x * rspd
	beast.velocity.z = dir.z * rspd
	beast.move_and_slide()

func _perform_attack() -> void:
	emit_signal("attack_request", target)

func on_damaged(attacker: Node) -> void:
	if attacker is Node3D:
		target = attacker
		if state == State.IDLE or state == State.PATROL:
			set_state(State.CHASE)

func _detect_player() -> Node3D:
	if beast == null:
		return null
	
	# Buscar en DetectionArea
	var overlapping = beast.detection_area.get_overlapping_bodies()
	for body in overlapping:
		if body.name == "Player" or body.name == "PlayerController":
			return body
	return null

func set_state(new_state: int) -> void:
	if state == new_state:
		return
	state = new_state
	state_timer = 0.0
	
	match new_state:
		State.IDLE:
			state_timer = randf_range(2.0, 5.0)
		State.PATROL:
			pass
		State.CHASE:
			pass
		State.ATTACK:
			pass
		State.FLEE:
			pass
		State.RETURN:
			pass
		State.DEAD:
			pass