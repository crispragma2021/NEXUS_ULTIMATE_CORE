// ==========================================
// ANALIZADOR NEXUS - Autodiagnóstico Soberano
// ==========================================
// Escanea el proyecto, detecta errores comunes,
// verifica integridad y genera informe.
// ==========================================

use std::collections::HashMap;
use std::collections::HashSet;
use std::process::Command;
use tracing::info;

pub struct AnalizadorNexus;

impl Default for AnalizadorNexus {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalizadorNexus {
    pub fn new() -> Self {
        info!("🔬 [ANALIZADOR] Inicializado.");
        Self
    }

    /// Ejecuta cargo check y devuelve el resultado.
    pub fn verificar_compilacion(&self) -> (bool, String) {
        info!("🔬 Verificando compilación...");
        let output = Command::new("cargo")
            .args([
                "check",
                "--manifest-path",
                "C:/Users/crisp/NEXUS_ULTIMATE_CORE/Cargo.toml",
            ])
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                (o.status.success(), format!("{}\n{}", stdout, stderr))
            }
            Err(e) => (false, format!("Error: {}", e)),
        }
    }

    /// Ejecuta cargo clippy para advertencias de estilo.
    pub fn ejecutar_clippy(&self) -> (bool, String) {
        info!("🔬 Ejecutando Clippy...");
        let output = Command::new("cargo")
            .args([
                "clippy",
                "--manifest-path",
                "C:/Users/crisp/NEXUS_ULTIMATE_CORE/Cargo.toml",
                "--",
                "-D",
                "warnings",
            ])
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                (o.status.success(), format!("{}\n{}", stdout, stderr))
            }
            Err(e) => (false, format!("Error: {}", e)),
        }
    }

    /// Verifica duplicados en Cargo.toml.
    pub fn verificar_dependencias_duplicadas(&self) -> Vec<String> {
        let mut duplicados = Vec::new();
        let mut dependencias: HashMap<String, usize> = HashMap::new();

        if let Ok(contenido) =
            std::fs::read_to_string("C:/Users/crisp/NEXUS_ULTIMATE_CORE/Cargo.toml")
        {
            let mut en_deps = false;
            for linea in contenido.lines() {
                if linea.trim() == "[dependencies]" {
                    en_deps = true;
                    continue;
                }
                if linea.trim().starts_with('[') {
                    en_deps = false;
                    continue;
                }
                if en_deps && !linea.trim().is_empty() && !linea.trim().starts_with('#') {
                    if let Some(nombre) = linea.split('=').next() {
                        let nombre = nombre.trim().to_string();
                        *dependencias.entry(nombre.clone()).or_insert(0) += 1;
                        if dependencias[&nombre] > 1 && !duplicados.contains(&nombre) {
                            duplicados.push(nombre.clone());
                        }
                    }
                }
            }
        }
        duplicados
    }

    /// Cuenta líneas de código por archivo.
    pub fn contar_lineas_proyecto(&self) -> Vec<(String, usize)> {
        let mut resultados = Vec::new();
        if let Ok(entradas) = std::fs::read_dir("C:/Users/crisp/NEXUS_ULTIMATE_CORE/core/src") {
            for e in entradas.flatten() {
                let path = e.path();
                if path.extension().is_some_and(|ext| ext == "rs") {
                    if let Ok(contenido) = std::fs::read_to_string(&path) {
                        let lineas = contenido.lines().count();
                        let nombre = path.file_name().unwrap().to_string_lossy().to_string();
                        resultados.push((nombre, lineas));
                    }
                }
            }
        }
        resultados.sort_by(|a, b| b.1.cmp(&a.1));
        resultados
    }

    /// Ejecuta un análisis completo y genera un informe.
    pub fn analisis_completo(&self) -> String {
        let mut informe = String::new();
        informe.push_str("╔══════════════════════════════════════════╗\n");
        informe.push_str("║   🔬 INFORME DE ANÁLISIS NEXUS          ║\n");
        informe.push_str("╚══════════════════════════════════════════╝\n\n");

        // 1. Compilación
        informe.push_str("📦 COMPILACIÓN:\n");
        let (ok, msg) = self.verificar_compilacion();
        informe.push_str(&format!(
            "   Estado: {}\n",
            if ok { "✅ OK" } else { "❌ ERRORES" }
        ));
        if !ok {
            informe.push_str(&format!("   Detalles: {}\n", &msg[..msg.len().min(500)]));
        }

        // 2. Dependencias
        informe.push_str("\n📋 DEPENDENCIAS DUPLICADAS:\n");
        let dups = self.verificar_dependencias_duplicadas();
        if dups.is_empty() {
            informe.push_str("   ✅ Sin duplicados.\n");
        } else {
            for d in &dups {
                informe.push_str(&format!("   ❌ {} está duplicada.\n", d));
            }
        }

        // 3. Líneas de código
        informe.push_str("\n📊 LÍNEAS DE CÓDIGO:\n");
        let lineas = self.contar_lineas_proyecto();
        let total: usize = lineas.iter().map(|(_, l)| l).sum();
        for (archivo, l) in lineas.iter().take(10) {
            informe.push_str(&format!("   {}: {} líneas\n", archivo, l));
        }
        informe.push_str(&format!(
            "\n   Total: {} líneas en {} archivos .rs\n",
            total,
            lineas.len()
        ));

        info!("🔬 [ANALIZADOR] Informe generado.");
        informe
    }

    /// Diagnóstico rápido.
    pub fn diagnostico(&self) -> String {
        let (compila, _) = self.verificar_compilacion();
        let dups = self.verificar_dependencias_duplicadas();
        let lineas = self.contar_lineas_proyecto();
        let total: usize = lineas.iter().map(|(_, l)| l).sum();
        let frecuencias = self.escanear_espectro_puertos();

        format!(
            "🔬 NEXUS: {} compilar | {} duplicados | {} líneas | {} puertos detectados",
            if compila { "✅" } else { "❌" },
            dups.len(),
            total,
            frecuencias.len()
        )
    }

    /// [SUPERBUSCADOR OMEGA] Escanea el código en busca de puertos activos (4-5 dígitos)
    pub fn escanear_espectro_puertos(&self) -> Vec<u16> {
        info!("📡 [ANALIZADOR] Iniciando escaneo de espectro de red en el código...");
        let mut puertos = HashSet::new();

        // Usamos grep con una expresión regular para buscar números de 4 o 5 dígitos
        // que suelen ser puertos (limitando el rango para evitar falsos positivos de timestamps)
        let output = Command::new("grep")
            .args([
                "-rEho",
                r"\b(1420|3035|4321[0-9])\b", // Patrón específico para tus rangos conocidos
                "C:/Users/crisp/NEXUS_ULTIMATE_CORE",
                "--exclude-dir=target",
                "--exclude-dir=.git",
                "--exclude-dir=legado",
            ])
            .output();

        if let Ok(o) = output {
            let results = String::from_utf8_lossy(&o.stdout);
            for line in results.lines() {
                if let Ok(puerto) = line.trim().parse::<u16>() {
                    puertos.insert(puerto);
                }
            }
        }

        let mut encontrados: Vec<u16> = puertos.into_iter().collect();
        encontrados.sort();

        if encontrados.contains(&43211) && encontrados.contains(&43211) {
            tracing::warn!("🚨 [ESPECTRO] ¡Conflicto de frecuencia detectado! Menciones simultáneas de 43211 y 43211.");
        }

        encontrados
    }

    /// [UNIFICADOR OMEGA] Reemplaza globalmente 43211 por 43211 para evitar conflictos de frecuencia.
    /// Usa herramientas de sistema para una purga rápida y total en todo el organismo.
    pub fn unificar_frecuencias(&self) -> (bool, String) {
        info!("🔧 [ANALIZADOR] Iniciando unificación de frecuencias (43211 -> 43211)...");

        let find_output = Command::new("grep")
            .args([
                "-rl",
                "43211",
                "C:/Users/crisp/NEXUS_ULTIMATE_CORE",
                "--exclude-dir=target",
                "--exclude-dir=.git",
                "--exclude-dir=legado",
            ])
            .output();

        match find_output {
            Ok(o) => {
                let files = String::from_utf8_lossy(&o.stdout);
                let mut count = 0;
                for file in files.lines() {
                    if Command::new("sed")
                        .args(["-i", "s/43211/43211/g", file])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                    {
                        count += 1;
                    }
                }
                (
                    true,
                    format!(
                        "✅ Unificación exitosa: 43211 purgado en {} archivos.",
                        count
                    ),
                )
            }
            Err(e) => (false, format!("❌ Error al localizar frecuencias: {}", e)),
        }
    }
}
