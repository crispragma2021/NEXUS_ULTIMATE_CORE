const { chromium } = require('playwright');
const fs = require('fs');

async function main() {
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    await page.setViewportSize({ width: 1920, height: 1080 });
    
    console.log('📸 Capturando HUD en http://localhost:5173...');
    try {
        await page.goto('http://localhost:5173', { waitUntil: 'networkidle', timeout: 30000 });
        await page.waitForTimeout(5000); // Esperar a que las gráficas carguen
        await page.screenshot({ path: '/tmp/nexus_trading_vision.png' });
        console.log('✅ Captura guardada en /tmp/nexus_trading_vision.png');
    } catch (e) {
        console.error('❌ Error capturando HUD:', e.message);
    } finally {
        await browser.close();
    }
}

main();
