## PlayerController.gd — Control del jugador con sistema de Skills estilo Tibia.
## NEXUS Protocol · Fase 1 · Núcleo
class_name PlayerController
extends CharacterBody3D

## Velocidad de movimiento horizontal.
@export var move_speed := 8.0
## Fuerza de salto.
@export var jump_velocity := 5.0
## Gravedad aplicada.
@export var gravity := 15.0

# Referencias a autoloads
@onready var skill_manager = get_node("/root/NexusSkillManager")
@onready var combat_manager = get_node("/root/NexusCombatManager")

# Componentes
var health: HealthComponent # Nueva declaración
@onready var health_node: Node = get_node("HealthComponent")
@onready var melee_sound = $MeleeSound
@onready var ranged_sound = $RangedSound
@onready var player_damage_sound = $PlayerDamageSound

var _camera: Camera3D = null
# Cooldowns básicos para ataques
var _melee_cooldown: float = 0.0
var _ranged_cooldown: float = 0.0
const MELEE_COOLDOWN_MAX = 0.8
const RANGED_COOLDOWN_MAX = 1.2

# Stats base de armas
const MELEE_BASE_DAMAGE = 12.0
const RANGED_BASE_DAMAGE = 8.0

# Estado
var in_safe_zone: bool = false
var _respawn_timer: float = 0.0

func _ready() -> void:
	collision_layer = 2   # capa "jugador"
	collision_mask = 1    # colisiona con "terreno"
	if _camera == null:
		_camera = _find_camera()
	
	# Conectar señales de salud de forma diferida: en modo headless (y en el primer
	# frame de cualquier ejecución) el script del nodo hijo puede no estar cargado
	# aún cuando el padre ejecuta su _ready(). Diferir al siguiente frame garantiza
	# que HealthComponent esté completamente inicializado.
	call_deferred("_setup_health")

func _setup_health() -> void:
	# Conectar señales de salud (con manejo de errores)
	if health_node:
		# Cast robusto: verifica la API real (set_max_health) en lugar de depender
		# del class_name global, que puede no estar registrado en modo headless.
		if health_node is HealthComponent or health_node.has_method("set_max_health"):
			health = health_node as HealthComponent
			if health != null:
				health.health_changed.connect(_on_health_changed)
				health.died.connect(_on_died)
			else:
				print("ERROR: health_node no pudo ser casteado a HealthComponent. Tipo real: ", health_node.get_class())
		else:
			print("ERROR: health_node no es un HealthComponent válido. Tipo real: ", health_node.get_class())
	else:
		print("ERROR: No se encontró el nodo HealthComponent.")
	
	# Inicializar HP máximo basado en skill Shielding
	_update_max_health()

func _physics_process(delta: float) -> void:
	# Gravedad.
	if not is_on_floor():
		velocity.y -= gravity * delta

	# Salto.
	if Input.is_action_just_pressed("ui_accept") and is_on_floor():
		velocity.y = jump_velocity

	# Movimiento horizontal respecto a la cámara.
	var input := Input.get_vector("ui_left", "ui_right", "ui_up", "ui_down")
	var dir := Vector3.ZERO
	if _camera != null:
		var forward := -_camera.global_transform.basis.z
		forward.y = 0.0
		forward = forward.normalized()
		var right := _camera.global_transform.basis.x
		right.y = 0.0
		right = right.normalized()
		dir = (forward * -input.y) + (right * input.x)
	else:
		dir = Vector3(input.x, 0.0, input.y)
	dir = dir.normalized()

	if dir != Vector3.ZERO:
		velocity.x = dir.x * move_speed
		velocity.z = dir.z * move_speed
	else:
		velocity.x = move_toward(velocity.x, 0.0, move_speed)
		velocity.z = move_toward(velocity.z, 0.0, move_speed)

	move_and_slide()

	# Actualizar cooldowns
	if _melee_cooldown > 0:
		_melee_cooldown -= delta
	if _ranged_cooldown > 0:
		_ranged_cooldown -= delta

	# Respawn timer
	if _respawn_timer > 0:
		_respawn_timer -= delta
		if _respawn_timer <= 0:
			_respawn()
		return

	# Ataque cuerpo a cuerpo (Click izquierdo / ui_click)
	if Input.is_action_pressed("ui_click") and _melee_cooldown <= 0:
		_perform_melee_attack()
		_melee_cooldown = MELEE_COOLDOWN_MAX

	# Ataque a distancia (Click derecho / ui_right_click)
	if Input.is_action_pressed("ui_right_click") and _ranged_cooldown <= 0:
		_perform_ranged_attack()
		_ranged_cooldown = RANGED_COOLDOWN_MAX

	# Bloqueo/Defensa (Shift / ui_shift)
	if Input.is_action_just_pressed("ui_shift"):
		_perform_block()

	# Notificar al manager de chunks de la nueva posición.
	var world := get_node_or_null("/root/NexusWorld")
	if world != null and world.has_method("_notify_player_pos"):
		world._notify_player_pos(global_position)

func _update_max_health() -> void:
	if health == null:
		return
	var shielding_level = skill_manager.get_skill_level(3)  # SHIELDING
	var base_hp = 100.0
	var bonus_hp = max(shielding_level - 10, 0) * 5.0
	health.set_max_health(base_hp + bonus_hp)

func _on_health_changed(current: float, max_hp: float) -> void:
	# Actualizar HUD
	emit_signal("health_changed", current, max_hp)

func _on_died() -> void:
	print("[PLAYER] Jugador muerto, respawning en 3s...")
	_respawn_timer = 3.0
	
	# Desactivar colisiones y movimiento
	set_collision_layer_value(2, false)
	set_collision_mask_value(1, false)
	velocity = Vector3.ZERO

func _respawn() -> void:
	# Buscar zona segura (ciudad)
	var safe_zone = get_tree().get_first_node_in_group("safe_zone")
	var spawn_pos = Vector3.ZERO
	
	if safe_zone != null:
		spawn_pos = safe_zone.global_position + Vector3.UP * 2.0
	else:
		# Fallback: origen del mundo
		spawn_pos = Vector3(0, 5, 0)
	
	global_position = spawn_pos
	if health != null:
		health.heal(health.max_health)
	
	# Reactivar colisiones
	set_collision_layer_value(2, true)
	set_collision_mask_value(1, true)
	
	print("[PLAYER] Respawn en ciudad")

func _perform_melee_attack() -> void:
	print("[NEXUS-COMBAT] Ataque cuerpo a cuerpo")

	# Raycast para detectar enemigo en rango corto
	var space_state = get_world_3d().direct_space_state
	var from = global_position + Vector3.UP * 1.5
	var to = from + (-global_transform.basis.z * 3.0)  # 3m rango corto
	var query = PhysicsRayQueryParameters3D.create(from, to)
	query.collision_mask = 4  # capa "entidades"
	query.exclude = [get_rid()]
	var result = space_state.intersect_ray(query)

	if result:
		var target = result.collider
		if target != null and target.has_method("take_damage"):
			# Calcular daño con skill
			var skill_level = skill_manager.get_skill_level(1)  # CLOSE_COMBAT
			var actual_damage = combat_manager.deal_damage(self, target, MELEE_BASE_DAMAGE, 0)  # SHORT range
			skill_manager.on_melee_hit()
			melee_sound.play() # Play melee sound
			_create_hit_effect(actual_damage, false, result.position) # Pass actual_damage and crit
	else:
		# Sin objetivo, solo registrar para XP
		skill_manager.on_melee_hit()
		melee_sound.play() # Play melee sound even if no target

func _perform_ranged_attack() -> void:
	print("[NEXUS-COMBAT] Ataque a distancia")

	# Disparar proyectil
	var direction = -global_transform.basis.z
	Projectile.shoot(self, direction, RANGED_BASE_DAMAGE, 1)  # MEDIUM range
	skill_manager.on_ranged_hit()
	ranged_sound.play() # Play ranged sound

func _perform_block() -> void:
	print("[NEXUS-COMBAT] Bloqueo/Defensa")
	# El bloqueo reduce daño recibido temporalmente
	skill_manager.on_block()

func _create_hit_effect(damage_amount: float, crit: bool, pos: Vector3) -> void:
	var combat_feedback_scene = preload("res://scenes/ui/combat_feedback.tscn")
	var combat_feedback_instance = combat_feedback_scene.instantiate()
	get_tree().root.add_child(combat_feedback_instance)
	combat_feedback_instance.init(damage_amount, crit, pos)

func _find_camera() -> Camera3D:
	var root := get_tree().current_scene
	if root != null:
		return root.get_node_or_null("Camera3D")
	return null

# Señal para HUD
signal health_changed(current: float, max: float)