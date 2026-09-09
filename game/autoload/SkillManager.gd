#!/usr/bin/env -S godot --script
# SkillManager — Autoload global que gestiona los 5 skills estilo Tibia
# Accesible desde cualquier script como SkillManager (autoload name: NexusSkillManager)

extends Node

# ─── CONSTANTES ──────────────────────────────────────────────────
enum SkillType {
	CLOSE_COMBAT,      # Combate cuerpo a cuerpo
	DISTANCE_FIGHTING, # Combate a distancia
	SHIELDING,         # Defensa / Bloqueo
	CORE_ENERGY,       # Energía / Psiónico / Hackeo
	SURVIVAL_HUNTING   # Supervivencia / Caza / Captura
}

const SKILL_NAMES = {
	SkillType.CLOSE_COMBAT: "Close Combat",
	SkillType.DISTANCE_FIGHTING: "Distance Fighting",
	SkillType.SHIELDING: "Shielding",
	SkillType.CORE_ENERGY: "Core Energy",
	SkillType.SURVIVAL_HUNTING: "Survival & Hunting"
}

# ─── ESTADO ──────────────────────────────────────────────────────
var skills: Dictionary = {}

# ─── INICIALIZACIÓN ──────────────────────────────────────────────

func _enter_tree() -> void:
	_initialize_skills()
	print("[NEXUS-SKILLS] SkillManager inicializado con 5 skills estilo Tibia")

func _initialize_skills() -> void:
	for skill_type in [SkillType.CLOSE_COMBAT, SkillType.DISTANCE_FIGHTING, SkillType.SHIELDING, SkillType.CORE_ENERGY, SkillType.SURVIVAL_HUNTING]:
		var name = SKILL_NAMES[skill_type]
		var sp = SkillProgress.new()
		sp.skill_name = name
		sp.level = 10  # Nivel inicial estilo Tibia
		sp.current_hits = 0
		sp.level_up.connect(_on_skill_level_up.bind(name))
		skills[skill_type] = sp

# ─── API PÚBLICA ─────────────────────────────────────────────────

# Registra un hit exitoso para un skill. Retorna true si subió de nivel.
func register_hit(skill_type: SkillType) -> bool:
	if not skills.has(skill_type):
		return false
	
	var sp: SkillProgress = skills[skill_type]
	return sp.add_hit()

# Obtiene el progreso actual de un skill
func get_skill_progress(skill_type: SkillType) -> Dictionary:
	if not skills.has(skill_type):
		return {}
	
	var sp: SkillProgress = skills[skill_type]
	return {
		"name": sp.skill_name,
		"level": sp.level,
		"current_hits": sp.current_hits,
		"hits_required": sp.get_hits_required(),
		"percentage": sp.get_progress_percentage()
	}

# Obtiene todos los skills (para UI)
func get_all_skills() -> Array[Dictionary]:
	var result = []
	for skill_type in [SkillType.CLOSE_COMBAT, SkillType.DISTANCE_FIGHTING, SkillType.SHIELDING, SkillType.CORE_ENERGY, SkillType.SURVIVAL_HUNTING]:
		result.append(get_skill_progress(skill_type))
	return result

# Obtiene nivel de un skill
func get_skill_level(skill_type: SkillType) -> int:
	if skills.has(skill_type):
		return skills[skill_type].level
	return 10

# Añade hits masivamente (para testing o recompensas de quest)
func add_hits(skill_type: SkillType, amount: int) -> void:
	if not skills.has(skill_type):
		return
	var sp: SkillProgress = skills[skill_type]
	for i in range(amount):
		sp.add_hit()

# Guarda estado completo (para SaveGame)
func save_state() -> Dictionary:
	var state = {}
	for skill_type in [SkillType.CLOSE_COMBAT, SkillType.DISTANCE_FIGHTING, SkillType.SHIELDING, SkillType.CORE_ENERGY, SkillType.SURVIVAL_HUNTING]:
		var sp: SkillProgress = skills[skill_type]
		state[skill_type] = sp.to_dict()
	return state

# Carga estado completo
func load_state(state: Dictionary) -> void:
	for skill_type in [SkillType.CLOSE_COMBAT, SkillType.DISTANCE_FIGHTING, SkillType.SHIELDING, SkillType.CORE_ENERGY, SkillType.SURVIVAL_HUNTING]:
		if state.has(skill_type):
			var sp: SkillProgress = SkillProgress.from_dict(state[skill_type])
			sp.level_up.connect(_on_skill_level_up.bind(SKILL_NAMES[skill_type]))
			skills[skill_type] = sp

# ─── SEÑALES ─────────────────────────────────────────────────────

signal skill_level_up(skill_name: String, new_level: int)
signal skill_progress_changed(skill_name: String, percentage: float)

func _on_skill_level_up(name: String, level: int) -> void:
	emit_signal("skill_level_up", name, level)
	print("[NEXUS-SKILLS] Señal: %s nivel %d" % [name, level])

# ─── HELPERS PARA COMBATE ────────────────────────────────────────

# Ataque cuerpo a cuerpo exitoso
func on_melee_hit() -> bool:
	return register_hit(SkillType.CLOSE_COMBAT)

# Ataque a distancia exitoso
func on_ranged_hit() -> bool:
	return register_hit(SkillType.DISTANCE_FIGHTING)

# Bloqueo/defensa exitoso
func on_block() -> bool:
	return register_hit(SkillType.SHIELDING)

# Uso de habilidad psiónica/energía/hackeo
func on_core_energy_use() -> bool:
	return register_hit(SkillType.CORE_ENERGY)

# Captura/rastreo/supervivencia exitosa
func on_survival_action() -> bool:
	return register_hit(SkillType.SURVIVAL_HUNTING)