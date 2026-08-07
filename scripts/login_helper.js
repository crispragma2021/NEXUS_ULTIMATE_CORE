const { chromium } = require('playwright-extra');
const stealth = require('puppeteer-extra-plugin-stealth')();
chromium.use(stealth);

(async () => {
    console.log("🔓 Abriendo navegador en modo CABEZA VISIBLE para inicio de sesión...");
    console.log("👉 Por favor, inicia sesión en Google y espera 5 segundos.");

    const USER_DATA_DIR = '/home/soberano/NEXUS_ULTIMATE_CORE/NEXUS_INTERFACE/nexus_browser_profile';
    const context = await chromium.launchPersistentContext(USER_DATA_DIR, {
        headless: false, // ¡Visible!
        viewport: null
    });

    const page = await context.newPage();
    const targetUrl = process.env.TARGET_URL || 'https://gemini.google.com/app';
    console.log(`🌍 Navegando a: ${targetUrl}`);
    await page.goto(targetUrl);

    console.log("🚦 Esperando a que el usuario cierre el navegador manualmente...");

    // Esperar hasta que se cierre el contexto (usuario cierra ventana)
    await new Promise(resolve => {
        context.on('close', resolve);
        // O mantener vivo indefinidamente hasta Ctrl+C
        setInterval(() => { }, 1000);
    });
})();
