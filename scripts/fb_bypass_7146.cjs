const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

(async () => {
  const targetUrl = process.argv[2]; // URL to scrape
  const outputFilename = process.argv[3]; // Output filename

  const useTor = process.argv.includes('--tor');
  const proxyArgs = useTor ? ['--proxy-server=socks5://127.0.0.1:9050'] : [];

  if (!targetUrl || !outputFilename) {
    console.error('Uso: node fb_bypass_7146.cjs <url> <output_filename> [--tor]');
    process.exit(1);
  }

  const browser = await puppeteer.launch({
    headless: "new",
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu', '--disable-dev-shm-usage', ...proxyArgs]
  });
  const page = await browser.newPage();
  
  // Establecer un User-Agent humano de élite
  await page.setUserAgent('Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36');
  await page.setViewport({ width: 1280, height: 800 });

  const newTargetUrl = 'https://www.facebook.com/marialuisa.villalba.7146'; // Using a fixed URL for this script, as it's specific to 7146
  console.log(`🚀 Navegando al perfil: ${newTargetUrl}`);
  
  try {
    await page.goto(newTargetUrl, { waitUntil: 'networkidle2', timeout: 30000 });
  } catch (e) {
    console.log(`⏱ Timeout de navegación (normal con modales): ${e.message}`);
  }

  console.log("🧹 Ejecutando Bypass de Modales y Limpieza de DOM...");
  await page.evaluate(() => {
    // 1. Eliminar modales de login/registro
    const selectors = [
      'div[role="dialog"]', 
      'div[id^="login_"]', 
      'div[class*="login"]',
      'div[aria-label*="Log In"]',
      'div[aria-label*="Iniciar sesión"]',
      'div[aria-label*="Close"]',
      'div[aria-label*="Cerrar"]'
    ];
    
    selectors.forEach(s => {
      document.querySelectorAll(s).forEach(el => el.remove());
    });

    // 2. Restaurar el scroll del cuerpo (Facebook lo bloquea cuando sale el modal)
    document.body.style.overflow = 'visible';
    document.documentElement.style.overflow = 'visible';
  });

  // Esperar un momento para que el layout se estabilice
  await new Promise(r => setTimeout(r, 3000));

  console.log("📸 Disparando Visión Omega...");
  await page.screenshot({ 
    path: '/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/fb_profile_7146.png', 
    fullPage: true 
  });

  // Extraer texto visible del perfil
  const profileText = await page.evaluate(() => {
    // Intentar obtener el título de la página
    const title = document.title;
    
    // Buscar el nombre del perfil en meta tags
    const metaDesc = document.querySelector('meta[property="og:title"]')?.content || '';
    const metaImage = document.querySelector('meta[property="og:image"]')?.content || '';
    const metaUrl = document.querySelector('meta[property="og:url"]')?.content || '';
    
    // Extraer todo el texto visible del body
    const bodyText = document.body?.innerText || '';
    
    // Buscar elementos específicos del perfil
    const nameElement = document.querySelector('h1')?.innerText || '';
    const bioElement = document.querySelector('[data-pagelet="ProfileTabs"]')?.innerText || '';
    
    return {
      title,
      metaDesc,
      metaImage,
      metaUrl,
      nameElement,
      bioElement,
      bodyText: bodyText.substring(0, 5000) // Limitar a 5000 chars
    };
  });

  console.log("\n📋 DATOS EXTRAÍDOS DEL PERFIL:");
  console.log("=".repeat(60));
  console.log(`Título: ${profileText.title}`);
  console.log(`Meta Description: ${profileText.metaDesc}`);
  console.log(`Meta Image: ${profileText.metaImage}`);
  console.log(`Meta URL: ${profileText.metaUrl}`);
  console.log(`Nombre (h1): ${profileText.nameElement}`);
  console.log(`Bio: ${profileText.bioElement}`);
  console.log("\n📄 TEXTO DEL PERFIL (primeros 5000 chars):");
  console.log(profileText.bodyText);

  // Guardar el texto extraído a un archivo para análisis
  const fs = require('fs');
  fs.writeFileSync(
    '/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/fb_profile_7146_text.txt',
    JSON.stringify(profileText, null, 2)
  );
  console.log("\n✅ Datos guardados en artifacts/fb_profile_7146_text.txt");
  
  await browser.close();
  console.log("✅ Misión completa.");
})();
