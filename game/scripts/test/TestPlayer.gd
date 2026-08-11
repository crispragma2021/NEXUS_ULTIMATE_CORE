class_name TestPlayer
extends CharacterBody3D

var health_component: HealthComponent

func _ready():
    health_component = $HealthComponent as HealthComponent
    if health_component == null:
        print("Test ERROR: HealthComponent no pudo ser casteado. Tipo real: ", $HealthComponent.get_class())
    else:
        print("Test SUCCESS: HealthComponent cargado correctamente. Max health: ", health_component.max_health)
