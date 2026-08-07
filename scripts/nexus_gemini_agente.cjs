const { chromium } = require('playwright');
const path = require('path');

async function launchAgente() {
    console.log("[NEXUS] Iniciando Agente de Explotación Agéntica...");
    
    // Identidad Chromebook Plus (CrOS 14541.0.0)
    const CHROMEOS_UA = "Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    
    const browser = await chromium.launch({
        headless: false, // Queremos que el Arquitecto vea la gloria de NEXUS
        args: [
            '--disable-blink-features=AutomationControlled',
            '--no-sandbox',
            '--window-size=1280,720'
        ]
    });

    const context = await browser.newContext({
        userAgent: CHROMEOS_UA,
        viewport: { width: 1280, height: 720 },
        deviceScaleFactor: 1,
        isMobile: false,
        hasTouch: false,
        locale: 'en-US',
        timezoneId: 'America/New_York'
    });

    // Inyectar spoofing profundo antes de cargar cualquier página
    await context.addInitScript(() => {
        Object.defineProperty(navigator, 'platform', { get: () => 'X11; CrOS x86_64' });
        Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 8 });
        Object.defineProperty(navigator, 'deviceMemory', { get: () => 8 });
        
        // Bypass de detección de Playwright/Selenium
        delete navigator.__proto__.webdriver;
    });

    const page = await context.newPage();
    
    try {
        console.log("[NEXUS] Navegando al portal de beneficios...");
        await page.goto('https://www.google.com/chromebook/perks/', { waitUntil: 'networkidle' });

        console.log("[NEXUS] Buscando oferta de Gemini Advanced...");
        
        // Esperar a que el sistema de Google valide el hardware inyectado
        await page.waitForTimeout(3000);

        // Intentar forzar la aparición del botón mediante scroll humano
        await page.mouse.wheel(0, 1000);
        await page.waitForTimeout(2000);

        // Localizar el botón de Gemini (usando selectores múltiples para robustez)
        const geminiButton = await page.$('text="Get Gemini Advanced"');
        if (geminiButton) {
            console.log("[NEXUS] ¡Oferta detectada! Resaltando y procediendo...");
            await geminiButton.evaluate(el => el.style.border = '5px solid #00ff00');
            await page.screenshot({ path: '/tmp/nexus_perk_found.png' });
            
            // Aquí el sistema esperará a que el usuario introduzca su cuenta 
            // o NEXUS continuará si tiene credenciales en memoria.
            console.log("[NEXUS] Esperando intervención para login o activación de tarjeta...");
        } else {
            console.warn("[NEXUS] El botón no apareció automáticamente. Intentando bypass de ruta...");
            await page.goto('https://one.google.com/offers?utm_source=chromebook&utm_medium=perks&utm_campaign=gemini_advanced');
        }

    } catch (error) {
        console.error("[NEXUS] Error en la misión:", error);
    }

    // Mantener abierto para que el Arquitecto vea el resultado
    // await browser.close();
}

launchAgente();
