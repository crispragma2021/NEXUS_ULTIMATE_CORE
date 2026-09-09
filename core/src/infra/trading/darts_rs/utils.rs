// 🔱 darts_rs — Utilidades matemáticas para forecasting
// ============================================================================
// Implementaciones en Rust Puro: matrices, regresión lineal, ACF/PACF,
// Levinson-Durbin, diferenciación, métricas de error.
// Sin dependencias externas — solo f64 y std.

/// Multiplicación de matrices: A (m×n) × B (n×p) = C (m×p)
pub fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    if a.is_empty() || b.is_empty() {
        return None;
    }
    let m = a.len();
    let n = a[0].len();
    let p = b[0].len();

    if b.len() != n {
        return None;
    }

    let mut result = vec![vec![0.0_f64; p]; m];
    for i in 0..m {
        for k in 0..n {
            let aik = a[i][k];
            for j in 0..p {
                result[i][j] += aik * b[k][j];
            }
        }
    }
    Some(result)
}

/// Transposición de matriz
pub fn matrix_transpose(m: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    if m.is_empty() {
        return None;
    }
    let rows = m.len();
    let cols = m[0].len();
    let mut result = vec![vec![0.0_f64; rows]; cols];
    for i in 0..rows {
        for j in 0..cols {
            result[j][i] = m[i][j];
        }
    }
    Some(result)
}

/// Multiplica matriz transpuesta(A) por vector y: X^T y
/// Usado en OLS para el término cruzado
pub fn xt_y(x: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    if x.is_empty() || x.len() != y.len() {
        return None;
    }
    let cols = x[0].len();
    let mut result = vec![0.0_f64; cols];
    for i in 0..x.len() {
        for j in 0..cols {
            result[j] += x[i][j] * y[i];
        }
    }
    Some(result)
}

/// Multiplica matriz transpuesta(A) por matriz A: X^T X
pub fn xt_x(x: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    if x.is_empty() {
        return None;
    }
    let cols = x[0].len();
    let mut result = vec![vec![0.0_f64; cols]; cols];
    for i in 0..x.len() {
        for r in 0..cols {
            for c in 0..cols {
                result[r][c] += x[i][r] * x[i][c];
            }
        }
    }
    Some(result)
}

/// Resuelve sistema lineal Ax = b usando eliminación Gaussiana con pivoteo parcial
pub fn solve_linear_system(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = a.len();
    if n == 0 || a[0].len() != n || b.len() != n {
        return None;
    }

    // Matriz aumentada
    let mut aug: Vec<Vec<f64>> = Vec::with_capacity(n);
    for i in 0..n {
        let mut row = a[i].clone();
        row.push(b[i]);
        aug.push(row);
    }

    // Eliminación hacia adelante con pivoteo
    for col in 0..n {
        // Pivoteo parcial: encontrar fila con mayor valor absoluto
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            let val = aug[row][col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }

        if max_val < 1e-15 {
            return None; // Sistema singular
        }

        // Intercambiar filas
        aug.swap(col, max_row);

        // Eliminar filas debajo
        for row in (col + 1)..n {
            let factor = aug[row][col] / aug[col][col];
            for j in col..=(n) {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Sustitución hacia atrás
    let mut x = vec![0.0_f64; n];
    for i in (0..n).rev() {
        let mut sum = aug[i][n];
        for j in (i + 1)..n {
            sum -= aug[i][j] * x[j];
        }
        x[i] = sum / aug[i][i];
    }

    Some(x)
}

/// Estimación OLS: β = (X^T X)^(-1) X^T y
/// Retorna coeficientes β
pub fn ols_estimate(x: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let xtx = xt_x(x)?;
    let xty = xt_y(x, y)?;
    solve_linear_system(&xtx, &xty)
}

/// Genera la matriz de diseño para regresión AR(p):
/// Cada fila i contiene [y_{i-1}, y_{i-2}, ..., y_{i-p}]
pub fn lag_matrix(data: &[f64], order: usize) -> Vec<Vec<f64>> {
    if data.len() <= order {
        return Vec::new();
    }
    let n = data.len() - order;
    let mut x = Vec::with_capacity(n);
    for i in 0..n {
        let mut row = Vec::with_capacity(order);
        for lag in 1..=order {
            row.push(data[i + order - lag]);
        }
        x.push(row);
    }
    x
}

/// Genera la matriz de diseño con constante (columna de 1s) para regresión
pub fn lag_matrix_with_const(data: &[f64], order: usize, include_constant: bool) -> Vec<Vec<f64>> {
    if data.len() <= order {
        return Vec::new();
    }
    let n = data.len() - order;
    let cols = if include_constant { order + 1 } else { order };
    let mut x = Vec::with_capacity(n);
    for i in 0..n {
        let mut row = Vec::with_capacity(cols);
        if include_constant {
            row.push(1.0);
        }
        for lag in 1..=order {
            row.push(data[i + order - lag]);
        }
        x.push(row);
    }
    x
}

/// Función de autocorrelación (ACF)
pub fn acf(data: &[f64], max_lag: usize) -> Vec<f64> {
    let n = data.len();
    if n < 2 || max_lag == 0 {
        return vec![1.0];
    }

    let mean = data.iter().sum::<f64>() / n as f64;
    let centered: Vec<f64> = data.iter().map(|v| v - mean).collect();
    let variance = centered.iter().map(|v| v * v).sum::<f64>();

    if variance.abs() < 1e-15 {
        return vec![1.0; (max_lag + 1).min(n)];
    }

    let max_lag = max_lag.min(n - 1);
    let mut result = Vec::with_capacity(max_lag + 1);
    result.push(1.0); // lag 0 siempre es 1

    for lag in 1..=max_lag {
        let mut cov = 0.0;
        for i in 0..(n - lag) {
            cov += centered[i] * centered[i + lag];
        }
        result.push(cov / variance);
    }

    result
}

/// Función de autocorrelación parcial (PACF) via Levinson-Durbin
pub fn pacf(data: &[f64], max_lag: usize) -> Vec<f64> {
    let acf_vals = acf(data, max_lag);
    if acf_vals.len() <= 1 {
        return vec![1.0];
    }
    pacf_from_acf(&acf_vals)
}

/// PACF desde ACF usando algoritmo de Levinson-Durbin
pub fn pacf_from_acf(acf_vals: &[f64]) -> Vec<f64> {
    let n = acf_vals.len() - 1; // max lag
    if n == 0 {
        return vec![1.0];
    }

    let mut pacf_vals = Vec::with_capacity(n + 1);
    pacf_vals.push(1.0); // lag 0

    // Algoritmo de Levinson-Durbin
    // En cada paso k, computamos ϕ_k1, ..., ϕ_kk
    let mut phi = Vec::with_capacity(n); // phi[k] = [ϕ_k1, ..., ϕ_kk]
    let mut v = Vec::with_capacity(n); // error de predicción

    // k = 1
    let phi_11 = acf_vals[1];
    phi.push(vec![phi_11]);
    v.push(1.0 - phi_11 * phi_11);
    pacf_vals.push(phi_11);

    for k in 2..=n {
        // Compute ϕ_kk
        let mut numerator = acf_vals[k];
        for j in 1..k {
            numerator -= phi[k - 2][j - 1] * acf_vals[k - j];
        }
        let denominator = v[k - 2];
        let phi_kk = if denominator.abs() > 1e-15 {
            numerator / denominator
        } else {
            0.0
        };

        // Actualizar ϕ_kj para j = 1, ..., k-1
        let mut new_phi = Vec::with_capacity(k);
        for j in 1..k {
            let val = phi[k - 2][j - 1] - phi_kk * phi[k - 2][k - j - 1];
            new_phi.push(val);
        }
        new_phi.push(phi_kk);
        phi.push(new_phi);
        v.push(v[k - 2] * (1.0 - phi_kk * phi_kk));
        pacf_vals.push(phi_kk);
    }

    pacf_vals
}

/// Diferenciación de orden d para estacionarizar la serie
pub fn diff(data: &[f64], order: usize) -> Vec<f64> {
    if order == 0 {
        return data.to_vec();
    }
    let mut current = data.to_vec();
    for _ in 0..order {
        if current.len() < 2 {
            return Vec::new();
        }
        let mut next = Vec::with_capacity(current.len() - 1);
        for i in 1..current.len() {
            next.push(current[i] - current[i - 1]);
        }
        current = next;
    }
    current
}

/// Inversa de diferenciación: reconstruye serie original desde serie diferenciada
/// `diff_series` = serie diferenciada (longitud n-d)
/// `initial` = primeros `d` valores de la serie original
/// Retorna serie reconstruida
pub fn inverse_diff(diff_series: &[f64], initial: &[f64]) -> Vec<f64> {
    if initial.is_empty() {
        return diff_series.to_vec();
    }
    let mut result = initial.to_vec();
    for &v in diff_series {
        result.push(v + result[result.len() - 1]);
    }
    result
}

/// Inversa de diferenciación de orden d
pub fn inverse_diff_order(diff_series: &[f64], initial: &[f64], order: usize) -> Vec<f64> {
    if order == 0 {
        return diff_series.to_vec();
    }
    let mut current = diff_series.to_vec();
    let mut initials = initial.to_vec();
    // Asegurar suficientes valores iniciales
    while initials.len() < order {
        initials.insert(0, 0.0);
    }

    for _ in 0..order {
        if current.is_empty() {
            break;
        }
        let mut next = Vec::with_capacity(current.len() + 1);
        // El primer valor reconstruido usa el último initial + primer diff
        next.push(initials[initials.len() - 1] + current[0]);
        for i in 1..current.len() {
            next.push(next[i - 1] + current[i]);
        }
        current = next;
        // actualizar initials quitando el usado y añadiendo el reconstruido
        if initials.len() > 1 {
            initials = initials[1..].to_vec();
        } else {
            initials = vec![current[0]];
        }
    }
    current
}

/// Error cuadrático medio
pub fn mean_squared_error(actual: &[f64], predicted: &[f64]) -> f64 {
    if actual.is_empty() || actual.len() != predicted.len() {
        return f64::NAN;
    }
    let mse = actual
        .iter()
        .zip(predicted.iter())
        .map(|(a, p)| (a - p).powi(2))
        .sum::<f64>()
        / actual.len() as f64;
    mse
}

/// Raíz del error cuadrático medio
pub fn rmse(actual: &[f64], predicted: &[f64]) -> f64 {
    mean_squared_error(actual, predicted).sqrt()
}

/// Error absoluto medio
pub fn mean_absolute_error(actual: &[f64], predicted: &[f64]) -> f64 {
    if actual.is_empty() || actual.len() != predicted.len() {
        return f64::NAN;
    }
    actual
        .iter()
        .zip(predicted.iter())
        .map(|(a, p)| (a - p).abs())
        .sum::<f64>()
        / actual.len() as f64
}

/// Promedio móvil simple (usado para suavizado y pronóstico naive)
pub fn simple_moving_average(data: &[f64], window: usize) -> Vec<f64> {
    if data.len() < window || window == 0 {
        return vec![f64::NAN; data.len()];
    }
    let mut result = vec![f64::NAN; data.len()];
    let mut sum: f64 = data[0..window].iter().sum();
    result[window - 1] = sum / window as f64;
    for i in window..data.len() {
        sum += data[i] - data[i - window];
        result[i] = sum / window as f64;
    }
    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply_identity() {
        let a = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let b = vec![vec![3.0, 4.0], vec![5.0, 6.0]];
        let result = matrix_multiply(&a, &b).unwrap();
        assert_eq!(result[0][0], 3.0);
        assert_eq!(result[1][1], 6.0);
    }

    #[test]
    fn test_matrix_multiply_dimension_mismatch() {
        let a = vec![vec![1.0, 2.0]];
        let b = vec![vec![3.0]];
        assert!(matrix_multiply(&a, &b).is_none());
    }

    #[test]
    fn test_matrix_transpose_square() {
        let m = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let t = matrix_transpose(&m).unwrap();
        assert_eq!(t[0][0], 1.0);
        assert_eq!(t[1][0], 2.0);
    }

    #[test]
    fn test_solve_linear_system_2x2() {
        // 2x + 3y = 8
        //  x + 2y = 5
        let a = vec![vec![2.0, 3.0], vec![1.0, 2.0]];
        let b = vec![8.0, 5.0];
        let x = solve_linear_system(&a, &b).unwrap();
        assert!((x[0] - 1.0).abs() < 1e-10);
        assert!((x[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_solve_singular_returns_none() {
        let a = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let b = vec![3.0, 6.0];
        assert!(solve_linear_system(&a, &b).is_none());
    }

    #[test]
    fn test_ols_simple() {
        // y = 2x + 1 con ruido mínimo
        let x = vec![
            vec![1.0, 1.0],
            vec![1.0, 2.0],
            vec![1.0, 3.0],
            vec![1.0, 4.0],
            vec![1.0, 5.0],
        ];
        let y = vec![3.0, 5.0, 7.0, 9.0, 11.0];
        let beta = ols_estimate(&x, &y).unwrap();
        assert!((beta[0] - 1.0).abs() < 1e-10); // intercepto
        assert!((beta[1] - 2.0).abs() < 1e-10); // pendiente
    }

    #[test]
    fn test_lag_matrix() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let lags = lag_matrix(&data, 2);
        assert_eq!(lags.len(), 3);
        // Column order: [y_{t-1}, y_{t-2}]
        // Row 0 predicts y[2]=3 from y[1]=2, y[0]=1
        assert_eq!(lags[0], vec![2.0, 1.0]);
        // Row 2 predicts y[4]=5 from y[3]=4, y[2]=3
        assert_eq!(lags[2], vec![4.0, 3.0]);
    }

    #[test]
    fn test_lag_matrix_with_const() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let lags = lag_matrix_with_const(&data, 2, true);
        assert_eq!(lags.len(), 2);
        // [const=1, y_{t-1}=2, y_{t-2}=1]
        assert_eq!(lags[0], vec![1.0, 2.0, 1.0]);
        // [const=1, y_{t-1}=3, y_{t-2}=2]
        assert_eq!(lags[1], vec![1.0, 3.0, 2.0]);
    }

    #[test]
    fn test_acf_lag0_is_1() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let acf_vals = acf(&data, 3);
        assert!((acf_vals[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_acf_constant_series() {
        let data = vec![5.0; 10];
        let acf_vals = acf(&data, 3);
        for &v in &acf_vals {
            assert!(
                (v - 1.0).abs() < 1e-10,
                "ACF should be 1 for constant series"
            );
        }
    }

    #[test]
    fn test_pacf_ar1_process() {
        // AR(1): y_t = 0.7*y_{t-1} + e_t
        let mut data = vec![0.0; 100];
        let mut rng = 12345u64;
        for i in 1..100 {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let noise = (rng as f64 / u64::MAX as f64) * 2.0 - 1.0;
            data[i] = 0.7 * data[i - 1] + noise * 0.5;
        }
        let pacf_vals = pacf(&data, 5);
        // Lag 1 debe dominar (cerca de 0.7)
        assert!(
            pacf_vals[1].abs() > 0.3,
            "PACF[1] = {} should be significant",
            pacf_vals[1]
        );
        // Lags > 1 deben ser cercanos a 0
        for lag in 2..=4 {
            assert!(
                pacf_vals[lag].abs() < 0.5,
                "PACF[{}] = {} should be small for AR(1)",
                lag,
                pacf_vals[lag]
            );
        }
    }

    #[test]
    fn test_diff_first_order() {
        let data = vec![1.0, 3.0, 6.0, 10.0];
        let d = diff(&data, 1);
        assert_eq!(d, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_diff_second_order() {
        let data = vec![1.0, 3.0, 6.0, 10.0];
        let d = diff(&data, 2);
        assert_eq!(d, vec![1.0, 1.0]);
    }

    #[test]
    fn test_inverse_diff() {
        let original = vec![10.0, 15.0, 13.0, 20.0];
        let d = diff(&original, 1);
        let reconstructed = inverse_diff(&d, &original[..1]);
        assert_eq!(reconstructed, original);
    }

    #[test]
    fn test_mse_perfect_prediction() {
        let actual = vec![1.0, 2.0, 3.0];
        let predicted = vec![1.0, 2.0, 3.0];
        assert!((mean_squared_error(&actual, &predicted) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_rmse() {
        let actual = vec![1.0, 3.0];
        let predicted = vec![1.0, 1.0]; // MSE = ((0)^2 + (2)^2)/2 = 4/2 = 2, RMSE = sqrt(2) ≈ 1.4142
        let expected_rmse = (2.0_f64).sqrt();
        assert!((rmse(&actual, &predicted) - expected_rmse).abs() < 1e-10);
    }

    #[test]
    fn test_sma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sma = simple_moving_average(&data, 3);
        assert!((sma[2] - 2.0).abs() < 1e-10);
        assert!((sma[4] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_xt_y_basic() {
        let x = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let y = vec![5.0, 6.0];
        let result = xt_y(&x, &y).unwrap();
        assert!((result[0] - (1.0 * 5.0 + 3.0 * 6.0)).abs() < 1e-10);
        assert!((result[1] - (2.0 * 5.0 + 4.0 * 6.0)).abs() < 1e-10);
    }

    #[test]
    fn test_ols_with_lags() {
        // AR(1): y_t = 0.5*y_{t-1} + 2
        let data = vec![2.0, 3.0, 3.5, 3.75, 3.875, 3.9375];
        let x = lag_matrix_with_const(&data, 1, true);
        let y: Vec<f64> = data[1..].to_vec();
        let beta = ols_estimate(&x, &y).unwrap();
        // intercepto ≈ 2, pendiente ≈ 0.5
        assert!((beta[1] - 0.5).abs() < 0.1, "AR coeff = {}", beta[1]);
    }

    #[test]
    fn test_levinson_pacf_from_acf_ar1() {
        // Para AR(1) con ϕ=0.7, PACF[1] ≈ 0.7, PACF[k>1] ≈ 0
        // ACF decae geométricamente: ρ_k = ϕ^k
        let phi: f64 = 0.7;
        let mut acf_vals = Vec::with_capacity(6);
        acf_vals.push(1.0);
        for k in 1..=5 {
            acf_vals.push(phi.powi(k as i32));
        }
        let pacf_vals = pacf_from_acf(&acf_vals);
        assert!(
            (pacf_vals[1] - 0.7).abs() < 0.01,
            "PACF[1] = {}",
            pacf_vals[1]
        );
        assert!(
            pacf_vals[2].abs() < 0.1,
            "PACF[2] = {} should be near 0",
            pacf_vals[2]
        );
    }
}
