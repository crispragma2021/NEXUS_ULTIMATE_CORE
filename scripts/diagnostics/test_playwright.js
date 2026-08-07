import { chromium } from 'playwright';
import fs from 'fs';

async function test() {
  console.log('🚀 Iniciando navegador Chromium con Playwright...');
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();

  const url = 'http://localhost:5173/';
  console.log(`🌐 Navegando a ${url}...`);
  
  try {
    await page.goto(url, { waitUntil: 'networkidle', timeout: 10000 });
    console.log('✅ Página cargada con éxito.');
    
    const title = await page.title();
    console.log(`📌 Título de la página: "${title}"`);
    
    const screenshotPath = '/tmp/nexus_vite_screenshot.png';
    console.log(`📸 Tomando captura de pantalla en ${screenshotPath}...`);
    await page.screenshot({ path: screenshotPath, fullPage: true });
    
    if (fs.existsSync(screenshotPath)) {
      console.log('🎉 ¡Captura de pantalla guardada correctamente! Playwright funciona al 100%.');
    } else {
      console.error('❌ Error: El archivo de captura de pantalla no se creó.');
    }
  } catch (error) {
    console.error('❌ Error durante la navegación:', error.message);
  } finally {
    await browser.close();
    console.log('🔒 Navegador cerrado.');
  }
}

test();
