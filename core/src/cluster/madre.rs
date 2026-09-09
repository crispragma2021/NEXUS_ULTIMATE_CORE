pub struct NexusMother {
    pub total_nodos_activos: u32,
    pub puerto_cluster: u16,
}

impl NexusMother {
    pub fn new() -> Self {
        Self { total_nodos_activos: 0, puerto_cluster: 43230 }
    }

    // IGNICIÓN DEL CLUSTER (Escucha de Hijos)
    pub fn iniciar_escucha(&mut self) {
        println!("🛰️ [CLUSTER] Servidor Madre Activo en Puerto {}. Esperando Nodos Hijos...", self.puerto_cluster);
    }

    // VINCULACIÓN DE UN HIJO (Mobile / Remote)
    pub fn vincular_hijo(&mut self, dispositivo: &str) {
        self.total_nodos_activos += 1;
        println!("🦾 [CLUSTER] Nuevo Nodo Hijo Vinculado: {}. Total Nodos: {}", dispositivo, self.total_nodos_activos);
    }
}
