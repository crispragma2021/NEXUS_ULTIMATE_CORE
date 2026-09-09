#!/usr/bin/env -S godot --script
# CombatManager — Autoload global de sistema de combate
# Accesible como NexusCombatManager

extends Node

# Rangos de combate
enum RangeType { SHORT, MEDIUM, LONG }

# Distancias en metros
const SHORT_MAX = 3.0
const MEDIUM_MAX = 15.0
const LONG_MAX = 40.0

# Multiplicadores por rango
const RANGE_MULTIPLIER = {
	RangeType.SHORT: 1.0,
	RangeType.MEDIUM: 1.0,
	RangeType.LONG: 1.0
}

func _ready() -> void:
	print("[NEXUS-COMBAT] CombatManager inicializado")

# Calcula el tipo de rango basado en distancia
func get_range_type(distance: float) -> int:
	if distance <= SHORT_MAX:
		return RangeType.SHORT
	elif distance <= MEDIUM_MAX:
		return RangeType.MEDIUM
	else:
		return RangeType.LONG

# Verifica si un atacante está en rango óptimo para su arma
func is_in_optimal_range(attacker_pos: Vector3, target_pos: Vector3, weapon_range: int) -> bool:
	var dist = attacker_pos.distance_to(target_pos)
	var range_type = get_range_type(dist)
	return range_type == weapon_range

# Calcula daño final
func calculate_damage(base_damage: float, skill_level: int, weapon_range: int, distance: float) -> float:
	var range_type = get_range_type(distance)
	
	# Multiplicador de rango
	var range_mult = 1.0
	if range_type != weapon_range:
		# Fuera de rango óptimo
		if weapon_range == RangeType.SHORT:
			range_mult = 0.0  # No alcanza
		elif weapon_range == RangeType.MEDIUM:
			range_mult = 0.3
		elif weapon_range == RangeType.LONG:
			range_mult = 0.0
	
	# Bonus por skill (1% por nivel sobre base 10)
	var skill_mult = 1.0 + max(skill_level - 10, 0) * 0.01
	
	return base_damage * skill_mult * range_mult

# Aplica daño a un objetivo
func deal_damage(attacker: Node, target: Node, base_damage: float, weapon_range: int) -> float:
	var attacker_pos = Vector3.ZERO
	var target_pos = Vector3.ZERO
	
	if attacker is Node3D:
		attacker_pos = attacker.global_position
	if target is Node3D:
		target_pos = target.global_position
	
	var distance = attacker_pos.distance_to(target_pos)
	var final_damage = calculate_damage(base_damage, _get_attacker_skill_level(attacker, weapon_range), weapon_range, distance)
	
	# Buscar HealthComponent en target
	var health = target.get_node_or_null("HealthComponent")
	if health == null and target is Node3D:
		# Buscar en hijos
		for child in target.get_children():
			if child is HealthComponent:
				health = child
				break
	
	if health != null and health.is_alive():
		health.take_damage(final_damage, attacker)
		return final_damage
	
	return 0.0

func _get_attacker_skill_level(attacker: Node, weapon_range: int) -> int:
	# Intentar obtener skill level del SkillManager
	var skill_mgr = get_node_or_null("/root/NexusSkillManager")
	if skill_mgr != null:
		match weapon_range:
			RangeType.SHORT:
				return skill_mgr.get_skill_level(1)  # CLOSE_COMBAT
			RangeType.MEDIUM, RangeType.LONG:
				return skill_mgr.get_skill_level(2)  # DISTANCE_FIGHTING
	return 10

# Obtiene distancia máxima de un rango
func get_range_max(range_type: int) -> float:
	match range_type:
		RangeType.SHORT: return SHORT_MAX
		RangeType.MEDIUM: return MEDIUM_MAX
		RangeType.LONG: return LONG_MAX
	return 0.0