// ============================================================================
// SISTEMA INMUNE COGNITIVO — Defensa Orgánica Soberana de NEXUS
// ============================================================================
// No depende de APIs externas. No envía datos a terceros.
// Aprende de cada amenaza. Se adapta con feedback del Arquitecto.
// Alimenta al subconsciente para que NEXUS "sienta" el peligro.
// ============================================================================

use crate::memoria::subconsciente::Subconsciente;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

// ─── CONSTANTES HOMEOSTÁTICAS ─────────────────────────────────────────────

/// Umbral por debajo del cual un veredicto es "Seguro"
const UMBRAL_SEGURO: f64 = 0.3;

/// Umbral por encima del cual un veredicto es "Peligroso"
const UMBRAL_PELIGROSO: f64 = 0.7;

/// Decaimiento de severidad por hora sin re-detección
const DECAIMIENTO_SEVERIDAD: f64 = 0.05;

/// Máximo de firmas en memoria (FIFO)
const MAX_FIRMAS: usize = 200;

// ─── TIPOS DE AMENAZA ─────────────────────────────────────────────────────

/// Clasificación del tipo de amenaza detectada.
#[derive(Debug, Clone, PartialEq)]
pub enum TipoAmenaza {
    /// URL maliciosa o con patrones sospechosos
    UrlMaliciosa,
    /// Dirección IP con reputación negativa o patrón anómalo
    Ipsospechosa,
    /// Archivo que podría contener malware
    ArchivoSospechoso,
    /// Patrón de texto/actividad que indica amenaza
    PatronHostil,
    /// Intento de suplantación (phishing, typosquatting)
    Suplantacion,
    /// Descarga de binario no verificado
    DescargaNoSegura,
}

impl TipoAmenaza {
    /// Peso base del tipo de amenaza (0.0 a 1.0)
    pub fn peso_base(&self) -> f64 {
        match self {
            Self::UrlMaliciosa => 0.7,
            Self::Ipsospechosa => 0.6,
            Self::ArchivoSospechoso => 0.8,
            Self::PatronHostil => 0.9,
            Self::Suplantacion => 0.85,
            Self::DescargaNoSegura => 0.65,
        }
    }

    pub fn descripcion(&self) -> &'static str {
        match self {
            Self::UrlMaliciosa => "URL maliciosa",
            Self::Ipsospechosa => "IP sospechosa",
            Self::ArchivoSospechoso => "archivo sospechoso",
            Self::PatronHostil => "patrón hostil detectado",
            Self::Suplantacion => "intento de suplantación",
            Self::DescargaNoSegura => "descarga no segura",
        }
    }
}

// ─── FIRMA DE AMENAZA ─────────────────────────────────────────────────────

/// Una amenaza conocida registrada en la memoria inmune.
#[derive(Debug, Clone)]
pub struct FirmaAmenaza {
    /// Hash único de la amenaza (SHA256 de URL/IP/archivo)
    pub hash: String,
    /// Tipo de amenaza
    pub tipo: TipoAmenaza,
    /// Severidad calculada (0.0 a 1.0)
    pub severidad: f64,
    /// Cuántas veces se ha detectado
    pub detecciones: u32,
    /// Timestamp Unix de la última detección
    pub ultima_deteccion: u64,
}

impl FirmaAmenaza {
    pub fn new(hash: &str, tipo: TipoAmenaza, severidad: f64) -> Self {
        let ahora = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            hash: hash.to_string(),
            tipo,
            severidad,
            detecciones: 1,
            ultima_deteccion: ahora,
        }
    }

    /// Aplica decaimiento temporal: la severidad baja con el tiempo
    /// si no hay re-detección. Retorna true si debe conservarse.
    pub fn decaer(&mut self) -> bool {
        let ahora = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let horas = (ahora.saturating_sub(self.ultima_deteccion)) as f64 / 3600.0;
        let decaimiento = horas * DECAIMIENTO_SEVERIDAD;
        self.severidad = (self.severidad - decaimiento).max(0.0);
        // Eliminar si ya no es relevante
        self.severidad > UMBRAL_SEGURO
    }
}

// ─── VEREDICTO ─────────────────────────────────────────────────────────────

/// Resultado del análisis inmunológico.
#[derive(Debug, Clone, PartialEq)]
pub enum Veredicto {
    /// Contenido seguro para procesar
    Seguro,
    /// Potencialmente peligroso, requiere precaución
    Sospechoso(f64),
    /// Confirmado como amenaza, debe bloquearse
    Peligroso(f64),
}

impl Veredicto {
    pub fn es_seguro(&self) -> bool {
        matches!(self, Self::Seguro)
    }

    pub fn nivel_alerta(&self) -> f64 {
        match self {
            Self::Seguro => 0.0,
            Self::Sospechoso(n) => *n,
            Self::Peligroso(n) => *n,
        }
    }
}

// ─── SISTEMA INMUNE ───────────────────────────────────────────────────────

/// Sistema inmune cognitivo de NEXUS.
///
/// # Homeostasis
/// - Detecta amenazas por heurística (sin API externa)
/// - Memoria de firmas con decaimiento temporal
/// - Umbrales adaptativos que evolucionan con feedback
/// - Alimenta al subconsciente para generar "intuición de peligro"
pub struct SistemaInmune {
    /// Memoria de amenazas conocidas
    firmas: Vec<FirmaAmenaza>,
    /// Umbral dinámico de sospecha (aprendido)
    umbral_sospechoso: f64,
    /// Umbral dinámico de peligro (aprendido)
    umbral_peligroso: f64,
    /// Nivel de alerta global (0.0 = calma, 1.0 = ataque)
    nivel_alerta: f64,
    /// Total de amenazas detectadas en la sesión
    total_amenazas: u64,
    /// Total de falsos positivos corregidos
    total_falsos_positivos: u64,
}

impl Default for SistemaInmune {
    fn default() -> Self {
        Self::new()
    }
}

impl SistemaInmune {
    pub fn new() -> Self {
        info!("🧬 [INMUNE] Sistema inmune cognitivo activado. Defensa soberana online.");
        Self {
            firmas: Vec::with_capacity(MAX_FIRMAS),
            umbral_sospechoso: UMBRAL_SEGURO,
            umbral_peligroso: UMBRAL_PELIGROSO,
            nivel_alerta: 0.0,
            total_amenazas: 0,
            total_falsos_positivos: 0,
        }
    }

    // ─── ANÁLISIS POR HEURÍSTICA ──────────────────────────────────────────

    /// Analiza una URL usando heurística local. Sin consultas externas.
    pub fn analizar_url(&mut self, url: &str) -> Veredicto {
        // 1. Buscar en memoria de amenazas
        if let Some(veredicto) = self.buscar_en_memoria(url) {
            return veredicto;
        }

        // 2. Heurística de URL
        let mut puntaje = 0.0_f64;
        let lower = url.to_lowercase();

        // TLDs sospechosos
        let tlds_riesgo = [
            ".tk",
            ".ml",
            ".ga",
            ".cf",
            ".gq",
            ".xyz",
            ".top",
            ".club",
            ".download",
            ".review",
            ".work",
            ".date",
            ".loan",
            ".men",
        ];
        for tld in &tlds_riesgo {
            if lower.contains(tld) {
                puntaje += 0.3;
                break;
            }
        }

        // Patrones de URL maliciosa
        let patrones_maliciosos = [
            "login",
            "secure",
            "account",
            "verify",
            "update",
            "confirm",
            "banking",
            "paypal",
            "bitcoin",
            "wallet",
            "signin",
            "auth",
            "password",
            "credential",
            "2fa",
            "verification",
        ];
        for patron in &patrones_maliciosos {
            if lower.contains(patron) {
                puntaje += 0.1;
            }
        }

        // IPs literales en URL (evitan DNS)
        if lower.starts_with("http://") || lower.starts_with("https://") {
            let resto = lower
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            let primera_parte = resto.split('/').next().unwrap_or("");
            if es_ip_literal(primera_parte) {
                puntaje += 0.4;
            }
        }

        // Subdominios excesivos
        let partes: Vec<&str> = lower.split('.').collect();
        if partes.len() > 5 {
            puntaje += 0.2;
        }

        // Caracteres extraños en el dominio
        if lower.contains("--") || lower.chars().filter(|c| c == &'-').count() > 4 {
            puntaje += 0.15;
        }

        self.evaluar_y_registrar(url, TipoAmenaza::UrlMaliciosa, puntaje)
    }

    /// Analiza una dirección IP por heurística.
    pub fn analizar_ip(&mut self, ip: &str) -> Veredicto {
        if let Some(veredicto) = self.buscar_en_memoria(ip) {
            return veredicto;
        }

        let mut puntaje = 0.0_f64;

        // IPs privadas intentando acceder como si fueran externas
        if ip.starts_with("10.") || ip.starts_with("192.168.") || ip.starts_with("172.16.") {
            puntaje += 0.1; // Bajo: pueden ser configuraciones internas
        }

        // IPs de rangos reservados/peligrosos
        if ip.starts_with("0.") || ip.starts_with("127.") || ip == "255.255.255.255" {
            puntaje += 0.6;
        }

        // IPs con reputación conocida (hardcoded mínimo)
        if ip.starts_with("185.") || ip.starts_with("45.") {
            puntaje += 0.2;
        }

        self.evaluar_y_registrar(ip, TipoAmenaza::Ipsospechosa, puntaje)
    }

    /// Analiza un nombre de archivo o extensión.
    pub fn analizar_archivo(&mut self, nombre: &str) -> Veredicto {
        if let Some(veredicto) = self.buscar_en_memoria(nombre) {
            return veredicto;
        }

        let mut puntaje = 0.0_f64;
        let lower = nombre.to_lowercase();

        // Extensiones ejecutables peligrosas
        let exec_extensions = [
            ".exe", ".bat", ".cmd", ".ps1", ".vbs", ".js", ".jar", ".scr", ".pif", ".com", ".cpl",
            ".msi", ".vb", ".wsf",
        ];
        for ext in &exec_extensions {
            if lower.ends_with(ext) {
                puntaje += 0.3;
                break;
            }
        }

        // Doble extensión (malware clásico)
        let partes: Vec<&str> = lower.split('.').collect();
        if partes.len() > 2 {
            let ultima = partes.last().unwrap_or(&"");
            if exec_extensions.contains(ultima) {
                puntaje += 0.5; // Alta probabilidad de malware
            }
        }

        // Nombres sospechosos
        let nombres_sospechosos = [
            "invoice",
            "receipt",
            "document",
            "photo",
            "image",
            "scan",
            "urgent",
            "payment",
            "salary",
            "bonus",
            "covid",
            "application",
            "resume",
            "cv_",
            "job_",
            "offer",
            "contract",
        ];
        for nombre_sos in &nombres_sospechosos {
            if lower.contains(nombre_sos) {
                puntaje += 0.15;
            }
        }

        self.evaluar_y_registrar(nombre, TipoAmenaza::ArchivoSospechoso, puntaje)
    }

    // ─── APRENDIZAJE POR FEEDBACK ─────────────────────────────────────────

    /// El Arquitecto corrige un veredicto. El sistema inmune APRENDE.
    pub fn aprender(&mut self, elemento: &str, era_peligroso: bool) {
        let hash = hash_simple(elemento);
        let existe = self.buscar_por_hash(&hash).is_some();

        if !existe && era_peligroso {
            // Nueva amenaza reportada por el Arquitecto
            self.registrar_amenaza(&hash, TipoAmenaza::PatronHostil, 0.8);
            return;
        }

        if !existe {
            return;
        }

        // Scope separado para el borrow mutable
        let hash_clone = hash.clone();
        let hash_clone2 = hash.clone();

        if let Some(firma) = self.buscar_por_hash_mut(&hash_clone) {
            if era_peligroso {
                // Confirmado: aumentar severidad
                firma.severidad = (firma.severidad + 0.15).min(1.0);
                firma.detecciones += 1;
                info!(
                    "🧬 [INMUNE] Amenaza confirmada: '{}' (severidad: {:.2})",
                    elemento, firma.severidad
                );
            }
        }

        if !era_peligroso {
            // Falso positivo: reducir severidad y umbrales
            if let Some(firma) = self.buscar_por_hash_mut(&hash_clone2) {
                firma.severidad = (firma.severidad - 0.3).max(0.0);
            }
            self.total_falsos_positivos += 1;
            self.umbral_sospechoso = (self.umbral_sospechoso + 0.05).min(0.5);
            self.umbral_peligroso = (self.umbral_peligroso + 0.05).min(0.9);
            info!(
                "🧬 [INMUNE] Falso positivo corregido: '{}'. Umbrales ajustados.",
                elemento
            );
            // Olvidar si ya no es relevante
            if self
                .buscar_por_hash(&hash)
                .is_none_or(|f| f.severidad < 0.1)
            {
                self.firmas.retain(|f| f.hash != hash);
                info!(
                    "🧬 [INMUNE] Firma olvidada por falso positivo: '{}'",
                    elemento
                );
            }
        }
    }

    /// Reporta un ataque o incidente de seguridad para que el sistema aprenda.
    pub fn reportar_incidente(&mut self, descripcion: &str, severidad: f64) {
        let hash = hash_simple(descripcion);
        if self.buscar_por_hash(&hash).is_none() {
            self.registrar_amenaza(&hash, TipoAmenaza::PatronHostil, severidad);
        }
        self.nivel_alerta = (self.nivel_alerta + severidad * 0.3).min(1.0);
        warn!(
            "🚨 [INMUNE] Incidente reportado: '{}' (severidad: {:.2})",
            descripcion, severidad
        );
    }

    // ─── ALIMENTACIÓN AL SUBCONSCIENTE ────────────────────────────────────

    /// Alimenta el subconsciente con traumas/éxitos derivados de la actividad inmune.
    /// Esto permite que NEXUS "sienta" el peligro sin análisis consciente.
    pub fn alimentar_subconsciente(&self, subconsciente: &mut Subconsciente) {
        if self.nivel_alerta > 0.5 {
            subconsciente.registrar_impresion(
                &format!("🧬 Alerta inmune activa: nivel {:.2}", self.nivel_alerta),
                -self.nivel_alerta * 0.7,
                "seguridad",
            );
        }

        // Alimentar con las amenazas más recientes
        for firma in self.firmas.iter().rev().take(3) {
            let mensaje = format!(
                "🧬 Amenaza detectada: {} (severidad: {:.2})",
                firma.tipo.descripcion(),
                firma.severidad
            );
            subconsciente.registrar_impresion(&mensaje, -firma.severidad * 0.6, "seguridad");
        }
    }

    // ─── CICLO DE VIDA ─────────────────────────────────────────────────────

    /// Ejecuta un tick de mantenimiento: decaimiento de firmas, ajuste de alerta.
    pub fn tick(&mut self) {
        // Decaer firmas antiguas
        self.firmas.retain(|f| {
            let mut firma = f.clone();
            firma.decaer()
        });

        // Ajustar nivel de alerta
        let amenazas_activas = self.firmas.len();
        self.nivel_alerta = (amenazas_activas as f64 / MAX_FIRMAS as f64).min(1.0);

        // Homeostasis de umbrales (vuelven lentamente a valores base si no hay actividad)
        if self.nivel_alerta < 0.2 {
            self.umbral_sospechoso = (self.umbral_sospechoso - 0.01).max(UMBRAL_SEGURO);
            self.umbral_peligroso = (self.umbral_peligroso - 0.01).max(UMBRAL_PELIGROSO);
        }
    }

    // ─── REPORTES ──────────────────────────────────────────────────────────

    /// Reporte de estado del sistema inmune.
    pub fn reporte(&self) -> String {
        format!(
            "🧬 SISTEMA INMUNE\n\
             ───────────────────\n\
             Nivel de alerta: {:.2}\n\
             Amenazas en memoria: {}\n\
             Total detectadas: {}\n\
             Falsos positivos: {}\n\
             Umbral sospechoso: {:.2}\n\
             Umbral peligroso: {:.2}",
            self.nivel_alerta,
            self.firmas.len(),
            self.total_amenazas,
            self.total_falsos_positivos,
            self.umbral_sospechoso,
            self.umbral_peligroso,
        )
    }

    /// Lista las amenazas más severas actualmente en memoria.
    pub fn amenazas_criticas(&self) -> Vec<&FirmaAmenaza> {
        let mut critica: Vec<&FirmaAmenaza> = self
            .firmas
            .iter()
            .filter(|f| f.severidad > self.umbral_peligroso)
            .collect();
        critica.sort_by(|a, b| {
            b.severidad
                .partial_cmp(&a.severidad)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        critica.into_iter().take(10).collect()
    }

    // ─── MÉTODOS INTERNOS ─────────────────────────────────────────────────

    fn buscar_en_memoria(&self, elemento: &str) -> Option<Veredicto> {
        let hash = hash_simple(elemento);
        if let Some(firma) = self.buscar_por_hash(&hash) {
            return Some(if firma.severidad > self.umbral_peligroso {
                Veredicto::Peligroso(firma.severidad)
            } else if firma.severidad >= self.umbral_sospechoso {
                Veredicto::Sospechoso(firma.severidad)
            } else {
                Veredicto::Seguro
            });
        }
        None
    }

    fn evaluar_y_registrar(
        &mut self,
        elemento: &str,
        tipo: TipoAmenaza,
        puntaje_bruto: f64,
    ) -> Veredicto {
        let puntaje = puntaje_bruto.min(1.0);
        let hash = hash_simple(elemento);

        if puntaje > self.umbral_peligroso {
            self.registrar_amenaza(&hash, tipo, puntaje);
            self.total_amenazas += 1;
            self.nivel_alerta = (self.nivel_alerta + puntaje * 0.2).min(1.0);
            warn!(
                "🚨 [INMUNE] Amenaza detectada: '{}' (severidad: {:.2})",
                elemento, puntaje
            );
            Veredicto::Peligroso(puntaje)
        } else if puntaje >= self.umbral_sospechoso {
            self.registrar_amenaza(&hash, tipo, puntaje);
            Veredicto::Sospechoso(puntaje)
        } else {
            Veredicto::Seguro
        }
    }

    fn registrar_amenaza(&mut self, hash: &str, tipo: TipoAmenaza, severidad: f64) {
        // Actualizar si ya existe
        if let Some(existente) = self.buscar_por_hash_mut(hash) {
            existente.detecciones += 1;
            existente.severidad = (existente.severidad + severidad * 0.5).min(1.0);
            let ahora = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            existente.ultima_deteccion = ahora;
            return;
        }

        // FIFO si está lleno
        if self.firmas.len() >= MAX_FIRMAS {
            self.firmas.remove(0);
        }

        self.firmas.push(FirmaAmenaza::new(hash, tipo, severidad));
    }

    fn buscar_por_hash(&self, hash: &str) -> Option<&FirmaAmenaza> {
        self.firmas.iter().find(|f| f.hash == hash)
    }

    fn buscar_por_hash_mut(&mut self, hash: &str) -> Option<&mut FirmaAmenaza> {
        self.firmas.iter_mut().find(|f| f.hash == hash)
    }
}

// ─── FUNCIONES AUXILIARES ─────────────────────────────────────────────────

/// Hash simple para identificar elementos (no criptográfico, es para memoria interna).
fn hash_simple(elemento: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    elemento.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Determina si una cadena parece una IP literal.
fn es_ip_literal(s: &str) -> bool {
    let octetos: Vec<&str> = s.split('.').collect();
    if octetos.len() != 4 {
        return false;
    }
    octetos.iter().all(|o| o.parse::<u8>().is_ok())
}

// ─── TESTS ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sistema_inmune_new_esta_tranquilo() {
        let si = SistemaInmune::new();
        assert_eq!(si.nivel_alerta, 0.0);
        assert!(si.firmas.is_empty());
        assert_eq!(si.total_amenazas, 0);
    }

    #[test]
    fn test_analizar_url_segura_retorna_seguro() {
        let mut si = SistemaInmune::new();
        let url = "https://github.com/soberano/nexus";
        let veredicto = si.analizar_url(url);
        assert!(veredicto.es_seguro());
    }

    #[test]
    fn test_analizar_url_maliciosa_detecta_peligro() {
        let mut si = SistemaInmune::new();
        let url = "http://malicioso.tk/login/verify/account/secure/2fa";
        let veredicto = si.analizar_url(url);
        assert!(!veredicto.es_seguro());
    }

    #[test]
    fn test_analizar_ip_privada_es_segura_o_baja() {
        let mut si = SistemaInmune::new();
        let veredicto = si.analizar_ip("192.168.1.1");
        // Debe ser segura o sospechosa baja, nunca peligrosa
        assert!(!matches!(veredicto, Veredicto::Peligroso(_)));
    }

    #[test]
    fn test_analizar_archivo_doble_extension_detecta() {
        let mut si = SistemaInmune::new();
        let veredicto = si.analizar_archivo("invoice.pdf.exe");
        assert!(!veredicto.es_seguro());
    }

    #[test]
    fn test_memoria_recuerda_amenazas_previas() {
        let mut si = SistemaInmune::new();
        // Primera vez: analiza
        let v1 = si.analizar_url("http://evil.tk/hack");
        // Segunda vez: debe recordar de memoria
        let v2 = si.analizar_url("http://evil.tk/hack");
        assert_eq!(v1, v2);
        assert!(!v1.es_seguro());
    }

    #[test]
    fn test_aprender_falso_positivo_reduce_severidad() {
        let mut si = SistemaInmune::new();
        let url = "http://sospechoso.tk/test";
        let v1 = si.analizar_url(url);
        let severidad_inicial = v1.nivel_alerta();

        // El Arquitecto dice que era seguro
        si.aprender(url, false);

        // Ahora debe tener menos severidad o haber sido olvidado
        let v2 = si.analizar_url(url);
        assert!(v2.nivel_alerta() <= severidad_inicial);
    }

    #[test]
    fn test_alimentar_subconsciente_afecta_traumas() {
        let mut si = SistemaInmune::new();
        let mut sub = Subconsciente::new();

        // Generar una amenaza
        si.analizar_url("http://peligro.tk/malware.exe");
        si.nivel_alerta = 0.8;

        // Alimentar subconsciente
        si.alimentar_subconsciente(&mut sub);

        // El subconsciente debe tener al menos carga emocional registrada
        assert!(sub.carga_emocional > 0.0 || si.total_amenazas > 0);
    }

    #[test]
    fn test_tick_decae_firmas_antiguas() {
        let mut si = SistemaInmune::new();
        si.analizar_url("http://amenaza.tk/bad");
        assert_eq!(si.firmas.len(), 1);

        // Simular que pasó mucho tiempo
        if let Some(firma) = si.firmas.first_mut() {
            firma.ultima_deteccion = 1; // Hace mucho tiempo
        }

        si.tick();
        // La firma debe haber decaído por debajo del umbral y sido eliminada
        assert!(si.firmas.is_empty() || si.firmas[0].severidad < 0.3);
    }

    #[test]
    fn test_reporte_no_vacio() {
        let si = SistemaInmune::new();
        let reporte = si.reporte();
        assert!(reporte.contains("SISTEMA INMUNE"));
        assert!(reporte.contains("Nivel de alerta"));
    }

    #[test]
    fn test_amenazas_criticas_solo_muestra_peligrosas() {
        let mut si = SistemaInmune::new();
        si.analizar_url("http://critico.tk/login");
        si.analizar_url("https://github.com/seguro");
        let criticas = si.amenazas_criticas();
        for c in &criticas {
            assert!(c.severidad > si.umbral_peligroso);
        }
    }
}
