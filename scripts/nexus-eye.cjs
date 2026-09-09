const puppeteer = require('puppeteer');
const path = require('path');

(async () => {
    console.log("🔍 [NEXUS EYE] Iniciando inspección de consola...");
    const browser = await puppeteer.launch({
        headless: "new",
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    const page = await browser.newPage();
    
    // REDIRECCIÓN DE CONSOLA A STDOUT
    page.on('console', msg => {
        console.log(`🖥️ [CONSOLE.${msg.type().toUpperCase()}] ${msg.text()}`);
    });

    page.on('pageerror', err => {
        console.log(`❌ [PAGE_ERROR] ${err.toString()}`);
    });

    const htmlPath = 'file://' + path.resolve(__dirname, '../nexus-ghost-shell/ui/index.html');
    await page.goto(htmlPath, { waitUntil: 'networkidle0' });

    // Intentar disparar el evento del Orbe
    console.log("🖱️ Intentando click en Orbe...");
    await page.click('#nexus-orb');

    // Esperar un poco para ver reacciones
    await new Promise(r => setTimeout(r, 2000));

    await browser.close();
    console.log("🔍 [NEXUS EYE] Inspección terminada.");
})();
