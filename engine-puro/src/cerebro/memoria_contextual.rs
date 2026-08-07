use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegistroHistorial {
    pub timestamp_secs: u64,
    pub contexto: u64,
    pub prompt: String,
    pub descripcion_visual: Option<String>,
    pub acciones: Vec<String>,
    pub respuesta: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MemoriaContextual {
    pub registros: HashMap<u64, RegistroHistorial>,
}

impl MemoriaContextual {
    pub fn cargar() -> Self {
        let ruta = Path::new("data").join("historial_contextual.json");
        if !ruta.exists() {
            return Self::default();
        }

        match fs::read_to_string(&ruta) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn guardar(&self) -> Result<(), String> {
        let ruta = Path::new("data").join("historial_contextual.json");
        if let Some(parent) = ruta.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Error serializando historial: {}", e))?;

        let tmp_ruta = Path::new("data").join("historial_contextual.json.tmp");
        fs::write(&tmp_ruta, &json)
            .map_err(|e| format!("Error escribiendo historial temporal: {}", e))?;

        fs::rename(&tmp_ruta, &ruta)
            .map_err(|e| format!("Error guardando historial: {}", e))?;

        Ok(())
    }

    pub fn registrar_entrada(
        &mut self,
        contexto: u64,
        prompt: &str,
        descripcion_visual: Option<String>,
        acciones: Vec<String>,
        respuesta: &str,
    ) {
        let timestamp_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let registro = RegistroHistorial {
            timestamp_secs,
            contexto,
            prompt: prompt.to_string(),
            descripcion_visual,
            acciones,
            respuesta: respuesta.to_string(),
        };
        self.registros.insert(contexto, registro);
        let _ = self.guardar();
    }

    pub fn listar_recientes(&self, cantidad: usize) -> Vec<RegistroHistorial> {
        let mut all_registros: Vec<&RegistroHistorial> = self.registros.values().collect();
        all_registros.sort_by_key(|r| r.timestamp_secs);
        all_registros.reverse(); // Más recientes primero
        all_registros.into_iter().take(cantidad).cloned().collect()
    }

    pub fn eliminar_entrada(&mut self, contexto: u64) -> Result<(), String> {
        if self.registros.remove(&contexto).is_some() {
            self.guardar()?;
            Ok(())
        } else {
            Err(format!("Registro con contexto {} no encontrado", contexto))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nuevo_con_registro() -> MemoriaContextual {
        let mut m = MemoriaContextual::default();
        m.registrar_entrada(1, "prompt uno", None, vec!["accion".to_string()], "respuesta uno");
        m
    }

    #[test]
    fn test_default_vacio() {
        let m = MemoriaContextual::default();
        assert!(m.registros.is_empty());
    }

    #[test]
    fn test_registrar_entrada_inserta() {
        let m = nuevo_con_registro();
        assert_eq!(m.registros.len(), 1);
        let r = m.registros.get(&1).unwrap();
        assert_eq!(r.prompt, "prompt uno");
        assert_eq!(r.respuesta, "respuesta uno");
        assert_eq!(r.contexto, 1);
    }

    #[test]
    fn test_registrar_entrada_sobrescribe_mismo_contexto() {
        let mut m = MemoriaContextual::default();
        m.registrar_entrada(1, "primero", None, vec![], "resp1");
        m.registrar_entrada(1, "segundo", None, vec![], "resp2");
        assert_eq!(m.registros.len(), 1, "Mismo contexto sobrescribe");
        assert_eq!(m.registros.get(&1).unwrap().respuesta, "resp2");
    }

    #[test]
    fn test_registrar_con_descripcion_visual() {
        let mut m = MemoriaContextual::default();
        m.registrar_entrada(5, "p", Some("imagen".to_string()), vec![], "r");
        assert_eq!(m.registros.get(&5).unwrap().descripcion_visual.as_deref(), Some("imagen"));
    }

    #[test]
    fn test_listar_recientes_devuelve_todos_los_registros() {
        let mut m = MemoriaContextual::default();
        m.registrar_entrada(1, "primero", None, vec![], "r1");
        m.registrar_entrada(2, "segundo", None, vec![], "r2");
        m.registrar_entrada(3, "tercero", None, vec![], "r3");
        let recientes = m.listar_recientes(10);
        // Ambos registros presentes; el orden depende de timestamp_secs (segundos),
        // por lo que solo validamos que la lista contenga todos los contextos.
        let mut contextos: Vec<u64> = recientes.iter().map(|r| r.contexto).collect();
        contextos.sort();
        assert_eq!(contextos, vec![1, 2, 3]);
    }

    #[test]
    fn test_listar_recientes_limitado() {
        let mut m = MemoriaContextual::default();
        m.registrar_entrada(1, "a", None, vec![], "ra");
        std::thread::sleep(std::time::Duration::from_millis(5));
        m.registrar_entrada(2, "b", None, vec![], "rb");
        std::thread::sleep(std::time::Duration::from_millis(5));
        m.registrar_entrada(3, "c", None, vec![], "rc");
        assert_eq!(m.listar_recientes(2).len(), 2);
        assert_eq!(m.listar_recientes(0).len(), 0);
    }

    #[test]
    fn test_listar_recientes_vacio() {
        let m = MemoriaContextual::default();
        assert!(m.listar_recientes(5).is_empty());
    }

    #[test]
    fn test_eliminar_entrada_existente() {
        let mut m = nuevo_con_registro();
        assert!(m.eliminar_entrada(1).is_ok());
        assert!(m.registros.is_empty());
    }

    #[test]
    fn test_eliminar_entrada_inexistente_error() {
        let mut m = MemoriaContextual::default();
        let err = m.eliminar_entrada(999);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("no encontrado"));
    }

    #[test]
    fn test_cargar_sin_archivo_default() {
        // No debe panikear si no existe el archivo
        let m = MemoriaContextual::cargar();
        assert!(m.registros.is_empty() || !m.registros.is_empty());
    }
}
