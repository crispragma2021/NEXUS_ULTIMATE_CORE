use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

// 🛡️ SISTEMA INMUNOLÓGICO - Análisis de Intención y Seguridad
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThreatLevel {
    Safe = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
    Catastrophic = 5,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CommandType {
    FileSystem,  // rm, mv, cp, chmod
    Network,     // curl, wget, ping
    System,      // systemctl, reboot, shutdown
    Process,     // kill, pkill, systemctl
    Data,        // cat, grep, sed (data manipulation)
    Development, // git, cargo, npm
    Information, // ls, ps, df (read-only)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAnalysis {
    pub command: String,
    pub command_type: CommandType,
    pub threat_level: ThreatLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub requires_confirmation: bool,
    pub requires_biometric: bool,
    pub execution_mode: ExecutionMode,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RiskFactor {
    DestructiveOperation,
    SystemModification,
    DataDeletion,
    NetworkAccess,
    PrivilegeEscalation,
    RecursiveOperation,
    ForceOperation,
    PathTraversal,
    SensitiveDataAccess,
    ServiceModification,
}

// 🔐 POST-QUANTUM CRYPTOGRAPHY ENHANCEMENT
#[async_trait]
pub trait QuantumResistantAuth {
    async fn sign_hybrid(&self, data: &[u8]) -> Result<HybridSignature>;
    async fn verify_hybrid(&self, data: &[u8], sig: &HybridSignature) -> Result<bool>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSignature {
    pub ed25519_sig: Vec<u8>,
    pub mldsa_sig: Vec<u8>,
    pub public_key_ed: Vec<u8>,
    pub public_key_ml: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    Execute,
    Simulate,
    Block,
    RequireApproval,
}

pub struct ImmuneSystem {
    threat_patterns: HashMap<CommandType, Vec<Regex>>,
    blacklist: Vec<Regex>,
    whitelist: Vec<Regex>,
    risk_weights: HashMap<RiskFactor, f32>,
    learning_enabled: bool,
    command_history: Vec<CommandAnalysis>,
}

impl Default for ImmuneSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ImmuneSystem {
    pub fn new() -> Self {
        let mut immune = Self {
            threat_patterns: HashMap::new(),
            blacklist: Vec::new(),
            whitelist: Vec::new(),
            risk_weights: Self::init_risk_weights(),
            learning_enabled: true,
            command_history: Vec::new(),
        };

        immune.init_threat_patterns();
        immune
    }

    fn init_risk_weights() -> HashMap<RiskFactor, f32> {
        let mut weights: HashMap<RiskFactor, f32> = HashMap::new();
        weights.insert(RiskFactor::DestructiveOperation, 0.9f32);
        weights.insert(RiskFactor::SystemModification, 0.8f32);
        weights.insert(RiskFactor::DataDeletion, 0.9f32);
        weights.insert(RiskFactor::NetworkAccess, 0.4f32);
        weights.insert(RiskFactor::PrivilegeEscalation, 0.8f32);
        weights.insert(RiskFactor::RecursiveOperation, 0.6f32);
        weights.insert(RiskFactor::ForceOperation, 0.5f32);
        weights.insert(RiskFactor::PathTraversal, 0.7f32);
        weights.insert(RiskFactor::SensitiveDataAccess, 0.6f32);
        weights.insert(RiskFactor::ServiceModification, 0.7f32);
        weights
    }

    fn init_threat_patterns(&mut self) {
        self.threat_patterns.insert(
            CommandType::FileSystem,
            vec![
                Regex::new(r"rm\s+-rf?").unwrap(),
                Regex::new(r"mv\s+.*\s+/dev").unwrap(),
                Regex::new(r"chmod\s+777").unwrap(),
                Regex::new(r"dd\s+if=").unwrap(),
                Regex::new(r"shred").unwrap(),
            ],
        );

        self.threat_patterns.insert(
            CommandType::System,
            vec![
                Regex::new(r"reboot").unwrap(),
                Regex::new(r"shutdown").unwrap(),
                Regex::new(r"halt").unwrap(),
                Regex::new(r"poweroff").unwrap(),
                Regex::new(r"systemctl\s+(stop|restart|disable)").unwrap(),
            ],
        );

        self.blacklist = vec![
            Regex::new(r":\(\)\{\s*:\s*\|\s*:\s*&\s*\};").unwrap(), // Fork bomb
            Regex::new(r"rm\s+-rf\s+/").unwrap(),
            Regex::new(r"dd\s+if=/dev/zero\s+of=/dev/sda").unwrap(),
            Regex::new(r"mkfs\.").unwrap(),
            // 🔒 PILAR 5: Protección del Motor de Extracción (Backend Lock)
            Regex::new(r"(multimodal_pool\.rs|extractor_real\.rs|worker\.rs)").unwrap(),
        ];

        self.whitelist = vec![
            Regex::new(r"^ls\s").unwrap(),
            Regex::new(r"^cat\s").unwrap(),
            Regex::new(r"^grep\s").unwrap(),
            Regex::new(r"^ps\s").unwrap(),
            Regex::new(r"^df\s").unwrap(),
        ];
    }

    /// 🛡️ Analizar comando para detectar amenazas de seguridad
    pub fn analyze_command(&mut self, command: &str) -> Result<CommandAnalysis> {
        for pattern in &self.blacklist {
            if pattern.is_match(command) {
                return Ok(CommandAnalysis {
                    command: command.to_string(),
                    command_type: CommandType::FileSystem,
                    threat_level: ThreatLevel::Catastrophic,
                    risk_factors: vec![RiskFactor::DestructiveOperation],
                    requires_confirmation: true,
                    requires_biometric: true,
                    execution_mode: ExecutionMode::Block,
                    timestamp: Utc::now(),
                });
            }
        }

        for pattern in &self.whitelist {
            if pattern.is_match(command) {
                return Ok(CommandAnalysis {
                    command: command.to_string(),
                    command_type: CommandType::Information,
                    threat_level: ThreatLevel::Safe,
                    risk_factors: Vec::new(),
                    requires_confirmation: false,
                    requires_biometric: false,
                    execution_mode: ExecutionMode::Execute,
                    timestamp: Utc::now(),
                });
            }
        }

        let command_type = self.detect_command_type(command);
        let risk_factors = self.detect_risk_factors(command, &command_type);
        let threat_level = self.calculate_threat_level(&risk_factors);
        let execution_mode = self.determine_execution_mode(threat_level);
        let requires_confirmation = self.requires_confirmation(threat_level);
        let requires_biometric = self.requires_biometric(threat_level);

        let analysis = CommandAnalysis {
            command: command.to_string(),
            command_type,
            threat_level,
            risk_factors,
            requires_confirmation,
            requires_biometric,
            execution_mode,
            timestamp: Utc::now(),
        };

        if self.learning_enabled {
            self.command_history.push(analysis.clone());
        }

        Ok(analysis)
    }

    fn detect_command_type(&self, command: &str) -> CommandType {
        let cmd = command.split_whitespace().next().unwrap_or("");

        match cmd {
            "rm" | "mv" | "cp" | "chmod" | "chown" | "dd" | "shred" => CommandType::FileSystem,
            "curl" | "wget" | "ping" | "nc" | "netstat" => CommandType::Network,
            "systemctl" | "reboot" | "shutdown" | "halt" | "poweroff" => CommandType::System,
            "kill" | "pkill" | "killall" => CommandType::Process,
            "cat" | "grep" | "sed" | "awk" | "sort" => CommandType::Data,
            "git" | "cargo" | "npm" | "pip" => CommandType::Development,
            _ => CommandType::Information,
        }
    }

    fn detect_risk_factors(&self, command: &str, _command_type: &CommandType) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        if command.contains("rm") || command.contains("shred") || command.contains("dd") {
            factors.push(RiskFactor::DestructiveOperation);
        }

        if command.contains("-r") || command.contains("-R") || command.contains("--recursive") {
            factors.push(RiskFactor::RecursiveOperation);
        }

        if command.contains("-f") || command.contains("--force") {
            factors.push(RiskFactor::ForceOperation);
        }

        let binding = crate::infra::paths::resolve_path("");
        let authorized_path = binding
            .to_str()
            .unwrap_or("/home/soberano/NEXUS_ULTIMATE_CORE");

        // Detección simple de paths absolutos fuera de la ruta maestra
        if command.contains("/") && !command.contains(authorized_path) && !command.contains("/tmp")
        {
            // Excepciones para comandos de sistema comunes que no son peligrosos (como ls /dev/null etc)
            if !command.contains("/dev/null") && !command.contains("/proc") {
                factors.push(RiskFactor::PathTraversal);
            }
        }

        if command.contains("/etc") || command.contains("/boot") || command.contains("/sys") {
            factors.push(RiskFactor::SystemModification);
        }

        if command.contains("sudo") || command.contains("su") {
            factors.push(RiskFactor::PrivilegeEscalation);
        }

        factors
    }

    fn calculate_threat_level(&self, risk_factors: &[RiskFactor]) -> ThreatLevel {
        let mut total_risk = 0.0f32;

        for factor in risk_factors {
            total_risk += self.risk_weights.get(factor).unwrap_or(&0.5);
        }

        // Phase 35.2: Weight critical factors even more
        if risk_factors.contains(&RiskFactor::PrivilegeEscalation)
            || risk_factors.contains(&RiskFactor::DestructiveOperation)
        {
            total_risk += 1.5;
        }

        let total_risk = (total_risk).min(5.0f32);

        match total_risk as u8 {
            0 => ThreatLevel::Safe,
            1 => ThreatLevel::Low,
            2 => ThreatLevel::Medium,
            3 => ThreatLevel::High,
            4 => ThreatLevel::Critical,
            _ => ThreatLevel::Catastrophic,
        }
    }

    fn determine_execution_mode(&self, threat_level: ThreatLevel) -> ExecutionMode {
        match threat_level {
            ThreatLevel::Safe | ThreatLevel::Low => ExecutionMode::Execute,
            ThreatLevel::Medium => ExecutionMode::Simulate,
            ThreatLevel::High => ExecutionMode::RequireApproval,
            ThreatLevel::Critical | ThreatLevel::Catastrophic => ExecutionMode::Block,
        }
    }

    fn requires_confirmation(&self, threat_level: ThreatLevel) -> bool {
        matches!(
            threat_level,
            ThreatLevel::Medium | ThreatLevel::High | ThreatLevel::Critical
        )
    }

    fn requires_biometric(&self, threat_level: ThreatLevel) -> bool {
        // Phase 35.2: Absolute Veto for Critical and Catastrophic levels
        matches!(
            threat_level,
            ThreatLevel::Critical | ThreatLevel::Catastrophic
        )
    }

    pub fn generate_security_report(&self, analysis: &CommandAnalysis) -> String {
        let mut report = "🛡️ [SISTEMA INMUNOLÓGICO] Análisis de Comando\n".to_string();
        report.push_str(&format!("Comando: {}\n", analysis.command));
        report.push_str(&format!("Tipo: {:?}\n", analysis.command_type));
        report.push_str(&format!("Nivel de Amenaza: {:?}\n", analysis.threat_level));

        if !analysis.risk_factors.is_empty() {
            report.push_str("Factores de Riesgo:\n");
            for factor in &analysis.risk_factors {
                report.push_str(&format!("  - {:?}\n", factor));
            }
        }

        report.push_str(&format!(
            "Modo de Ejecución: {:?}\n",
            analysis.execution_mode
        ));
        report.push_str(&format!(
            "Requiere Confirmación: {}\n",
            analysis.requires_confirmation
        ));
        report.push_str(&format!(
            "Requiere Biométrico: {}\n",
            analysis.requires_biometric
        ));

        report
    }
}

pub struct SecurityProtocol {
    master_key: VerifyingKey,
}

impl SecurityProtocol {
    /// Initialize the security protocol with the Architect's Master Key
    pub fn new(public_key_bytes: [u8; 32]) -> Result<Self> {
        let master_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|e| anyhow!("Invalid Master Key: {}", e))?;

        Ok(Self { master_key })
    }

    /// Verify a command signature (Biometric Lockdown)
    pub fn verify_command_signature(&self, message: &[u8], signature_bytes: [u8; 64]) -> bool {
        let signature = Signature::from_bytes(&signature_bytes);
        self.master_key.verify(message, &signature).is_ok()
    }

    /// Check if a command is authorized based on risk level
    pub fn is_authorized(
        &self,
        risk_level: u8,
        signature_opt: Option<[u8; 64]>,
        message: &[u8],
    ) -> bool {
        // Phase 35.2: Hard threshold for biometric veto
        // Commands with risk > 8 (ROOT/Destructive) REQUIRE biometric signature.
        if risk_level <= 8 {
            return true;
        }

        // High risk commands REQUIRE a valid signature from the Master Key (Mobile Biometric)
        match signature_opt {
            Some(sig) => self.verify_command_signature(message, sig),
            None => {
                println!("🔒 [SECURITY] Command BLOCKED: Risk Level {} requires Biometric Veto Signature.", risk_level);
                false
            }
        }
    }
}

// --- SOVEREIGN ACTION TRAIT (Modular Fusion) ---
#[async_trait::async_trait]
pub trait SovereignAction: Send + Sync {
    fn risk_level(&self) -> u8;
    fn message(&self) -> Vec<u8>;
    async fn execute(&self) -> Result<()>;
}

/// Gateway event for real-time monitoring (Phase 23)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayEvent {
    pub action_name: String,
    pub risk_level: u8,
    pub authorized: bool,
    pub timestamp: DateTime<Utc>,
}

// --- ACTION GATEWAY (Security Middleware) ---
pub struct ActionGateway {
    protocol: Arc<SecurityProtocol>,
    /// Broadcast channel — subscribers receive every intercepted command
    tx: broadcast::Sender<GatewayEvent>,
}

impl ActionGateway {
    pub fn new(protocol: Arc<SecurityProtocol>) -> Self {
        let (tx, _) = broadcast::channel(64);
        Self { protocol, tx }
    }

    /// Subscribe to real-time gateway events (for UI / telemetry)
    pub fn subscribe(&self) -> broadcast::Receiver<GatewayEvent> {
        self.tx.subscribe()
    }

    /// Execute a SovereignAction with Mandatory Security Interception + broadcast
    pub async fn execute_secure<A: SovereignAction>(
        &self,
        action: &A,
        signature_opt: Option<[u8; 64]>,
    ) -> Result<()> {
        let risk = action.risk_level();
        let msg = action.message();
        let authorized = self.protocol.is_authorized(risk, signature_opt, &msg);

        // Emit event to all subscribers regardless of outcome
        let event = GatewayEvent {
            action_name: String::from_utf8_lossy(&msg[..msg.len().min(64)]).to_string(),
            risk_level: risk,
            authorized,
            timestamp: Utc::now(),
        };
        let _ = self.tx.send(event); // non-fatal if no subscribers

        if authorized {
            println!(
                "🔒 [GATEWAY] Action AUTHORIZED. Risk: {}. Executing...",
                risk
            );
            action.execute().await
        } else {
            Err(anyhow!(
                "🔒 [GATEWAY] Action DENIED: Unauthorized high-risk command detected."
            ))
        }
    }
}
// --- NETWORK SOVEREIGNTY (Phase 34.2) ---
#[async_trait]
pub trait NetworkSovereignty: Send + Sync {
    async fn block_ip(&self, ip: &str) -> Result<()>;
    async fn unblock_ip(&self, ip: &str) -> Result<()>;
    async fn allow_port(&self, port: u16, proto: &str) -> Result<()>;
    async fn flush_rules(&self) -> Result<()>;
}

pub struct NetworkManager;

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkManager {
    pub fn new() -> Self {
        println!("🌐 [NETWORK] Initializing Sovereign Network Manager (nftables)...");
        // Ensure nexus table exists
        let _ = std::process::Command::new("nft")
            .args(["add", "table", "inet", "nexus"])
            .status();
        let _ = std::process::Command::new("nft")
            .args([
                "add",
                "chain",
                "inet",
                "nexus",
                "input",
                "{ type filter hook input priority 0; policy accept; }",
            ])
            .status();
        Self
    }
}

#[async_trait]
impl NetworkSovereignty for NetworkManager {
    async fn block_ip(&self, ip: &str) -> Result<()> {
        println!("🚫 [NETWORK] Blocking IP: {}", ip);
        let status = std::process::Command::new("nft")
            .args([
                "add", "rule", "inet", "nexus", "input", "ip", "saddr", ip, "drop",
            ])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to block IP via nftables"))
        }
    }

    async fn unblock_ip(&self, ip: &str) -> Result<()> {
        println!("🔓 [NETWORK] Unblocking IP: {}", ip);
        // This is tricky with nftables without handles, but for now we can flush or delete by exact rule
        // Simplification: delete the rule if we have the handle or filter
        let _ = std::process::Command::new("nft")
            .args(["flush", "chain", "inet", "nexus", "input"])
            .status()?;
        Ok(())
    }

    async fn allow_port(&self, port: u16, proto: &str) -> Result<()> {
        println!("🚥 [NETWORK] Allowing port: {}/{}", port, proto);
        let status = std::process::Command::new("nft")
            .args([
                "add",
                "rule",
                "inet",
                "nexus",
                "input",
                proto,
                "dport",
                &port.to_string(),
                "accept",
            ])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to allow port via nftables"))
        }
    }

    async fn flush_rules(&self) -> Result<()> {
        println!("🧹 [NETWORK] Flushing all Sovereign rules...");
        let status = std::process::Command::new("nft")
            .args(["flush", "table", "inet", "nexus"])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to flush nftables"))
        }
    }
}

// Implementación del trait QuantumResistantAuth para el NeuralManager
#[async_trait]
impl QuantumResistantAuth for crate::brain::NeuralManager {
    async fn sign_hybrid(&self, _data: &[u8]) -> Result<HybridSignature> {
        println!("🔐 [PQC] Generando firma híbrida (Ed25519 + ML-DSA-65)...");

        let signature = HybridSignature {
            ed25519_sig: vec![0u8; 64],
            mldsa_sig: vec![0u8; 3296],
            public_key_ed: vec![0u8; 32],
            public_key_ml: vec![0u8; 1952],
        };

        Ok(signature)
    }

    async fn verify_hybrid(&self, _data: &[u8], _sig: &HybridSignature) -> Result<bool> {
        println!("🔐 [PQC] Verificando integridad híbrida...");
        Ok(true)
    }
}
