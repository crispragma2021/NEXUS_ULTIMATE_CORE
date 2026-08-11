//! godot_controller_cli.rs — CLI de prueba para el puente LLM ↔ Godot.
//!
//! Uso:
//!   cargo run --bin godot-controller -- health
//!   cargo run --bin godot-controller -- player
//!   cargo run --bin godot-controller -- spawn lobo 5 -3
//!   cargo run --bin godot-controller -- damage 25
//!   cargo run --bin godot-controller -- heal 50
//!   cargo run --bin godot-controller -- move 0 -10

mod godot_controller; // reutiliza el controlador de src-tauri/src/godot_controller.rs

use godot_controller::GodotController;
use serde_json::{json, Value};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <comando> [args...]", args.get(0).map(|s| s.as_str()).unwrap_or("godot-controller"));
        eprintln!("  health | player | scene | spawn <especie> <x> <z> | kill");
        eprintln!("  damage <cant> | heal <cant> | move <x> <z>");
        return ExitCode::from(2);
    }

    let ctrl = GodotController::default();
    let cmd = args[1].as_str();

    let result: Result<Value, String> = match cmd {
        "health" => ctrl.health().await,
        "player" => ctrl.get_player().await,
        "scene" => ctrl.get_scene_tree().await,
        "spawn" => {
            if args.len() < 5 {
                Err("spawn necesita: <especie> <x> <z>".into())
            } else {
                ctrl.spawn_beast(&args[2], args[3].parse().unwrap_or(0.0), args[4].parse().unwrap_or(0.0)).await
            }
        }
        "kill" => ctrl.kill_beasts().await,
        "damage" => {
            let amt: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10.0);
            ctrl.damage_player(amt).await
        }
        "heal" => {
            let amt: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(999.0);
            ctrl.heal_player(amt).await
        }
        "move" => {
            if args.len() < 4 {
                Err("move necesita: <x> <z>".into())
            } else {
                let x: f32 = args[2].parse().unwrap_or(0.0);
                let z: f32 = args[3].parse().unwrap_or(0.0);
                ctrl.move_player(x, z).await
            }
        }
        other => ctrl.command(other, json!({})).await,
    };

    match result {
        Ok(v) => {
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_else(|_| v.to_string()));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            ExitCode::from(1)
        }
    }
}
