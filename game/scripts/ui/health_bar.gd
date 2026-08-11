extends Control

class_name HealthBar

@export var max_health: float = 100.0:
    set(value):
        max_health = max(1.0, value)
        update_bar()
@export var current_health: float = 100.0:
    set(value):
        current_health = clamp(value, 0.0, max_health)
        update_bar()

@onready var health_progress_bar = $HealthProgressBar

func _ready():
    update_bar()

func update_bar():
    if health_progress_bar:
        health_progress_bar.value = current_health
        health_progress_bar.max_value = max_health

func _on_health_changed(new_health: float):
    current_health = new_health
    update_bar()
