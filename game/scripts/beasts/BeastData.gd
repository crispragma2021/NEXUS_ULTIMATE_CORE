#!/usr/bin/env -S godot --script
# BeastData — Resource con estadísticas de bestia
# Se usa para configurar tipos de bestias en el editor

class_name BeastData
extends Resource

@export var species_name: String = "Lobo"
@export var level_min: int = 1
@export var level_max: int = 5
@export var base_health: float = 50.0
@export var base_damage: float = 5.0
@export var attack_range: float = 2.0
@export var move_speed: float = 3.0
@export var detection_radius: float = 15.0
@export var chase_radius: float = 40.0
@export var flee_health_threshold: float = 0.2
@export var color: Color = Color(0.5, 0.5, 0.5)
@export var xp_reward: int = 1

# Biomas donde puede spawnear
@export var biomes: Array[String] = ["forest", "plains"]

# Tipo de ataque
enum AttackType { MELEE, RANGED, CHARGE }
@export var attack_type: int = AttackType.MELEE

# Tiempo entre ataques
@export var attack_cooldown: float = 1.5