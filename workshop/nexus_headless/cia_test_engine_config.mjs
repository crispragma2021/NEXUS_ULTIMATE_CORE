#!/usr/bin/env node
/**
 * ═══ CIA FOIA — TEST CONFIG EXACTA DEL ENGINE ═══
 * Usa Todos los flags que el engine exitoso usó
 */
import puppeteer from 'puppeteer';
import fs from 'fs';

const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';

async function main() {
  const browser = await puppeteer.launch({
    headless: true,
    executablePath: CHROME_PATH,
    args: [
      '--no-sandbox', '--disable-setuid-sandbox',
      '--disable-dev-shm-usage', '--disable-gpu',
      '--disable-web-security',
      '--disable-features=IsolateOrigins,site-per-process',
      '--disable-blink-features=AutomationControlled',
      '--window-size=1280,1024',
    ],
  });

  const page = await browser.newPage();
  await page.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36');
  await page.setViewport({ width: 1280, height: 1024 });
  await page.evaluateOnNewDocument(() => {
    Object.defineProperty(navigator, 'webdriver', { get: () => false });
    Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
    Object.defineProperty(navigator, 'languages', { get: () => ['es-ES', 'es', 'en'] });
  });

  // Monitorear navegación
  page.on('request', req => {
    if (req.url().includes('search/site'))
      console.log(`  ➡️  Request: ${req.url().slice(0, 100)} [${req.method()}]`);
  });
  page.on('response', res => {
    if (res.url().includes('/readingroom'))
      console.log(`  ⬅️  Response: ${res.url().slice(0, 100)} [${res.status()}]`);
  });

  // Navegar a búsqueda
  console.log('[1] Navegando a búsqueda...');
  const response = await page.goto('https://www.cia.gov/readingroom/search/site/operation%20condor', {
    waitUntil: 'networkidle0',
    timeout: 60000,
  });
  console.log(`  Final URL: ${page.url()}`);
  console.log(`  Status: ${response?.status()}`);
  console.log(`  Body length: ${(await page.content()).length}`);

  // Esperar renderizado SPA
  console.log('\n[2] Esperando render SPA (5s)...');
  await new Promise(r => setTimeout(r, 5000));

  console.log(`  Final URL after wait: ${page.url()}`);
  console.log(`  Body: ${(await page.evaluate(() => document.body.innerText.length))} chars`);

  // Buscar "Search found" en el texto
  const hasResults = await page.evaluate(() => {
    const text = document.body.innerText;
    const foundMatch = text.match(/Search found (\d+) items/);
    return {
      hasSearchFound: foundMatch !== null,
      count: foundMatch ? foundMatch[1] : null,
      hasDocText: text.includes('OPERATION CONDOR') || text.includes('Condor'),
      sample: text.slice(1000, 2000),
    };
  });
  console.log(`\n  Has "Search found": ${hasResults.hasSearchFound} (count: ${hasResults.count})`);
  console.log(`  Has doc text: ${hasResults.hasDocText}`);
  console.log(`\n  Sample text (1000-2000):`);
  console.log(hasResults.sample);

  // Buscar enlaces a documentos
  const docLinks = await page.evaluate(() => {
    const links = [];
    document.querySelectorAll('a').forEach(a => {
      if (a.href.includes('/document/') || a.href.includes('.pdf')) {
        links.push({
          text: a.innerText.trim().slice(0, 80),
          href: a.href.slice(0, 150),
        });
      }
    });
    return links;
  });
  console.log(`\nDoc/PDF links found: ${docLinks.length}`);
  docLinks.forEach(d => console.log(`  "${d.text}" → ${d.href}`));

  // Capturar HTML completo
  fs.writeFileSync('/tmp/cia_search_full.html', await page.content());
  console.log('\nHTML guardado en /tmp/cia_search_full.html');

  await browser.close();
}

main().catch(e => { console.error('ERROR:', e.message); process.exit(1); });
