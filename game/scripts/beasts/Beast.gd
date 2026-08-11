#!/usr/bin/env -S godot --script
# Beast — Controlador de bestia con HealthComponent y IA

class_name Beast
extends CharacterBody3D

# Referencia a datos de la bestia
@export var beast_data: BeastData = null

# Componentes
@onready var health: HealthComponent = get_node("HealthComponent")
@onready var ai: BeastAI = get_node("BeastAI")
@onready var mesh: MeshInstance3D = get_node("MeshInstance3D")
@onready var detection_area: Area3D = get_node("DetectionArea")
@onready var collision_shape: CollisionShape3D = get_node("CollisionShape3D")
@onready var combat_manager = get_node("/root/NexusCombatManager")

# Estado
var current_level: int = 1
var is_aggressive: bool = true
var is_dead: bool = false # Añadido para el AI
var home_position: Vector3 = Vector3.ZERO

func _ready() -> void:
	# Configurar capa de colisión (entidades)
	collision_layer = 4  # capa "entidades"
	collision_mask = 1 | 2  # colisiona con terreno y jugador
	
	home_position = global_position
	
	# Inicializar health
	if beast_data != null:
		_initialize_from_data()
	else:
		health.max_health = 50.0
		health.current_health = 50.0
	
	# Conectar señales
	health.died.connect(_on_died)
	health.damaged.connect(_on_damaged)
	ai.attack_request.connect(_on_attack_performed) # Conectar la señal de ataque de la IA
	
	# Aplicar color
	if mesh != null and beast_data != null:
		var mat = mesh.get_surface_override_material(0)
		if mat == null:
			mat = StandardMaterial3D.new()
			mesh.set_surface_override_material(0, mat)
		mat.albedo_color = beast_data.color
	
	# Iniciar IA
	if ai != null:
		ai.initialize(self, beast_data)

func _initialize_from_data() -> void:
	if beast_data == null:
		return
	
	# Nivel aleatorio entre min y max
	current_level = randi_range(beast_data.level_min, beast_data.level_max)
	
	# Escalar stats por nivel
	var level_mult = 1.0 + (current_level - 1) * 0.15
	health.max_health = beast_data.base_health * level_mult
	health.current_health = health.max_health

func _on_died() -> void:
	var species := beast_data.species_name if beast_data != null else "Bestia"
	print("[BEAST] %s nivel %d murió" % [species, current_level])
	
	is_dead = true # Marcar como muerto
	
	# Emitir señal de muerte para XP/loot
	emit_signal("beast_died", self, current_level)
	
	# Desactivar IA y colisiones
	if ai != null:
		ai.set_state(BeastAI.State.DEAD)
	set_collision_layer_value(4, false)
	set_collision_mask_value(1, false)
	set_collision_mask_value(2, false)
	
	# Animación de muerte / desaparecer
	var tween = create_tween()
	if mesh != null:
		tween.tween_property(mesh.material_override, "albedo_color:a", 0.0, 1.0)
	tween.tween_callback(queue_free.bind()).set_delay(2.0)

func take_damage(amount: float, attacker: Node, range: int = 0) -> void:
	if health != null:
		health.take_damage(amount, attacker)

func _on_damaged(amount: float, attacker: Node) -> void:
	if ai != null:
		ai.on_damaged(attacker)

func _on_attack_performed(target_node: Node) -> void:
	if beast_data == null or not is_instance_valid(target_node):
		return
	
	var base_damage = beast_data.base_damage
	var range_type = 0 # Asumiendo ataque cuerpo a cuerpo para bestias por ahora
	
	combat_manager.deal_damage(self, target_node, base_damage, range_type)
	
# Señales
signal beast_died(beast: Beast, level: int)
signal beast_attacked(beast: Beast, target: Node, damage: float)