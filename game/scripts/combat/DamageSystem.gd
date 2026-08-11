#!/usr/bin/env -S godot --script
# DamageSystem — Cálculo centralizado de daño
# Funciones puras, sin estado, para testabilidad

class_name DamageSystem
extends RefCounted

# Tipos de daño
enum DamageType { PHYSICAL, MAGIC, POISON, FIRE, COLD }

# Fórmula unificada de daño
# base_damage: daño base del arma/habilidad
# attacker_level: nivel del atacante (para escalado)
# skill_level: nivel de skill relevante (Tibia style, base 10)
# target_defense: defensa del objetivo (reduce daño %)
# damage_type: tipo de daño (para resistencias futuras)
# range_multiplier: multiplicador por rango (0.0 - 1.0+)
# critical_chance: probabilidad de crítico (0.0 - 1.0)
# critical_multiplier: multiplicador de crítico (default 2.0)
static func calculate_damage(
	base_damage: float,
	attacker_level: int = 1,
	skill_level: int = 10,
	target_defense: float = 0.0,
	damage_type: int = DamageType.PHYSICAL,
	range_multiplier: float = 1.0,
	critical_chance: float = 0.0,
	critical_multiplier: float = 2.0
) -> Dictionary:
	
	# 1. Escalado por nivel de atacante (0.5% por nivel)
	var level_mult = 1.0 + attacker_level * 0.005
	
	# 2. Escalado por skill (1% por nivel sobre base 10)
	var skill_mult = 1.0 + max(skill_level - 10, 0) * 0.01
	
	# 3. Daño antes de defensa
	var pre_defense = base_damage * level_mult * skill_mult * range_multiplier
	
	# 4. Reducción por defensa (cada punto = 0.5% reducción, max 75%)
	var defense_reduction = min(target_defense * 0.005, 0.75)
	var after_defense = pre_defense * (1.0 - defense_reduction)
	
	# 5. Crítico
	var is_critical = false
	if critical_chance > 0.0 and randf() < critical_chance:
		after_defense *= critical_multiplier
		is_critical = true
	
	# 6. Mínimo 1 de daño si hay ataque
	var final_damage = max(after_defense, 1.0) if base_damage > 0.0 else 0.0
	
	return {
		"damage": int(final_damage),
		"raw_damage": base_damage,
		"level_mult": level_mult,
		"skill_mult": skill_mult,
		"range_mult": range_multiplier,
		"defense_reduction": defense_reduction,
		"is_critical": is_critical,
		"damage_type": damage_type
	}

# Calcula multiplicador de rango
static func get_range_multiplier(weapon_range: int, distance: float) -> float:
	# weapon_range: 0=SHORT, 1=MEDIUM, 2=LONG
	var range_max = [3.0, 15.0, 40.0]
	var max_dist = range_max[weapon_range] if weapon_range < range_max.size() else 40.0
	
	if distance <= max_dist:
		return 1.0
	else:
		# Penalización por distancia excesiva
		var excess = distance - max_dist
		return max(1.0 - excess * 0.05, 0.0)

# Genera número de daño para UI
static func format_damage_number(damage_dict: Dictionary) -> String:
	var dmg = damage_dict.damage
	if damage_dict.is_critical:
		return "CRIT %d!" % [dmg]
	return str(dmg)