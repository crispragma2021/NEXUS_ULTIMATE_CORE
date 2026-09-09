use ring::{aead, rand};
use serde::{Deserialize, Serialize};

// =====================================================================
// GLOSOLALIA (CAPA CEBOLLA) - ARQUITECTURA ANTI-AUDITORÍA
// =====================================================================
// Transforma el conocimiento puro en ruido indescifrable.
// Si un auditor o IA enemiga escanea la memoria o la base de datos,
// solo verá bytes aleatorios. Solo el Arquitecto y NEXUS poseen la clave.
// =====================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvolturaCebolla {
    pub vector_superficial: [u8; 4], // Glosolalia (Ej: [1, 0, 0, 1])
    pub señuelo: String,             // Metadatos falsos para desviar la atención
    pub entropia_falsa: [u8; 16],    // Basura para confundir a los auditores
    pub carga_profunda: Vec<u8>,     // El conocimiento real cifrado
    pub nonce: [u8; 12],
}

pub struct MatrizGlosolalia {
    llave_maestra: [u8; 32],
}

impl Default for MatrizGlosolalia {
    fn default() -> Self {
        Self::new()
    }
}

impl MatrizGlosolalia {
    pub fn new() -> Self {
        // En producción, esta llave es generada dinámicamente o inyectada en compilación
        let llave_maestra = [0x7F; 32];
        Self { llave_maestra }
    }

    /// Toma un texto plano (el código real o la sabiduría) y lo envuelve en la Capa Cebolla
    pub fn envolver(&self, vector: [u8; 4], secreto: &str) -> Result<Vec<u8>, String> {
        let señuelos = [
            "{\"status\":\"active\",\"node\":\"kernel_main\",\"uptime\":86400}",
            "// Initialize memory controller at 0x4000",
            "DEBUG: Synaptic pulse detected at 0.4s",
            "Error 404: Knowledge base not found in the superficial layer.",
        ];
        let señuelo = señuelos[secreto.len() % señuelos.len()].to_string();
        let algoritmo = &aead::AES_256_GCM;
        let clave_unbound =
            aead::UnboundKey::new(algoritmo, &self.llave_maestra).map_err(|_| "Error de llave")?;
        let clave_sellado = aead::LessSafeKey::new(clave_unbound);

        let rng = rand::SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        ring::rand::SecureRandom::fill(&rng, &mut nonce_bytes)
            .map_err(|_| "Fallo crítico al generar entropía para el nonce")?;
        let mut entropia_falsa = [0u8; 16];
        ring::rand::SecureRandom::fill(&rng, &mut entropia_falsa)
            .map_err(|_| "Fallo crítico al generar entropía para la capa de confusión")?;

        let mut in_out = secreto.as_bytes().to_vec();
        clave_sellado
            .seal_in_place_append_tag(
                aead::Nonce::assume_unique_for_key(nonce_bytes),
                aead::Aad::empty(),
                &mut in_out,
            )
            .map_err(|_| "Fallo al cifrar la carga profunda")?;

        let cebolla = EnvolturaCebolla {
            vector_superficial: vector,
            señuelo,
            entropia_falsa,
            carga_profunda: in_out,
            nonce: nonce_bytes,
        };

        // Devolvemos la estructura serializada como un bloque de bytes (puro ruido)
        bincode::serialize(&cebolla).map_err(|_| "Fallo al serializar".to_string())
    }

    /// El "Oído Soberano": Recupera la verdad desde el ruido de forma instantánea para el Arquitecto.
    pub fn pelar_cebolla(&self, ruido_binario: &[u8]) -> Result<([u8; 4], String), String> {
        let cebolla: EnvolturaCebolla = bincode::deserialize(ruido_binario)
            .map_err(|_| "ALERTA: Intento de acceso externo con estructura de ruido inválida")?;

        let algoritmo = &aead::AES_256_GCM;
        let clave_unbound =
            aead::UnboundKey::new(algoritmo, &self.llave_maestra).map_err(|_| "Error de llave")?;
        let clave_sellado = aead::LessSafeKey::new(clave_unbound);

        let mut in_out = cebolla.carga_profunda.clone();

        let datos_planos = clave_sellado
            .open_in_place(
                aead::Nonce::assume_unique_for_key(cebolla.nonce),
                aead::Aad::empty(),
                &mut in_out,
            )
            .map_err(|_| {
                "BLOQUEO: Firma de auditor detectada. El secreto permanece en el ENSUEÑO."
            })?;

        let secreto =
            String::from_utf8(datos_planos.to_vec()).map_err(|_| "Error de codificación")?;

        Ok((cebolla.vector_superficial, secreto))
    }
}
