## SafeZone — Zona segura (ciudad) donde el jugador está protegido
## NEXUS Protocol · Fase Beta · Área segura delimitada por muros de la ciudad

extends Area3D

signal player_entered_safe_zone
signal player_exited_safe_zone

func _ready() -> void:
	# Asegurarse de que el Area3D detecta cuerpos
	monitorable = true
	monitoring = true

func _on_body_entered(body: Node3D) -> void:
	if body.is_in_group("player"):
		player_entered_safe_zone.emit()

func _on_body_exited(body: Node3D) -> void:
	if body.is_in_group("player"):
		player_exited_safe_zone.emit()
