class_name Projectile
extends Area3D

@export var speed: float = 20.0
@export var damage: float = 10.0
@export var range: int = 0  # 0: corto, 1: medio, 2: largo
@export var life_time: float = 5.0

var direction: Vector3 = Vector3.FORWARD
var owner_node: Node = null

func _ready():
	set_as_top_level(true) # Para que no herede escala ni rotación del padre
	if has_node("Timer"):
		$Timer.wait_time = life_time
		$Timer.start()

func _physics_process(delta: float):
	global_position += direction * speed * delta

func _on_area_entered(area: Area3D):
	if area == owner_node: return # Ignorar colisión con el que dispara

	if area.has_method("take_damage"):
		area.take_damage(damage, owner_node, range)

	queue_free()

func _on_body_entered(body: Node3D):
	if body == owner_node: return # Ignorar colisión con el que dispara

	if body.has_method("take_damage"):
		body.take_damage(damage, owner_node, range)

	queue_free()

func _on_timer_timeout():
	queue_free()

# Método estático para disparar un proyectil
static func shoot(shooter: Node, dir: Vector3, base_damage: float, combat_range: int):
	var projectile_scene = preload("res://scenes/combat/projectile.tscn")
	var projectile_instance = projectile_scene.instantiate()

	projectile_instance.global_position = shooter.global_position + Vector3.UP * 1.5 # Altura media del personaje
	projectile_instance.direction = dir.normalized()
	projectile_instance.damage = base_damage
	projectile_instance.owner_node = shooter
	projectile_instance.range = combat_range

	# Añadir a la escena actual
	var tree = shooter.get_tree()
	if tree != null and tree.current_scene != null:
		tree.current_scene.add_child(projectile_instance)
