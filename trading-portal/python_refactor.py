import re

with open("src/api.rs", "r") as f:
    content = f.read()

# Remove everything before `fn load_binance_keys`
match = re.search(r'fn load_binance_keys', content)
if match:
    content = "use axum::{extract::*, response::*, Json};\nuse serde_json::json;\nuse std::sync::Arc;\nuse tracing::{info, warn};\nuse crate::state::*;\nuse crate::error::AppError;\nuse crate::nexus_futures;\nuse crate::prediccion;\nuse axum::extract::ws::{WebSocketUpgrade, WebSocket, Message as AxumMessage};\nuse futures::{sink::SinkExt, stream::StreamExt};\n\n" + content[match.start():]

# Make specific functions pub
funcs = [
    r'(async fn api_\w+)',
    r'(fn load_binance_keys)',
    r'(fn save_binance_keys)',
    r'(async fn conectar_binance_ws)',
    r'(async fn procesar_tick_mercado)',
    r'(async fn ws_handler)',
]
for pat in funcs:
    content = re.sub(r'(?<!pub\s)' + pat, r'pub \1', content)

# Remove `main`
main_match = re.search(r'#\[tokio::main\]', content)
if main_match:
    content = content[:main_match.start()]

# Now replace locks
# DashMap: state.precio_actual.lock().await -> state.precio_actual (DashMap doesn't need lock)
content = content.replace("state.precio_actual.lock().await", "state.precio_actual")
# RwLock: state.ordenes.lock().await -> state.ordenes.read() or write()
# Since we don't know which, a safe fallback is to just replace .lock().await with .write() for now, or .read(). Wait, parking_lot RwLock doesn't have .await!
# Let's replace `.lock().await` with `.write()` everywhere for RwLock except we have to be careful.
# Actually, tokio Mutex was used before, so `.lock().await`.
# Let's do regex replacements:
content = re.sub(r'state\.(\w+)\.lock\(\)\.await', r'state.\1.write()', content)
content = re.sub(r'state\.inner\.(\w+)\.lock\(\)\.await', r'state.inner.\1.write()', content)
content = re.sub(r'sim\.estado_json\(\)\.lock\(\)\.await', r'sim.estado_json()', content) # futures sim probably doesn't lock like this? Wait.

with open("src/api.rs", "w") as f:
    f.write(content)
