const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

(async () => {
    console.log("🚀 INICIANDO VALIDACIÓN DE NEXUS UI...");
    const browser = await puppeteer.launch({
        headless: "new",
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    const page = await browser.newPage();
    
    const htmlPath = 'file://' + path.resolve(__dirname, '../nexus-ghost-shell/ui/index.html');
    console.log(`📂 Cargando: ${htmlPath}`);
    
    await page.goto(htmlPath, { waitUntil: 'networkidle0' });

    // 1. Verificar existencia del Orbe
    const orb = await page.$('#nexus-orb');
    console.log(`👁️ Orbe detectado: ${orb ? 'SÍ' : 'NO'}`);

    // 2. Verificar Ace Editor
    const editorExists = await page.evaluate(() => {
        return typeof ace !== 'undefined' && document.getElementById('editor') !== null;
    });
    console.log(`💻 Ace Editor inicializado: ${editorExists ? 'SÍ' : 'NO'}`);

    // 3. Captura de pantalla interna (MI VISIÓN)
    if (!fs.existsSync('./artifacts')) fs.mkdirSync('./artifacts');
    await page.screenshot({ path: './artifacts/nexus_vision_test.png' });
    console.log("📸 Screenshot guardado en ./artifacts/nexus_vision_test.png");

    await browser.close();
    console.log("🏁 VALIDACIÓN FINALIZADA.");
})();
