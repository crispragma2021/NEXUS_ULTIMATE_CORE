// ═══════════════════════════════════════════════════════════════════════════
// take_screenshot.cjs — Vision Bridge (Playwright headless aislado)
// ---------------------------------------------------------------------------
// Contrato consumido por core/src/autodiagnostico/vision_bridge.rs:
//   node take_screenshot.cjs <url> <ruta_salida.png>
//
// - Lanza un Chromium PROPIO (headless) con perfil temporal aislado.
// - NO toca el navegador personal del Arquitecto (Chrome/Firefox/Edge).
// - Usa 'domcontentloaded' (no 'networkidle') porque las apps de trading
//   mantienen WebSockets persistentes que impiden alcanzar el estado idle.
// ═══════════════════════════════════════════════════════════════════════════

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const os = require('os');

const url = process.argv[2] || 'http://localhost:42220';
const outputPath = process.argv[3] || '/tmp/nexus_vite_screenshot.png';

(async () => {
  let browser = null;
  try {
    // Asegurar que el directorio de salida exista
    fs.mkdirSync(path.dirname(outputPath), { recursive: true });

    console.log(`🚀 [take_screenshot] Lanzando Chromium headless aislado para ${url}...`);

    // Perfil temporal propio aislado — nunca el perfil del usuario.
    // launchPersistentContext aplica el userDataDir como perfil dedicado,
    // garantizando cookies/sesión propias y cero contacto con Chrome personal.
    const userDataDir = fs.mkdtempSync(path.join(os.tmpdir(), 'nexus-browser-'));

    const context = await chromium.launchPersistentContext(userDataDir, {
      headless: true,
      viewport: { width: 1440, height: 900 },
      deviceScaleFactor: 1,
      args: ['--no-sandbox', '--disable-dev-shm-usage'],
    });
    browser = context;
    const page = context.pages()[0] || await context.newPage();

    // Esperar el DOM (no networkidle: el feed WS nunca 'idle').
    await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 20000 });

    // Pequeña espera para que rendericen los valores dinámicos (telemetría).
    await page.waitForTimeout(2500);

    // Captura de página completa
    await page.screenshot({ path: outputPath, fullPage: true });

    if (fs.existsSync(outputPath)) {
      const bytes = fs.statSync(outputPath).size;
      console.log(`✅ [take_screenshot] Captura guardada: ${outputPath} (${bytes} bytes)`);
    } else {
      console.error(`❌ [take_screenshot] No se creó el archivo: ${outputPath}`);
      process.exit(1);
    }
  } catch (err) {
    console.error(`❌ [take_screenshot] Error: ${err.message}`);
    process.exit(1);
  } finally {
    if (browser) await browser.close().catch(() => {});
    console.log('🔒 [take_screenshot] Navegador aislado cerrado.');
  }
})();
