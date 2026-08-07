const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

(async () => {
  const targetUrl = process.argv[2]; // URL to scrape
  const outputFilename = process.argv[3]; // Output filename

  if (!targetUrl || !outputFilename) {
    console.error('Uso: node tbuscolatam_scraper.cjs <url> <output_filename>');
    process.exit(1);
  }

  const browser = await puppeteer.launch({
    headless: "new",
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu']
  });
  const page = await browser.newPage();
  
  // Establecer un User-Agent humano de élite
  await page.setUserAgent('Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36');

  console.log(`🚀 Navegando a: ${targetUrl}`);
  await page.goto(targetUrl, { waitUntil: 'networkidle2' });

  console.log("🧹 Intentando bypass de modales y limpieza de DOM...");
  await page.evaluate(() => {
    // Intentar cerrar popups/modales genéricos
    const selectors = [
      'div[role="dialog"]', 
      'div[id*="modal"]', 
      'div[class*="popup"]',
      'div[aria-modal="true"]',
      // Específicos para tbuscolatam, si se identifican
    ];
    selectors.forEach(s => {
      document.querySelectorAll(s).forEach(el => el.remove());
    });
    // Desactivar scroll si un modal lo bloquea
    document.body.style.overflow = 'auto';
  });

  // Extraer información relevante
  const data = await page.evaluate(() => {
    const extractText = (selector) => {
      const el = document.querySelector(selector);
      return el ? el.innerText.trim() : null;
    };

    const extractAllText = () => {
      return document.body.innerText;
    };

    // Intentar extraer datos estructurados si hay tablas o campos claros
    const name = extractText('h1');
    const generalInfo = extractText('.profile-header'); // O un selector más específico
    const judicialRecords = extractText('.judicial-records'); // O un selector más específico

    return {
      url: window.location.href,
      title: document.title,
      name: name,
      generalInfo: generalInfo,
      judicialRecords: judicialRecords,
      fullText: extractAllText(),
    };
  });

  // Guardar la información extraída
  const outputPath = path.join('artifacts', outputFilename);
  fs.writeFileSync(outputPath, JSON.stringify(data, null, 2));
  console.log(`✅ Información guardada en: ${outputPath}`);

  // Opcional: tomar captura de pantalla
  const screenshotPath = outputPath.replace('.json', '.png');
  await page.screenshot({ path: screenshotPath, fullPage: true });
  console.log(`📸 Captura de pantalla guardada en: ${screenshotPath}`);

  await browser.close();
})();
