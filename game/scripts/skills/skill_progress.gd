#!/usr/bin/env -S godot --script
# SkillProgress — Skill al estilo Tibia (hits, nivel, progreso %)
# Se usa como recurso dentro de SkillManager

class_name SkillProgress
extends Resource

@export var skill_name: String = ""
@export var level: int = 10
@export var current_hits: int = 0

# Fórmula de escala Tibia: hits requeridos crecen exponencialmente
func _get_hits_required_for_level(lvl: int) -> int:
	if lvl <= 20:
		# Progresión ultra rápida al inicio (3-15 hits por nivel)
		return lvl * 3
	elif lvl <= 50:
		# Curva media
		return lvl * 12
	else:
		# Curva alto nivel (exponencial suave)
		return int(pow(lvl, 1.8) * 5.0)

func get_hits_required() -> int:
	return _get_hits_required_for_level(level)

func get_progress_percentage() -> float:
	var req = get_hits_required()
	if req == 0:
		return 0.0
	return (current_hits as float / req) * 100.0

# Registra un impacto exitoso (hit). Retorna true si subió de nivel.
func add_hit() -> bool:
	current_hits += 1
	var required = get_hits_required()
	
	if current_hits >= required:
		level += 1
		current_hits = 0
		print("[NEXUS-SKILL] ¡ADVANCE! %s subió al nivel %d" % [skill_name, level])
		emit_signal("level_up", skill_name, level)
		return true
	return false

# Para guardar/cargar estado
func to_dict() -> Dictionary:
	return {
		"skill_name": skill_name,
		"level": level,
		"current_hits": current_hits
	}

static func from_dict(data: Dictionary) -> SkillProgress:
	var sp = SkillProgress.new()
	sp.skill_name = data.get("skill_name", "")
	sp.level = data.get("level", 10)
	sp.current_hits = data.get("current_hits", 0)
	return sp

# Señal para UI
signal level_up(skill_name: String, new_level: int)