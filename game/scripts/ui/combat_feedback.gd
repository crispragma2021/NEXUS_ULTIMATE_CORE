class_name CombatFeedback
extends Label3D

func init(damage_amount: float, crit: bool, pos: Vector3):
	text = str(int(damage_amount))
	global_position = pos + Vector3.UP * 2.0
	billboard = BaseMaterial3D.BILLBOARD_ENABLED

	if crit:
		set("theme_override_colors/font_color", Color("ff8c00")) # Naranja para críticos
		font_size = 32
	else:
		set("theme_override_colors/font_color", Color("ffffff")) # Blanco normal
		font_size = 24

	var tween = create_tween()
	tween.tween_property(self, "global_position:y", pos.y + 4.0, 1.0)
	tween.tween_property(self, "modulate:a", 0.0, 1.0)
	tween.tween_callback(queue_free)
