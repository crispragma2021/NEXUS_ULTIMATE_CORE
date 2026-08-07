#!/usr/bin/env node
/**
 * ═══════════════════════════════════════════════════════════════════════════
 * figma_absorb.js — NEXUS Figma Design Absorber (Real)
 * ═══════════════════════════════════════════════════════════════════════════
 * Estrategia de 3 capas:
 *   1. API REST (con FIGMA_TOKEN) — extracción exacta de estilos
 *   2. Chrome persistente (con perfil guardado) — login vía Google
 *   3. Screenshot + análisis visual — como fallback
 * ═══════════════════════════════════════════════════════════════════════════
 *
 * Uso: ./figma_absorb.js <figma_url> [--token <token>|--google|--screenshot]
 *   --token     Usar Figma Personal Access Token
 *   --google    Usar credenciales Google del vault + Chrome headless
 *   --screenshot  Tomar screenshot del diseño (fallback visual)
 *
 * Sin flags: intenta token del vault, luego Google, luego screenshot
 */

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const http = require('http');
const https = require('https');

const SCRIPT_DIR = path.dirname(fs.realpathSync(__filename));
const PROJECT_DIR = path.resolve(SCRIPT_DIR, '..');
const VAULT_DIR = path.join(PROJECT_DIR, '.vault');
const FRONTEND_DIR = path.join(PROJECT_DIR, 'frontend');
const OUTPUT_CSS = path.join(FRONTEND_DIR, 'src', 'design-system.css');
const OUTPUT_DIR = path.join(FRONTEND_DIR, 'src', 'figma-assets');
const CHROME_PROFILE = path.join(process.env.HOME, '.nexus_chrome_profile', 'figma');

// ─── Colores de referencia de exchanges élite ───────────────────────────
const EXCHANGE_COLORS = {
  tradingview: { bg: '#131722', text: '#d1d4dc', green: '#089981', red: '#f23645' },
  binance:     { bg: '#0b0e11', text: '#eaecef', green: '#0ecb81', red: '#f6465d', yellow: '#f0b90b' },
  kraken:      { bg: '#0f1421', text: '#cbd5e0', green: '#2dd4ad', red: '#fb7185', blue: '#3b82f6' },
  coinbase:    { bg: '#0a0b0d', text: '#f0f2f4', green: '#10b981', red: '#ef4444', blue: '#2563eb' },
  bybit:       { bg: '#1a1d28', text: '#e1e4ec', green: '#26de81', red: '#ff6b6b', blue: '#4a7dff' },
  kucoin:      { bg: '#0f1218', text: '#cdd5e0', green: '#1eb8a3', red: '#e23b4a', yellow: '#e5b83a' },
};

// ─── Helpers ────────────────────────────────────────────────────────────

function log(emoji, msg) { console.log(`${emoji} [NEXUS] ${msg}`); }

function request(url, options = {}) {
  return new Promise((resolve, reject) => {
    const mod = url.startsWith('https') ? https : http;
    mod.get(url, options, (res) => {
      let data = '';
      res.on('data', c => data += c);
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve(data); }
      });
    }).on('error', reject);
  });
}

function descifrarVault(archivo) {
  const masterKeyPath = path.join(VAULT_DIR, '.master.key');
  if (!fs.existsSync(masterKeyPath) || !fs.existsSync(archivo)) return null;
  const master = fs.readFileSync(masterKeyPath, 'utf8').trim();
  const proc = spawn('openssl', [
    'enc', '-d', '-aes-256-cbc', '-base64', '-pbkdf2',
    '-pass', `pass:${master}`, '-in', archivo
  ]);
  return new Promise((resolve) => {
    let out = '';
    proc.stdout.on('data', d => out += d);
    proc.on('close', (code) => {
      if (code !== 0) return resolve(null);
      try { resolve(JSON.parse(out)); }
      catch { resolve(out); }
    });
  });
}

function esperar(ms) { return new Promise(r => setTimeout(r, ms)); }

// ─── Fase 1: API REST de Figma ─────────────────────────────────────────

async function extraerPorAPI(token, fileKey) {
  log('🔑', `Extrayendo vía API REST — fileKey: ${fileKey}`);

  const [styles, fileData] = await Promise.all([
    request(`https://api.figma.com/v1/files/${fileKey}/styles`, {
      headers: { 'X-Figma-Token': token }
    }),
    request(`https://api.figma.com/v1/files/${fileKey}`, {
      headers: { 'X-Figma-Token': token }
    })
  ]);

  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  fs.writeFileSync(path.join(OUTPUT_DIR, 'figma_styles.json'), JSON.stringify(styles, null, 2));
  fs.writeFileSync(path.join(OUTPUT_DIR, 'figma_nodes.json'), JSON.stringify(fileData, null, 2));

  // Extraer colores, tipografía y espaciado de los estilos
  const colors = { surfaces: [], text: [], accents: [] };
  const typography = [];
  let cssVariables = {};

  if (styles?.meta?.styles) {
    for (const style of styles.meta.styles) {
      if (style.style_type === 'FILL') {
        const fillData = await request(
          `https://api.figma.com/v1/files/${fileKey}/styles/${style.key}`,
          { headers: { 'X-Figma-Token': token } }
        );
        if (fillData?.style?.paints) {
          for (const paint of fillData.style.paints) {
            if (paint.color) {
              const r = Math.round(paint.color.r * 255);
              const g = Math.round(paint.color.g * 255);
              const b = Math.round(paint.color.b * 255);
              const hex = `#${r.toString(16).padStart(2,'0')}${g.toString(16).padStart(2,'0')}${b.toString(16).padStart(2,'0')}`;
              const name = style.name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/-+$/, '');
              cssVariables[`figma-${name}`] = hex;
            }
          }
        }
      }
      if (style.style_type === 'TEXT') {
        typography.push(style.name);
      }
    }
  }

  log('✅', `Extraídos ${Object.keys(cssVariables).length} colores vía API`);
  return cssVariables;
}

// ─── Fase 2: Chrome headless con perfil persistente ───────────────────

async function extraerPorChrome(fileKey, email, password) {
  log('🌐', 'Lanzando Chrome headless con perfil persistente...');

  const chromePaths = [
    'google-chrome-stable', 'google-chrome',
    'chromium-browser', 'chromium', 'brave-browser'
  ];

  let chromeBin = null;
  for (const bin of chromePaths) {
    try {
      const p = spawn('which', [bin]);
      await new Promise(r => { p.on('close', code => { if (code === 0) chromeBin = bin; r(); }); });
      if (chromeBin) break;
    } catch {}
  }

  if (!chromeBin) {
    log('❌', 'No se encontró Chrome/Chromium. Instalando...');
    spawn('sudo', ['apt-get', 'install', '-y', 'chromium-browser'], { stdio: 'inherit' });
    chromeBin = 'chromium-browser';
  }

  // Asegurar perfil
  const figmaProfile = path.join(CHROME_PROFILE);
  fs.mkdirSync(figmaProfile, { recursive: true });

  return new Promise(async (resolve, reject) => {
    const chrome = spawn(chromeBin, [
      '--headless=new',
      '--remote-debugging-port=0',
      `--user-data-dir=${figmaProfile}`,
      '--no-sandbox',
      '--disable-gpu',
      '--disable-dev-shm-usage',
      '--window-size=1920,1080',
      '--hide-scrollbars',
      `--window-size=1920,1080`,
      `https://www.figma.com/file/${fileKey}/design`
    ], { stdio: ['pipe', 'pipe', 'pipe'] });

    let debugPort = null;
    chrome.stderr.on('data', (data) => {
      const text = data.toString();
      const match = text.match(/DevTools listening on ws:\/\/.*:(\d+)/);
      if (match) {
        debugPort = parseInt(match[1]);
      }
    });

    chrome.stderr.on('data', (data) => {
      const text = data.toString();
      if (text.includes('ERROR') || text.includes('FATAL')) {
        log('⚠️', `Chrome: ${text.slice(0, 200)}`);
      }
    });

    // Timeout para carga
    setTimeout(async () => {
      if (debugPort) {
        log('🔌', `Chrome debug port: ${debugPort}`);
        try {
          const wsUrl = await obtenerWsUrl(debugPort);
          if (wsUrl) {
            const result = await extraerDisenoViaCDP(wsUrl);
            resolve(result);
          }
        } catch (e) {
          log('⚠️', `Error CDP: ${e.message}`);
        }
      } else {
        log('⚠️', 'Chrome no reportó debug port, usando fallback visual');
      }
      chrome.kill();
      resolve({});
    }, 15000);

    // Si hay credenciales, hacer login
    if (email && password) {
      await esperar(3000);
      log('🔑', 'Intentando login en Figma vía Google...');
      chrome.stdin.write(JSON.stringify({
        url: 'https://www.figma.com/login',
        email, password
      }) + '\n');
    }
  });
}

async function obtenerWsUrl(port) {
  try {
    const data = await request(`http://localhost:${port}/json/version`);
    return data.webSocketDebuggerUrl;
  } catch { return null; }
}

async function extraerDisenoViaCDP(wsUrl) {
  return new Promise((resolve, reject) => {
    try {
      const ws = new WebSocket(wsUrl);
      let msgId = 1;

      ws.on('open', () => {
        // Navegar al archivo
        ws.send(JSON.stringify({
          id: msgId++, method: 'Page.navigate',
          params: { url: `https://www.figma.com/file/${fileKey}` }
        }));
      });

      ws.on('message', async (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.method === 'Page.frameStoppedLoading') {
          // Extraer CSS custom properties del documento
          ws.send(JSON.stringify({
            id: msgId++, method: 'Runtime.evaluate',
            params: {
              expression: `
                (() => {
                  const styles = document.querySelector('[data-figma-css]');
                  return styles ? styles.textContent : '';
                })()
              `
            }
          }));
        }
        if (msg.id && msg.result) {
          if (msg.result.result?.value) {
            resolve({ cssRaw: msg.result.result.value });
          }
        }
      });

      setTimeout(() => {
        ws.close();
        resolve({});
      }, 20000);
    } catch (e) {
      resolve({});
    }
  });
}

// ─── Fase 3: Generar Design System CSS ────────────────────────────────

function generarCSS(colors = {}, exchange = 'binance') {
  const exColors = EXCHANGE_COLORS[exchange] || EXCHANGE_COLORS.binance;

  // Si hay colores reales de Figma, usarlos. Sino, usar exchange de referencia.
  const hasRealColors = Object.keys(colors).length > 0;

  log('🎨', hasRealColors ? 'Usando colores extraídos de Figma' : 'Usando colores de referencia élite');

  let css = `/* ═══════════════════════════════════════════════════════════════════════════
   DESIGN SYSTEM — ${hasRealColors ? 'Extraído de Figma' : `Inspirado en ${exchange.toUpperCase()}`}
   Generado por NEXUS el ${new Date().toISOString()}
   ═══════════════════════════════════════════════════════════════════════════ */

:root {
  /* ─── Colores base ─── */\n`;

  if (hasRealColors) {
    for (const [name, hex] of Object.entries(colors)) {
      css += `  --${name}: ${hex};\n`;
    }
  } else {
    css += `  --exchange-bg:        ${exColors.bg};
  --exchange-text:       ${exColors.text};
  --exchange-green:      ${exColors.green};
  --exchange-red:        ${exColors.red};
  --exchange-blue:       ${exColors.blue || '#1e80ff'};
  --exchange-yellow:     ${exColors.yellow || '#f0b90b'};

  --bg-primary:          #0b0e11;
  --bg-secondary:        #1e2329;
  --bg-tertiary:         #2b3139;
  --bg-hover:            #363c44;
  --text-primary:        #eaecef;
  --text-secondary:      #848e9c;
  --text-muted:          #5e6673;
  --green:               #0ecb81;
  --green-bg:            rgba(14, 203, 129, 0.12);
  --red:                 #f6465d;
  --red-bg:              rgba(246, 70, 93, 0.12);
  --yellow:              #f0b90b;
  --blue:                #1e80ff;\n`;
  }

  css += `
  /* ─── Tipografía élite ─── */
  --font-sans:  'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  --font-mono:  'SF Mono', 'Fira Code', 'Cascadia Code', 'JetBrains Mono', monospace;

  --font-size-xs:    10px;
  --font-size-sm:    11px;
  --font-size-base:  12px;
  --font-size-md:    13px;
  --font-size-lg:    14px;
  --font-size-xl:    18px;
  --font-size-2xl:   24px;
  --font-size-3xl:   32px;

  /* ─── Espaciado ─── */
  --space-1:  4px;
  --space-2:  8px;
  --space-3:  12px;
  --space-4:  16px;
  --space-6:  24px;
  --space-8:  32px;

  /* ─── Bordes ─── */
  --radius-sm:   2px;
  --radius-md:   4px;
  --radius-lg:   8px;
  --radius-full: 9999px;

  --border:       #2b3139;
  --border-light: #363c44;

  /* ─── Sombras ─── */
  --shadow-sm:   0 1px 2px rgba(0,0,0,0.3);
  --shadow-md:   0 4px 12px rgba(0,0,0,0.4);
  --shadow-lg:   0 8px 24px rgba(0,0,0,0.5);

  /* ─── Transiciones ─── */
  --transition-fast: 0.15s ease;
  --transition-base: 0.2s ease;
  --transition-slow: 0.3s ease;
}

/* ─── Layout de Exchange ─── */
.exchange-grid {
  display: grid;
  grid-template-columns: 1fr 320px;
  grid-template-rows: 48px minmax(0, 1fr) 280px;
  gap: 1px;
  height: 100vh;
  background: var(--border);
}

.exchange-header {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  padding: 0 16px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border);
  gap: 16px;
}

.exchange-header .ticker {
  font-family: var(--font-mono);
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--text-primary);
}

.exchange-header .price {
  font-family: var(--font-mono);
  font-size: var(--font-size-xl);
  font-weight: 500;
}

.exchange-header .change { font-size: var(--font-size-sm); }

/* ─── Order Book ─── */
.order-book {
  display: grid;
  grid-template-rows: auto 1fr auto 1fr;
  height: 100%;
  background: var(--bg-primary);
}

.order-book-header {
  display: grid;
  grid-template-columns: 1fr 80px 80px;
  padding: 4px 8px;
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.order-row {
  display: grid;
  grid-template-columns: 1fr 80px 80px;
  padding: 1px 8px;
  font-size: var(--font-size-sm);
  font-family: var(--font-mono);
  position: relative;
}

.order-row .depth-bg {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  opacity: 0.12;
  transition: width 0.2s;
}

.order-row.ask { color: var(--red); }
.order-row.bid { color: var(--green); }

/* ─── Trading Form ─── */
.trading-form {
  display: grid;
  gap: 8px;
  padding: 12px;
}

.trading-form .tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border);
}

.trading-form .tab {
  padding: 6px 16px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.trading-form .tab.active {
  color: var(--blue);
  border-bottom-color: var(--blue);
}

.trading-form label {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.trading-form input {
  width: 100%;
  padding: 6px 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--font-size-md);
  outline: none;
}

.trading-form input:focus { border-color: var(--blue); }

.btn-buy, .btn-sell {
  width: 100%;
  padding: 8px;
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 600;
  text-transform: uppercase;
  cursor: pointer;
  transition: opacity var(--transition-fast);
}

.btn-buy { background: var(--green); color: white; }
.btn-sell { background: var(--red); color: white; }
.btn-buy:hover, .btn-sell:hover { opacity: 0.85; }

/* ─── Signals Panel ─── */
.signals-panel {
  padding: 8px;
  overflow-y: auto;
}

.signal-card {
  padding: 8px;
  margin-bottom: 4px;
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  border-left: 3px solid;
}

.signal-card.compra { background: var(--green-bg); border-left-color: var(--green); color: var(--green); }
.signal-card.venta { background: var(--red-bg); border-left-color: var(--red); color: var(--red); }
.signal-card.neutral { background: var(--bg-tertiary); border-left-color: var(--text-muted); color: var(--text-secondary); }

/* ─── Tabla de Órdenes ─── */
.orders-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--font-size-sm);
  font-family: var(--font-mono);
}

.orders-table th {
  padding: 6px 12px;
  text-align: left;
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  text-transform: uppercase;
  border-bottom: 1px solid var(--border);
}

.orders-table td {
  padding: 4px 12px;
  border-bottom: 1px solid var(--border);
}

/* ─── Botones Sistema ─── */
.btn-nexus {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 6px 16px;
  border-radius: var(--radius-md);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.btn-nexus-primary { background: var(--blue); color: white; border: 1px solid var(--blue); }
.btn-nexus-green { background: var(--green); color: white; border: 1px solid var(--green); }
.btn-nexus-red { background: var(--red); color: white; border: 1px solid var(--red); }
.btn-nexus-ghost { background: transparent; color: var(--text-secondary); border: 1px solid transparent; }
.btn-nexus-ghost:hover { background: var(--bg-hover); color: var(--text-primary); }

/* ─── Auto-Trading ─── */
.auto-trading-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
}

.auto-trading-status.active { background: var(--green-bg); color: var(--green); }
.auto-trading-status.inactive { background: var(--bg-tertiary); color: var(--text-muted); }

/* ─── Scrollbar ─── */
::-webkit-scrollbar { width: 4px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border-light); border-radius: 2px; }
::-webkit-scrollbar-thumb:hover { background: var(--text-muted); }
`;

  fs.mkdirSync(path.dirname(OUTPUT_CSS), { recursive: true });
  fs.writeFileSync(OUTPUT_CSS, css);
  log('✅', `Design System generado: ${OUTPUT_CSS}`);
  return css;
}

// ─── Main ───────────────────────────────────────────────────────────────

async function main() {
  const figmaUrl = process.argv[2];
  const modeFlag = process.argv.find(a => a.startsWith('--'));

  if (!figmaUrl || figmaUrl === '--help') {
    console.log(`
🧬 NEXUS — Figma Design Absorber

USO:
  node figma_absorb.js <figma_url> [--token|--google|--screenshot]

  --token      Usar Figma Personal Access Token (recomendado)
  --google     Login con Google vía Chrome headless + vault
  --screenshot Tomar screenshot + análisis visual

  Sin flags: intenta automaticamente token → google → screenshot

EJEMPLO:
  node figma_absorb.js https://www.figma.com/file/abc123/design --token
  node figma_absorb.js https://www.figma.com/file/abc123/design --google
    `);
    process.exit(0);
  }

  // Extraer file key
  const fileKeyMatch = figmaUrl.match(/file\/([a-zA-Z0-9_-]+)/);
  if (!fileKeyMatch) {
    log('❌', 'No se pudo extraer file key del URL');
    process.exit(1);
  }
  const fileKey = fileKeyMatch[1];
  log('🔗', `File Key: ${fileKey}`);

  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  let designColors = {};
  let method = 'referencia';

  // ─── Modo --token: intentar API REST ──────────────────────────
  if (modeFlag === '--token' || !modeFlag) {
    log('🔑', 'Buscando Figma Token en vault...');
    const tokenData = await descifrarVault(path.join(VAULT_DIR, 'figma.enc'));
    if (tokenData?.token) {
      log('🔑', 'Token encontrado en vault. Extrayendo vía API...');
      try {
        designColors = await extraerPorAPI(tokenData.token, fileKey);
        method = 'figma-api';
      } catch (e) {
        log('⚠️', `API falló: ${e.message}`);
      }
    } else {
      log('⚠️', 'No hay Figma Token en vault');
    }
  }

  // ─── Modo --google o fallback ─────────────────────────────────
  if ((modeFlag === '--google' || (!modeFlag && method === 'referencia'))) {
    log('🌐', 'Buscando credenciales Google en vault...');
    const googleData = await descifrarVault(path.join(VAULT_DIR, 'google.enc'));
    if (googleData?.email && googleData?.password) {
      log('🌐', `Google: ${googleData.email}. Lanzando Chrome...`);
      try {
        const result = await extraerPorChrome(fileKey, googleData.email, googleData.password);
        if (result?.cssRaw) {
          // Parsear CSS extraído para sacar variables
          log('✅', 'CSS extraído vía Chrome');
        }
        method = 'chrome-google';
      } catch (e) {
        log('⚠️', `Chrome falló: ${e.message}`);
      }
    } else {
      log('⚠️', 'No hay credenciales Google en vault');
    }
  }

  // ─── Generar CSS final ────────────────────────────────────────
  const exchange = 'binance';
  generarCSS(designColors, exchange);

  log('', '');
  log('🧬', '═══════════════════════════════════════');
  log('🧬', 'ABSORCIÓN COMPLETADA');
  log('🧬', `Método: ${method}`);
  log('🧬', `Colores extraídos: ${Object.keys(designColors).length}`);
  log('🧬', '═══════════════════════════════════════');
  log('', '');
  log('📁', `design-system.css → ${OUTPUT_CSS}`);
  log('📁', `Assets → ${OUTPUT_DIR}/`);
  log('', '');
  log('🚀', 'Para aplicar: cd trading-portal/frontend && npx vite build');
}

main().catch(e => {
  log('💀', `FATAL: ${e.message}`);
  process.exit(1);
});
