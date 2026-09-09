pub struct GangliosBasales {
    pub habitos_exitosos: Vec<String>,
}

impl GangliosBasales {
    pub fn automatizar_exito(&mut self, accion: &str) {
        self.habitos_exitosos.push(accion.to_string());
        println!(
            "🧠 [GANGLIOS] Acción '{}' automatizada como HÁBITO.",
            accion
        );
    }
}

impl GangliosBasales {
    pub fn refinar_enter_trinity(&mut self) {
        self.habitos_exitosos.push("TRINITY_ENTER_CDP".to_string());
        println!("🧠 [GANGLIOS] Reflejo de Trinity Enter asimilado en el ADN.");
    }
}
