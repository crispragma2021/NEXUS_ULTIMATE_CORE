# NEXUS Sovereign Justfile 🔱⚡🛡️

# Default task: list all commands
default:
    @just --list

# Start development server (Frontend + Backend)
dev:
    @echo "🚀 Starting NEXUS Development Environment..."
    cargo tauri dev

# Run all tests with Nextest
test:
    @echo "🏎️ Running Elite Test Suite..."
    cargo nextest run

# Professional Audit
audit:
    @echo "🛡️ Running Sentinel Security Audit..."
    ./scripts/nexus_audit.sh

# --- TRIPLE SOVEREIGN COMMANDS ---

# Start full sovereign environment
sovereign:
    @echo "🔱 Activating NEXUS Triple Sovereign Mode..."
    @just voice &
    @just heal &
    @just dev

# Start autonomous healer
heal:
    @echo "🩹 Activating Autonomous Healer..."
    python3 scripts/autonomous_healer.py

# Start voice ear
voice:
    @echo "🎙️ Activating Sovereign Voice Ear..."
    python3 skills/voice_listener.py

# Release the Chaos Monkey (Verification Test)
monkey:
    @echo "🐒 Releasing the Chaos Monkey..."
    python3 scripts/chaos_monkey.py

# Clean build artifacts
clean:
    @echo "🧹 Cleaning workspace..."
    cargo clean

squadron:
	@echo "🤖 Activando Escuadrón NEXUS (CrewAI)..."
	python3 scripts/nexus_squadron.py

# --- ORCHESTRATION COMMANDS ---

# List available free models on OpenRouter
models-list:
	python3 scripts/model_orchestrator.py list

# Set primary model (e.g., just model-set deepseek/deepseek-r1:free)
model-set model:
	python3 scripts/model_orchestrator.py set {{model}}

# Activate GLM5 Orchestration Loop (Active Build Mode)
glm5-active:
	@echo "🧠 [GLM5] Launching Agent Injection Protocol..."
	./scripts/glm5_orchestrator.sh

# Toggle OpenRouter fallback mode
fallback mode="on":
	python3 scripts/model_orchestrator.py fallback {{mode}}

# Refresh the Model Registry (Neural Sync)
refresh-models:
	@echo "🧠 [NEURAL] Refreshing Model Registry..."
	./scripts/setup_ollama_hybrid.sh

# Build eBPF Kernel components
build-ebpf:
	@echo "🔬 [KERNEL] Building eBPF bytecode (xtask)..."
	cargo xtask build-ebpf --release

# Inject OpenRouter API Key into Vault
openrouter-key key:
	@sqlite3 $$HOME/.nexus_data/nexus.db "INSERT OR REPLACE INTO preferences (key, value) VALUES ('OPENROUTER_API_KEY', '{{key}}');"
	@sqlite3 $$HOME/.nexus_data/nexus.db "INSERT OR REPLACE INTO facts (key, value, context) VALUES ('OPENROUTER_API_KEY', '{{key}}', 'secrets');"
	@echo "✅ OpenRouter API Key Injected into Vault"

# Run the high-speed Rust DB Sentinel
db-audit:
	./scripts/rust_tools/db_sentinel/target/release/db_sentinel $$HOME/.nexus_data/nexus.db

# Build the project
build:
	nice -n 15 ionice -c 3 cargo build -j 14

# Build the Rust DB Sentinel
db-build:
	cd scripts/rust_tools/db_sentinel && nice -n 15 ionice -c 3 cargo build --release -j $(nproc | awk '{print ($1 > 2 ? $1 - 2 : 1)}')

# Activate Eco-Mode (Hibernates heavy services)
eco-on:
	python3 scripts/eco_sentinel.py on

# Deactivate Eco-Mode (Restores services)
eco-off:
	python3 scripts/eco_sentinel.py off

# Check resource status
eco-status:
	python3 scripts/eco_sentinel.py status

# Start the Guardian Mode monitor (Background)
guardian-start:
	@nohup python3 scripts/eco_sentinel.py guardian > /dev/null 2>&1 &
	@echo "🛰️ Guardian Mode Active: NEXUS is watching for High-Performance Tasks..."

# --- CLOUD CONSOLIDATION ---

# Sync project to Google Drive (Recursive with exclusions)
sync-drive:
	./.venv/bin/python3 nexus_drive_consolidator.py

# --- SYSTEMD SERVICE MANAGEMENT ---

# Install and enable NEXUS system services
services-install:
	@sudo cp systemd/*.service /etc/systemd/system/
	@sudo cp systemd/*.timer /etc/systemd/system/
	@sudo systemctl daemon-reload
	@sudo systemctl enable nexus-watchdog.timer nexus_sensor.timer nexus.service nexus-dashboard.service
	@echo "✅ Services installed and timers enabled."

# Start all NEXUS services and timers
services-start:
	@sudo systemctl start nexus-watchdog.timer nexus_sensor.timer
	@sudo systemctl start nexus-watchdog.service nexus_sensor.service nexus.service nexus-dashboard.service
	@echo "🚀 Services and Timers started."

# Check status of NEXUS services
services-status:
	@systemctl status nexus-watchdog.timer nexus_sensor.timer --no-pager
	@systemctl status nexus-watchdog.service nexus_sensor.service --no-pager

# Restart all NEXUS services
services-restart:
	@sudo systemctl restart nexus-watchdog.service nexus_sensor.service
	@echo "🔄 Services restarted."
# --- ZENITH OMEGA (Monolith Mode) ---

# Run the OMEGA++ Supreme Core (Native Rust)
omega:
	@echo "🛡️ Launching OMEGA++ Supreme Core..."
	./NEXUS_CORE/target/release/nexus_core

# Build and seal the Zenith Core with maximum LTO
seal:
	@echo "🧊 Sealing Zenith Architecture (LTO Release)..."
	nice -n 15 ionice -c 3 cargo build --release -j $(nproc | awk '{print ($1 > 4 ? $1 - 4 : 2)}')

# Final OMEGA Audit
omega-audit:
    @./scripts/nexus_audit.sh

# --- DIVINE OPTIMIZATION ---

# Execute Divine Ignition (PGO + BOLT)
divine:
	@chmod +x scripts/divine_ignition.sh
	@./scripts/divine_ignition.sh

# --- PERFORMANCE: Antigravity Coexistence ---

# Fix 3: cargo check con prioridad I/O idle y CPU nice máxima — fluido absoluto
check:
	@echo "🔬 [NEXUS] cargo check (nice 19 + ionice idle — simbiosis total)..."
	nice -n 19 ionice -c 3 cargo check -p nexus_ultimate_core

# Check completo del workspace con prioridad mínima
check-all:
	@echo "🔬 [NEXUS] Workspace check completo (nice 19 + ionice idle)..."
	nice -n 19 ionice -c 3 cargo check --workspace

# Asegurar que /tmp/nexus-target existe en tmpfs antes de compilar
tmpfs-target:
	@mkdir -p /tmp/nexus-target
	@echo "✅ /tmp/nexus-target listo (compilaciones en RAM, SSD libre)"

# --- DISTRIBUTION SYNTHESIS ---

# Professional Packaging for Distribution
distribute: seal
	@echo "📦 Synthesizing Sovereign Distribution..."
	@mkdir -p dist/bin dist/scripts dist/config
	@cp target/release/nexus_* dist/bin/ || true
	@cp target/release/mcp_*_native dist/bin/ || true
	@cp bin/* dist/bin/ || true
	@cp scripts/*.sh dist/scripts/ || true
	@cp scripts/*.py dist/scripts/ || true
	@cp -r .agent dist/
	@cp Cargo.toml CODEBASE.md README.md dist/
	@echo "✅ Distribution package ready in dist/"
# --- MONOLITHIC UNIFICATION ---

# Launch the Unified NEXUS Monolith (Core + UI + Vision)
launch:
	@echo "🔱 Launching Sovereign Monolith..."
	@just omega-core &
	@just omega-vision &
	@just omega-ui

omega-core:
	@echo "🧠 [CORE] Starting Brain (Threads 4-15)..."
	cargo run --package nexus_core --release

omega-vision:
	@echo "👁️ [VISION] Starting Ojo (Threads 0-3)..."
	cargo run --package nexus_vision --release

omega-ui:
	@echo "🎨 [UI] Starting Dashboard (Core 0)..."
	cargo run --package nexus_ui --release

# Clean all members
clean-all:
	cargo clean
	@rm -rf nexus_core/target nexus_vision/target nexus_ui/target
# --- LEGACY / ARCHIVE (Asimilado por LanceDB/Identidad) ---

# Launch Qdrant Vector Memory (ARCHIVED)
qdrant-up-legacy:
	@echo "⚠️ [LEGADO] Qdrant ha sido reemplazado por LanceDB."
	@docker start nexus_qdrant || docker run -d --name nexus_qdrant -p 6333:6333 -p 6334:6334 --memory="512m" -v $$HOME/.qdrant:/qdrant/storage qdrant/qdrant:latest

# Index the NEXUS codebase semantically (ARCHIVED)
qdrant-index-legacy:
	@echo "⚠️ [LEGADO] Usa el motor de Conciencia Soberana (LanceDB) ahora."
	python3 legado/scripts/qdrant_manager.py index
