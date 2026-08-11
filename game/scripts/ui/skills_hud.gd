#!/usr/bin/env -S godot --script
# SkillsHUD — Barras de progreso estilo Tibia para los 5 skills
# Se instancia en la escena principal (Main.tscn) como hijo del CanvasLayer/UI

class_name SkillsHUD
extends Control

@onready var skill_container: VBoxContainer = $VBoxContainer
@onready var template_bar: HBoxContainer = $VBoxContainer/SkillBarTemplate

# Referencias a las barras creadas dinámicamente
var skill_bars: Dictionary = {}

func _ready() -> void:
	_build_skill_ui()
	_connect_signals()
	_update_all_bars()

func _build_skill_ui() -> void:
	# Ocultar template
	template_bar.visible = false
	
	var sm = get_node_or_null("/root/NexusSkillManager")
	if sm == null:
		printerr("[SKILLS-HUD] NexusSkillManager no encontrado")
		return
	
	var skill_order = [
		sm.SkillType.CLOSE_COMBAT,
		sm.SkillType.DISTANCE_FIGHTING,
		sm.SkillType.SHIELDING,
		sm.SkillType.CORE_ENERGY,
		sm.SkillType.SURVIVAL_HUNTING
	]
	
	for skill_type in skill_order:
		var skill_name = sm.SKILL_NAMES[skill_type]
		var bar_instance = _create_skill_bar(skill_name, skill_type)
		skill_bars[skill_type] = bar_instance
		skill_container.add_child(bar_instance)

func _create_skill_bar(skill_name: String, _skill_type: int) -> HBoxContainer:
	var bar = template_bar.duplicate() as HBoxContainer
	bar.visible = true
	bar.name = "SkillBar_%s" % skill_name
	
	# Configurar label del nombre
	var name_label = bar.get_node("NameLabel")
	if name_label:
		name_label.text = skill_name
	
	# Configurar label del nivel
	var level_label = bar.get_node("LevelLabel")
	if level_label:
		level_label.text = "10"
	
	# Configurar ProgressBar
	var progress_bar = bar.get_node("ProgressBar")
	if progress_bar:
		progress_bar.value = 0
		progress_bar.max_value = 100
	
	return bar

func _connect_signals() -> void:
	var sm = get_node_or_null("/root/NexusSkillManager")
	if sm and sm.has_signal("skill_level_up"):
		sm.skill_level_up.connect(_on_skill_level_up)
	if sm and sm.has_signal("skill_progress_changed"):
		sm.skill_progress_changed.connect(_on_skill_progress_changed)

func _on_skill_level_up(skill_name: String, new_level: int) -> void:
	# Actualizar label de nivel
	for skill_type in skill_bars:
		var bar = skill_bars[skill_type]
		var name_label = bar.get_node("NameLabel")
		if name_label and name_label.text == skill_name:
			var level_label = bar.get_node("LevelLabel")
			if level_label:
				level_label.text = str(new_level)
			# Efecto visual de level up
			_play_level_up_effect(bar)
			break

func _on_skill_progress_changed(skill_name: String, percentage: float) -> void:
	# Actualizar barra de progreso
	for skill_type in skill_bars:
		var bar = skill_bars[skill_type]
		var name_label = bar.get_node("NameLabel")
		if name_label and name_label.text == skill_name:
			var progress_bar = bar.get_node("ProgressBar")
			if progress_bar:
				progress_bar.value = percentage
			break

func _update_all_bars() -> void:
	var sm = get_node_or_null("/root/NexusSkillManager")
	if not sm:
		return
	for skill_type in skill_bars:
		var bar = skill_bars[skill_type]
		var name_label = bar.get_node("NameLabel")
		if name_label:
			var progress = sm.get_skill_progress(skill_type)
			var level_label = bar.get_node("LevelLabel")
			var progress_bar = bar.get_node("ProgressBar")
			
			if level_label:
				level_label.text = str(progress.get("level", 10))
			if progress_bar:
				progress_bar.value = progress.get("percentage", 0.0)

func _play_level_up_effect(bar: HBoxContainer) -> void:
	# Parpadeo dorado en la barra
	var progress_bar = bar.get_node("ProgressBar")
	if progress_bar:
		var tween = create_tween()
		tween.tween_property(progress_bar, "modulate", Color.GOLD, 0.1)
		tween.tween_property(progress_bar, "modulate", Color.WHITE, 0.1)
		tween.tween_property(progress_bar, "modulate", Color.GOLD, 0.1)
		tween.tween_property(progress_bar, "modulate", Color.WHITE, 0.1)
	
	# Sonido si existe
	var level_sound = bar.get_node_or_null("LevelUpSound")
	if level_sound:
		level_sound.play()