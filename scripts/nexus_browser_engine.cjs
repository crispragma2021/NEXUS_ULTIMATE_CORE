// ═══════════════════════════════════════════════════════════════════════════
// nexus_browser_engine.cjs — Motor Unificado de Navegación Web Interactiva
// ═══════════════════════════════════════════════════════════════════════════

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const os = require('os');

const accion = process.argv[2] || 'navegar';
const url = process.argv[3] || 'https://google.com';
const param1 = process.argv[4]; // selector / texto / ruta_salida
const param2 = process.argv[5]; // texto a escribir / ruta_salida
const param3 = process.argv[6]; // ruta_salida

(async () => {
  let browser = null;
  try {
    let outputPath = '/tmp/nexus_web_screenshot.png';
    if (param1 && param1.endsWith('.png')) outputPath = param1;
    else if (param2 && param2.endsWith('.png')) outputPath = param2;
    else if (param3 && param3.endsWith('.png')) outputPath = param3;

    fs.mkdirSync(path.dirname(outputPath), { recursive: true });

    const userDataDir = fs.mkdtempSync(path.join(os.tmpdir(), 'nexus-unified-browser-'));
    
    // Usar el binario del sistema de Chrome si existe para máxima compatibilidad
    const chromePath = fs.existsSync('/usr/bin/google-chrome') 
      ? '/usr/bin/google-chrome' 
      : undefined;

    const launchOpts = {
      headless: true,
      viewport: { width: 1280, height: 800 },
      args: ['--no-sandbox', '--disable-dev-shm-usage'],
    };
    if (chromePath) {
      launchOpts.executablePath = chromePath;
    }

    const context = await chromium.launchPersistentContext(userDataDir, launchOpts);
    browser = context;
    const page = context.pages()[0] || await context.newPage();

    console.log(`🌐 [NEXUS-BROWSER] Ejecutando '${accion}' en ${url}...`);

    await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 25000 });
    await page.waitForTimeout(1500);

    if (accion === 'clic' && param1) {
      console.log(`🖱️ [NEXUS-BROWSER] Haciendo clic en: ${param1}`);
      try {
        await page.click(param1, { timeout: 5000 });
      } catch {
        await page.click(`text="${param1}"`, { timeout: 5000 });
      }
      await page.waitForTimeout(2000);
    } else if (accion === 'escribir' && param1 && param2) {
      console.log(`⌨️ [NEXUS-BROWSER] Escribiendo en '${param1}': ${param2}`);
      await page.fill(param1, param2);
      await page.waitForTimeout(1000);
    }

    // Tomar captura
    await page.screenshot({ path: outputPath, fullPage: false });

    // Extraer contenido legible de texto
    const titulo = await page.title();
    const textoPagina = await page.evaluate(() => {
      return document.body ? document.body.innerText.replace(/\s+/g, ' ').slice(0, 3000) : '';
    });

    console.log(`✅ [NEXUS-BROWSER] Éxito. Título: "${titulo}"`);
    console.log(`📸 [NEXUS-BROWSER] Captura guardada: ${outputPath}`);
    console.log(`📄 [NEXUS-BROWSER] Texto (primeros 500 caracteres):\n${textoPagina.slice(0, 500)}`);
  } catch (err) {
    console.error(`❌ [NEXUS-BROWSER] Error: ${err.message}`);
    process.exit(1);
  } finally {
    if (browser) await browser.close().catch(() => {});
  }
})();
