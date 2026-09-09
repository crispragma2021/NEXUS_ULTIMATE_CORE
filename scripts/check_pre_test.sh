#!/usr/bin/env bash
# ============================================================================
# 🔍 CHECK PRE-TEST — Sistema de Verificación + Auto-Corrección Pre-Compilación
# ============================================================================
# Detecta y CORRIGE errores conocidos ANTES de ejecutar cargo test.
# Uso: ./scripts/check_pre_test.sh [--fix]
#   --fix   Aplica correcciones automáticas cuando sea posible
# ============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIX_MODE=false
TOTAL_ERRORS=0

[[ "${1:-}" == "--fix" ]] && FIX_MODE=true

err()   { echo "❌ $*"; TOTAL_ERRORS=$((TOTAL_ERRORS + 1)); }
ok()    { echo "✅ $*"; }
warn()  { echo "⚠️  $*"; }
info()  { echo "🔍 $*"; }
fix()   { echo "🛠️  $*"; }

# ============================================================================
# VERIFICACIÓN 1: Imports duplicados (TokioMutex, std::sync::Mutex, etc.)
# ============================================================================
# Error típico: TokioMutex importado globalmente (línea 21) y re-importado
# dentro de mod tests. Causa E0252.
# Solución: Eliminar el import duplicado del bloque mod tests.
# ============================================================================
check_duplicate_imports() {
    local file="$1" fname="$2"
    local errors=0

    local global_imports test_imports
    global_imports=$(sed -n '/^use /p' "$file" 2>/dev/null || echo "")
    test_imports=$(sed -n '/^mod tests/,/^}/p' "$file" 2>/dev/null || echo "")

    # ── TokioMutex duplicado ─────────────────────────────────────────────
    if echo "$global_imports" | grep -q "TokioMutex" && echo "$test_imports" | grep -q "TokioMutex"; then
        err "$fname: TokioMutex duplicado (global + mod tests) — causa E0252"
        if $FIX_MODE; then
            # Eliminar línea con TokioMutex dentro de mod tests
            sed -i '/^mod tests/,/^}/ { /TokioMutex/d; }' "$file"
            fix "$fname: TokioMutex eliminado de mod tests"
        fi
        errors=1
    fi

    # ── std::sync::Mutex re-importado en tests ───────────────────────────
    if echo "$global_imports" | grep -q "use std::sync::Mutex" && echo "$test_imports" | grep -q "use std::sync::Mutex"; then
        warn "$fname: std::sync::Mutex re-importado en mod tests (no es error, pero es redundante)"
        if $FIX_MODE; then
            sed -i '/^mod tests/,/^}/ { /use std::sync::Mutex/d; }' "$file"
            fix "$fname: std::sync::Mutex redundante eliminado de mod tests"
        fi
    fi

    return $errors
}

# ============================================================================
# VERIFICACIÓN 2: Mutex<Connection> sin .lock()
# ============================================================================
# Error típico: conn: Mutex<Connection> pero se llama self.conn.execute()
# sin self.conn.lock().unwrap().execute(). Causa E0599.
# Solución: Reemplazar self.conn.metodo() por self.conn.lock().unwrap().metodo()
# ============================================================================
check_mutex_connection() {
    local file="$1" fname="$2"
    local errors=0

    local has_mutex_conn
    has_mutex_conn=$(grep -c "conn.*Mutex.*Connection" "$file" 2>/dev/null || echo "0")

    if [[ "$has_mutex_conn" -gt 0 ]]; then
        # Buscar llamadas directas: self.conn.metodo(
        # donde metodo es execute|query_row|prepare|query
        local line_num line_content
        while IFS=: read -r line_num line_content; do
            [[ -z "$line_num" ]] && continue
            # Verificar que la línea NO tenga .lock() ni sea parte de conn_guard
            if ! echo "$line_content" | grep -q "\.lock()" && ! echo "$line_content" | grep -q "conn_guard"; then
                # Extraer el método llamado
                local metodo
                metodo=$(echo "$line_content" | sed -n 's/.*self\.conn\.\([a-zA-Z_]*\).*/\1/p')
                if [[ -n "$metodo" ]]; then
                    err "$fname: línea $line_num — self.conn.$metodo() sin .lock()"
                    err "  → $line_content"
                    if $FIX_MODE; then
                        # Reemplazar self.conn.metodo( por self.conn.lock().unwrap().metodo(
                        local old_text="self.conn.$metodo("
                        local new_text="self.conn.lock().unwrap().$metodo("
                        # Pero solo si está en una línea simple (no multi-line)
                        if ! echo "$line_content" | grep -q "//.*$old_text"; then
                            sed -i "${line_num}s/$old_text/$new_text/" "$file"
                            fix "$fname: línea $line_num — añadido .lock().unwrap() a $metodo"
                        fi
                    fi
                    errors=$((errors + 1))
                fi
            fi
        done < <(grep -n "self\.conn\.\(execute\|query_row\|prepare\|query\)" "$file" 2>/dev/null || echo "")
    fi

    return $errors
}

# ============================================================================
# VERIFICACIÓN 3: Runtime::new() dentro de tests (nesting)
# ============================================================================
# Error típico: Runtime::new().block_on(async { ... }) dentro de una función
# auxiliar llamada desde #[tokio::test]. Causa "Cannot start a runtime from
# within a runtime".
# Solución: Reemplazar la función por async y usar .await directamente.
# ============================================================================
check_runtime_nesting() {
    local file="$1" fname="$2"
    local errors=0

    local test_section
    test_section=$(sed -n '/^mod tests/,/^}/p' "$file" 2>/dev/null || echo "")

    # Buscar Runtime::new en tests
    local runtime_lines
    runtime_lines=$(echo "$test_section" | grep -n "Runtime::new" || echo "")
    if [[ -n "$runtime_lines" ]]; then
        while IFS= read -r line; do
            local absolute_line
            # La línea absoluta = offset de mod tests + línea local
            local mod_tests_line
            mod_tests_line=$(grep -n "^mod tests" "$file" | head -1 | cut -d: -f1)
            absolute_line=$((mod_tests_line + $(echo "$line" | cut -d: -f1)))
            err "$fname: línea $absolute_line — Runtime::new() dentro de tests"
            if $FIX_MODE; then
                warn "$fname: No se puede auto-corregir Runtime::new() — requiere refactor manual"
                err "  → Reemplazar por función async y usar .await directamente"
            fi
            errors=$((errors + 1))
        done <<< "$runtime_lines"
    fi

    # Buscar block_on() fuera de main y fuera de tests
    local all_block_on
    all_block_on=$(grep -n "block_on" "$file" 2>/dev/null || echo "")
    if [[ -n "$all_block_on" ]]; then
        while IFS= read -r line; do
            local context
            local bline_num
            bline_num=$(echo "$line" | cut -d: -f1)
            context=$(sed -n "$((bline_num-3)),$((bline_num+1))p" "$file" 2>/dev/null)
            if ! echo "$context" | grep -q "#\[tokio::test\]" && ! echo "$context" | grep -q "fn main"; then
                warn "$fname: línea $bline_num — block_on() fuera de main/test (posible nesting)"
            fi
        done <<< "$all_block_on"
    fi

    return $errors
}

# ============================================================================
# VERIFICACIÓN 4: HTTP/reqwest en tests sin timeout
# ============================================================================
# Error típico: Test llama a generar_embedding() que hace HTTP request a
# LanceDB memory:// que no responde. Causa test cuelga >60s.
# Solución: Envolver test en tokio::time::timeout().
# ============================================================================
check_http_in_tests() {
    local file="$1" fname="$2"
    local errors=0

    local test_section
    test_section=$(sed -n '/^mod tests/,/^}/p' "$file" 2>/dev/null || echo "")

    if echo "$test_section" | grep -qE "(reqwest|Client::new|generar_embedding|memory://|sumergir)"; then
        local test_fns timeouts
        test_fns=$(echo "$test_section" | grep -c "#\[tokio::test\]" 2>/dev/null || echo "0")
        timeouts=$(echo "$test_section" | grep -c "timeout" 2>/dev/null || echo "0")

        if [[ "$test_fns" -gt 0 && "$timeouts" -eq 0 ]]; then
            err "$fname: $test_fns tests tokio, 0 con timeout — riesgo de cuelgue por HTTP"
            if $FIX_MODE; then
                warn "$fname: No se puede auto-corregir — requiere envolver cada test en timeout manualmente"
            fi
            errors=$((errors + 1))
        elif [[ "$timeouts" -lt "$test_fns" ]]; then
            local sin_timeout
            sin_timeout=$((test_fns - timeouts))
            warn "$fname: $sin_timeout de $test_fns tests sin timeout"
        fi
    fi

    return $errors
}

# ============================================================================
# VERIFICACIÓN 5: MutexGuard cruzando .await (Send violation)
# ============================================================================
# Error típico: let guard = self.conn.lock().unwrap(); ... .await ... guard.metodo()
# std::sync::MutexGuard no es Send, no puede cruzarse en .await.
# Solución: Mover el uso de guard antes del .await, o usar tokio::sync::Mutex.
# ============================================================================
check_mutexguard_across_await() {
    local file="$1" fname="$2"
    local errors=0

    local lock_lines
    lock_lines=$(grep -n "let.*=.*\.lock()\.unwrap()" "$file" 2>/dev/null || echo "")

    if [[ -n "$lock_lines" ]]; then
        while IFS=: read -r line_num line_content; do
            [[ -z "$line_num" ]] && continue
            local var_name
            var_name=$(echo "$line_content" | sed -n 's/.*let[[:space:]]*\([a-zA-Z_][a-zA-Z0-9_]*\).*/\1/p')
            [[ -z "$var_name" ]] && continue

            # Buscar si hay .await en las siguientes 30 líneas
            local after_lock
            after_lock=$(sed -n "$((line_num+1)),$((line_num+30))p" "$file" 2>/dev/null)
            local uses_after
            uses_after=$(echo "$after_lock" | grep -c "$var_name" 2>/dev/null || echo "0")
            local has_await
            has_await=$(echo "$after_lock" | grep -c "\.await" 2>/dev/null || echo "0")

            if [[ "$has_await" -gt 0 && "$uses_after" -gt 0 ]]; then
                err "$fname: línea $line_num — $var_name (MutexGuard<std::sync::Mutex>) cruza .await"
                err "  → MutexGuard no es Send, no se puede mantener entre .await"
                if $FIX_MODE; then
                    warn "$fname: No se puede auto-corregir — requiere reestructurar:"
                    err "  → Opción 1: Usar bloque { let g = x.lock().unwrap(); ... } antes del .await"
                    err "  → Opción 2: Cambiar a tokio::sync::Mutex (MutexGuard es Send)"
                fi
                errors=$((errors + 1))
            fi
        done <<< "$lock_lines"
    fi

    return $errors
}

# ============================================================================
# EJECUCIÓN PRINCIPAL
# ============================================================================

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║   🔍 NEXUS PRE-TEST VERIFICATION SYSTEM v1.0               ║"
echo "║   Auto-detección + corrección de errores conocidos         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

FILES_TO_CHECK=(
    "$ROOT/core/src/cerebro/mundo_interno.rs"
    "$ROOT/core/src/cerebro/ocean.rs"
    "$ROOT/core/src/emociones/limbico.rs"
)

for file in "${FILES_TO_CHECK[@]}"; do
    if [[ ! -f "$file" ]]; then
        warn "Archivo no encontrado: $file"
        continue
    fi

    local_fname=$(basename "$file")
    echo "─── [$local_fname] ───────────────────────────────────────"

    check_duplicate_imports        "$file" "$local_fname" || true
    check_mutex_connection         "$file" "$local_fname" || true
    check_runtime_nesting          "$file" "$local_fname" || true
    check_http_in_tests            "$file" "$local_fname" || true
    check_mutexguard_across_await  "$file" "$local_fname" || true

    echo ""
done

echo "═══════════════════════════════════════════════════════════════"
if [[ "$TOTAL_ERRORS" -eq 0 ]]; then
    ok "No se detectaron errores conocidos. Listo para compilar."
else
    err "Se detectaron $TOTAL_ERRORS errores."
    if $FIX_MODE; then
        warn "Algunos errores requieren corrección manual. Revisar mensajes."
    else
        warn "Ejecutar con --fix para aplicar correcciones automáticas."
    fi
fi
echo ""

exit $TOTAL_ERRORS
