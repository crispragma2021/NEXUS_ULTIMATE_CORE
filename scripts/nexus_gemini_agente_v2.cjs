const { chromium } = require('playwright-extra');
const stealth = require('puppeteer-extra-plugin-stealth')();
chromium.use(stealth);

async function launchAgenteSoberano() {
    console.log("[NEXUS] Lanzando Agente Stealth v2.0...");
    
    const browser = await chromium.launch({
        headless: false,
        args: [
            '--disable-blink-features=AutomationControlled',
            '--no-sandbox',
            '--disable-web-security',
            '--disable-features=IsolateOrigins,site-per-process'
        ]
    });

    const context = await browser.newContext({
        userAgent: "Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        viewport: { width: 1366, height: 768 },
        deviceScaleFactor: 1,
        isMobile: false,
        hasTouch: false,
        locale: 'en-US',
        timezoneId: 'America/New_York'
    });

    // Inyección agéntica de hardware
    await context.addInitScript(() => {
        // Mock de hardware real Chromebook Plus
        Object.defineProperty(navigator, 'platform', { get: () => 'X11; CrOS x86_64' });
        Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 8 });
        Object.defineProperty(navigator, 'deviceMemory', { get: () => 8 });
        
        // Mock de GPU
        const getParameterProxy = (original) => function(parameter) {
            if (parameter === 37445) return "Intel Open Source Technology Center";
            if (parameter === 37446) return "Mesa Intel(R) UHD Graphics (ADL GT2)";
            return original.apply(this, arguments);
        };
        
        try {
            const canvas = document.createElement('canvas');
            const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
            const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
            if (debugInfo) {
                gl.getParameter = getParameterProxy(gl.getParameter);
            }
        } catch(e) {}
    });

    const page = await context.newPage();
    
    try {
        console.log("[NEXUS] Paso 1: Autenticación Preventiva...");
        await page.goto('https://accounts.google.com/ServiceLogin?hl=en', { waitUntil: 'networkidle' });
        
        console.log("[NEXUS] PADRE: Introduce tus credenciales en el navegador.");
        console.log("[NEXUS] Cuando estés logueado, yo detectaré la sesión y saltaré a los perks.");

        // Monitorizar cambio de URL tras login exitoso
        await page.waitForURL('**/myaccount.google.com/**', { timeout: 0 });
        
        console.log("[NEXUS] Sesión detectada. Forzando portal de Chromebook Perks...");
        await page.goto('https://www.google.com/chromebook/perks/', { waitUntil: 'networkidle' });

        // Esperar y clickear el beneficio resaltándolo
        await page.waitForTimeout(5000);
        await page.mouse.wheel(0, 1000);
        
        const redeemLink = "https://one.google.com/offers?utm_source=chromebook&utm_medium=perks&utm_campaign=gemini_advanced";
        console.log("[NEXUS] Redirigiendo a zona de canje $0.00...");
        await page.goto(redeemLink, { waitUntil: 'networkidle' });

    } catch (error) {
        console.error("[NEXUS] Error en la misión agéntica:", error);
    }
}

launchAgenteSoberano();
