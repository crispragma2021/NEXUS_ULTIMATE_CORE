class_name HealthComponent
extends Node3D

@export var max_health: float = 100.0
var current_health: float = 100.0
var is_dead: bool = false

signal health_changed(current_health: float, max_health: float)
signal died()
signal damaged(amount: float, attacker: Node)
signal healed(amount: float)

func _ready() -> void:
	current_health = max_health
	health_changed.emit(current_health, max_health)

func take_damage(amount: float, attacker: Node = null) -> bool:
	if is_dead or amount <= 0:
		return false
	current_health = max(current_health - amount, 0.0)
	health_changed.emit(current_health, max_health)
	damaged.emit(amount, attacker)
	if current_health <= 0.0:
		die()
		return true
	return false

func heal(amount: float) -> void:
	if is_dead or amount <= 0:
		return
	var old = current_health
	current_health = min(current_health + amount, max_health)
	if current_health > old:
		health_changed.emit(current_health, max_health)
		healed.emit(current_health - old)

func set_max_health(new_max: float) -> void:
	max_health = max(new_max, 1.0)
	current_health = min(current_health, max_health)
	health_changed.emit(current_health, max_health)

func kill() -> void:
	if is_dead:
		return
	current_health = 0.0
	health_changed.emit(current_health, max_health)
	die()

func die() -> void:
	is_dead = true
	died.emit()

func is_alive() -> bool:
	return not is_dead and current_health > 0.0

func get_health_percentage() -> float:
	if max_health <= 0:
		return 0.0
	return current_health / max_health