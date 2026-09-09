// ============================================================================
// 🔀 DIFF ENGINE — Cálculo de unified diffs entre versiones de código
// ============================================================================
// Genera diffs unificados (formato estilo `diff -u`) entre una versión
// original y una corregida, para registrar los cambios aplicados por los
// debuggers en el Session Store (DiffEntry).
//
// Estrategia:
//   - `DiffEngine` (determinista, sin red): algoritmo LCS sobre líneas para
//     encontrar las secuencias común y cambiada, y emitir el diff unificado.
//   - Límite de líneas por seguridad (el LCS es O(n*m)); por encima del límite
//     se degrada a "archivo reemplazado".
// ============================================================================

use std::time::Instant;

/// Límite de líneas para el LCS (protección O(n*m)).
const MAX_LINEAS_LCS: usize = 4_000;

/// Resultado del cálculo de diff entre dos versiones de un archivo.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoDiff {
    /// Diff unificado en formato texto (`@@` hunks, líneas `+`/`-`/espacio).
    pub diff_unificado: String,
    /// Líneas añadidas netas.
    pub lineas_agregadas: usize,
    /// Líneas eliminadas netas.
    pub lineas_eliminadas: usize,
    /// `true` si hubo cambios; `false` si ambas versiones son idénticas.
    pub hay_cambios: bool,
    /// Duración del cálculo en milisegundos.
    pub duration_ms: u64,
}

impl ResultadoDiff {
    /// Un diff vacío (sin cambios).
    fn vacio() -> Self {
        ResultadoDiff {
            diff_unificado: String::new(),
            lineas_agregadas: 0,
            lineas_eliminadas: 0,
            hay_cambios: false,
            duration_ms: 0,
        }
    }
}

/// Motor de cálculo de diffs unificados.
#[derive(Debug, Clone, Default)]
pub struct DiffEngine;

impl DiffEngine {
    /// Calcula el diff unificado entre `original` y `corregido`.
    pub fn calcular_diff(&self, ruta: &str, original: &str, corregido: &str) -> ResultadoDiff {
        let inicio = Instant::now();

        if original == corregido {
            let mut r = ResultadoDiff::vacio();
            r.duration_ms = inicio.elapsed().as_millis() as u64;
            return r;
        }

        let orig: Vec<&str> = original.lines().collect();
        let corr: Vec<&str> = corregido.lines().collect();

        // Degradación segura para archivos muy grandes.
        if orig.len() > MAX_LINEAS_LCS || corr.len() > MAX_LINEAS_LCS {
            return ResultadoDiff {
                diff_unificado: format!(
                    "--- a/{ruta}\n+++ b/{ruta}\n@@ -1,{} +1,{} @@\n-<{} líneas originales>\n+<{} líneas corregidas>\n",
                    orig.len(),
                    corr.len(),
                    orig.len(),
                    corr.len()
                ),
                lineas_agregadas: corr.len(),
                lineas_eliminadas: orig.len(),
                hay_cambios: true,
                duration_ms: inicio.elapsed().as_millis() as u64,
            };
        }

        // Matriz LCS sobre líneas.
        let (n, m) = (orig.len(), corr.len());
        let mut dp = vec![vec![0usize; m + 1]; n + 1];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                dp[i][j] = if orig[i] == corr[j] {
                    dp[i + 1][j + 1] + 1
                } else {
                    dp[i + 1][j].max(dp[i][j + 1])
                };
            }
        }

        // Construcción de la secuencia de operaciones (recorrido de la matriz).
        let mut ops: Vec<(char, &str)> = Vec::with_capacity(n + m);
        let (mut i, mut j) = (0usize, 0usize);
        while i < n && j < m {
            if orig[i] == corr[j] {
                ops.push((' ', orig[i]));
                i += 1;
                j += 1;
            } else if dp[i + 1][j] >= dp[i][j + 1] {
                ops.push(('-', orig[i]));
                i += 1;
            } else {
                ops.push(('+', corr[j]));
                j += 1;
            }
        }
        while i < n {
            ops.push(('-', orig[i]));
            i += 1;
        }
        while j < m {
            ops.push(('+', corr[j]));
            j += 1;
        }

        // Emitir diff unificado con hunks agrupados por contexto.
        let diff = self.empaquetar_unificado(ruta, &ops, &orig, &corr);
        let agregadas = ops.iter().filter(|(c, _)| *c == '+').count();
        let eliminadas = ops.iter().filter(|(c, _)| *c == '-').count();

        ResultadoDiff {
            diff_unificado: diff,
            lineas_agregadas: agregadas,
            lineas_eliminadas: eliminadas,
            hay_cambios: true,
            duration_ms: inicio.elapsed().as_millis() as u64,
        }
    }

    /// Empaqueta las operaciones en hunks unificados con 3 líneas de contexto.
    fn empaquetar_unificado(
        &self,
        ruta: &str,
        ops: &[(char, &str)],
        _orig: &[&str],
        _corr: &[&str],
    ) -> String {
        const CONTEXTO: usize = 3;
        let mut salida = String::new();
        salida.push_str(&format!("--- a/{ruta}\n"));
        salida.push_str(&format!("+++ b/{ruta}\n"));

        let mut idx = 0usize;
        while idx < ops.len() {
            // Buscar el inicio del siguiente cambio.
            while idx < ops.len() && ops[idx].0 == ' ' {
                idx += 1;
            }
            if idx >= ops.len() {
                break;
            }
            // Retroceder hasta CONTEXTO líneas antes (o el inicio).
            let mut hunk_inicio = idx;
            let mut cont = 0usize;
            while hunk_inicio > 0 && cont < CONTEXTO && ops[hunk_inicio - 1].0 == ' ' {
                hunk_inicio -= 1;
                cont += 1;
            }
            // Avanzar hasta CONTEXTO líneas después del último cambio.
            let mut hunk_fin = idx;
            while hunk_fin < ops.len() {
                if ops[hunk_fin].0 != ' ' {
                    hunk_fin += 1;
                    cont = 0;
                } else {
                    if cont >= CONTEXTO {
                        break;
                    }
                    cont += 1;
                    hunk_fin += 1;
                }
            }
            // Número de líneas de contexto de este hunk (líneas ' ' al inicio
            // y al final; las interiores entre cambios no cuentan como contexto
            // adicional para los rangos).
            let mut n_contexto_ini = 0usize;
            let mut k = hunk_inicio;
            while k < hunk_fin && ops[k].0 == ' ' {
                n_contexto_ini += 1;
                k += 1;
            }
            let mut n_contexto_fin = 0usize;
            let mut k = hunk_fin;
            while k > hunk_inicio && ops[k - 1].0 == ' ' {
                n_contexto_fin += 1;
                k -= 1;
            }
            let n_contexto = n_contexto_ini + n_contexto_fin;

            // Calcular contadores de cambio dentro del hunk.
            let mut n_orig_hunk = n_contexto;
            let mut n_corr_hunk = n_contexto;
            for op in &ops[hunk_inicio..hunk_fin] {
                match op.0 {
                    '-' => n_orig_hunk += 1,
                    '+' => n_corr_hunk += 1,
                    _ => {}
                }
            }

            // Offset (1-based) del inicio del hunk en cada lado: nº de líneas
            // no-'+' (original) / no-'-' (corregido) antes de hunk_inicio.
            let mut inicio_orig = 1usize;
            let mut inicio_corr = 1usize;
            for op in &ops[..hunk_inicio] {
                if op.0 != '+' {
                    inicio_orig += 1;
                }
                if op.0 != '-' {
                    inicio_corr += 1;
                }
            }

            let rango_orig = if n_orig_hunk == 1 {
                format!("{inicio_orig}")
            } else {
                format!("{inicio_orig},{n_orig_hunk}")
            };
            let rango_corr = if n_corr_hunk == 1 {
                format!("{inicio_corr}")
            } else {
                format!("{inicio_corr},{n_corr_hunk}")
            };
            salida.push_str(&format!("@@ -{rango_orig} +{rango_corr} @@\n"));
            for op in &ops[hunk_inicio..hunk_fin] {
                salida.push_str(&format!("{}{}\n", op.0, op.1));
            }
            idx = hunk_fin;
        }

        salida
    }

    /// Aplica un diff unificado a un contenido y devuelve el resultado.
    /// Simple: si el diff es `---`/`+++` con `@@`, reconstruye aplicando hunks.
    /// Devuelve `None` si el diff no es aplicable (p.ej. contexto no coincide).
    pub fn aplicar_diff(&self, original: &str, diff_unificado: &str) -> Option<String> {
        // Enfoque determinista: descomponer el diff y reconstruir línea a línea.
        let mut resultado: Vec<&str> = Vec::new();
        let mut orig_lines: Vec<&str> = original.lines().collect();
        let mut pos_orig = 0usize;

        let mut en_hunk = false;
        for linea in diff_unificado.lines() {
            if let Some(resto) = linea.strip_prefix("@@ ") {
                // Nuevo hunk: parsear el rango original.
                let num = resto
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('-')
                    .to_string();
                let inicio: usize = num
                    .split(',')
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                // Sincronizar pos_orig al inicio del hunk (base 1 → índice 0).
                pos_orig = inicio.saturating_sub(1);
                en_hunk = true;
                continue;
            }
            if !en_hunk {
                continue;
            }
            let c = linea.chars().next().unwrap_or(' ');
            let texto = &linea[c.len_utf8()..];
            match c {
                ' ' => {
                    // Contexto: debe coincidir con la línea original.
                    if pos_orig >= orig_lines.len() || orig_lines[pos_orig] != texto {
                        return None;
                    }
                    resultado.push(orig_lines[pos_orig]);
                    pos_orig += 1;
                }
                '-' => {
                    if pos_orig >= orig_lines.len() || orig_lines[pos_orig] != texto {
                        return None;
                    }
                    pos_orig += 1;
                }
                '+' => resultado.push(texto),
                _ => {}
            }
        }

        // Adjuntar líneas restantes no cubiertas por el diff.
        while pos_orig < orig_lines.len() {
            resultado.push(orig_lines[pos_orig]);
            pos_orig += 1;
        }

        Some(resultado.join("\n"))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identico_no_hay_cambios() {
        let d = DiffEngine;
        let r = d.calcular_diff("src/App.tsx", "a\nb\nc\n", "a\nb\nc\n");
        assert!(!r.hay_cambios);
        assert_eq!(r.lineas_agregadas, 0);
        assert_eq!(r.lineas_eliminadas, 0);
    }

    #[test]
    fn test_diff_simple_detecta_agregado_y_eliminado() {
        let d = DiffEngine;
        let r = d.calcular_diff("src/App.tsx", "a\nb\nc\n", "a\nX\nc\n");
        assert!(r.hay_cambios);
        assert_eq!(r.lineas_agregadas, 1);
        assert_eq!(r.lineas_eliminadas, 1);
        assert!(r.diff_unificado.contains("@@"));
        assert!(r.diff_unificado.contains("-b"));
        assert!(r.diff_unificado.contains("+X"));
    }

    #[test]
    fn test_diff_tiene_cabeceras_de_archivo() {
        let d = DiffEngine;
        let r = d.calcular_diff("src/App.tsx", "hola\n", "hola\nadios\n");
        assert!(r.diff_unificado.contains("--- a/src/App.tsx"));
        assert!(r.diff_unificado.contains("+++ b/src/App.tsx"));
    }

    #[test]
    fn test_aplicar_diff_reconstruye_corregido() {
        let d = DiffEngine;
        let original = "linea1\nlinea2\nlinea3\n";
        let corregido = "linea1\nlineaNUEVA\nlinea3\n";
        let r = d.calcular_diff("src/App.tsx", original, corregido);
        let aplicado = d.aplicar_diff(original, &r.diff_unificado).unwrap();
        assert_eq!(aplicado, "linea1\nlineaNUEVA\nlinea3");
    }

    #[test]
    fn test_diff_vacio_o_archivo_grande() {
        let d = DiffEngine;
        // Archivo por encima del límite degrada a "reemplazado" sin panico.
        let grande = "x\n".repeat(MAX_LINEAS_LCS + 10);
        let r = d.calcular_diff("big.rs", &grande, &format!("{grande}y\n"));
        assert!(r.hay_cambios);
        assert!(r.lineas_agregadas > 0);
    }
}
