#!/usr/bin/env node
/**
 * ═══ CIA FOIA — TEST CON COOKIES + INTERCEPT ═══
 * Estrategia: visitar home primero, luego search con cookies
 * Interceptar todas las requests para entender el SPA
 */
import puppeteer from 'puppeteer';
import fs from 'fs';

const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';

async function main() {
  const browser = await puppeteer.launch({
    headless: true,
    executablePath: CHROME_PATH,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
  });

  const page = await browser.newPage();
  await page.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36');
  await page.setViewport({ width: 1280, height: 1024 });
  await page.evaluateOnNewDocument(() => {
    Object.defineProperty(navigator, 'webdriver', { get: () => false });
    Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
  });

  // Interceptar TODAS las requests de red
  const networkRequests = [];
  page.on('request', req => {
    networkRequests.push({
      url: req.url().slice(0, 120),
      method: req.method(),
      type: req.resourceType(),
      headers: req.headers(),
    });
  });

  // Paso 1: Home page para cookies
  console.log('[1] Visitando home para obtener cookies...');
  await page.goto('https://www.cia.gov/readingroom/', {
    waitUntil: 'networkidle0', timeout: 30000
  });
  await new Promise(r => setTimeout(r, 2000));
  
  const cookies = await page.cookies();
  console.log(`  Cookies: ${cookies.length}`);
  cookies.forEach(c => console.log(`    ${c.name}=${c.value.slice(0, 30)}`));

  // Paso 2: Navegar a search URL
  console.log('\n[2] Navegando a search/site/operation%20condor...');
  const response = await page.goto('https://www.cia.gov/readingroom/search/site/operation%20condor', {
    waitUntil: 'networkidle0', timeout: 60000
  });
  console.log(`  Status: ${response?.status()}`);
  console.log(`  Final URL: ${page.url()}`);
  
  await new Promise(r => setTimeout(r, 5000));

  // Extraer resultados
  const result = await page.evaluate(() => {
    const text = document.body.innerText;
    return {
      url: window.location.href,
      textLen: text.length,
      hasSearchFound: text.includes('Search found'),
      textSample: text.slice(500, 2000),
    };
  });
  console.log(`\n  Has search results: ${result.hasSearchFound}`);
  console.log(`  Text sample:\n${result.textSample}`);

  // Mostrar requests relevantes
  console.log('\n[3] Network requests relevantes:');
  networkRequests.filter(r => 
    r.url.includes('search') || r.url.includes('solr') || 
    r.url.includes('api') || r.url.includes('json') ||
    r.url.includes('document')
  ).forEach(r => {
    console.log(`  [${r.method}] ${r.type} ${r.url}`);
  });

  // Paso 3: Probar POST directo al formulario de búsqueda
  console.log('\n[4] Probando POST al formulario...');
  const postResult = await page.evaluate(async () => {
    try {
      const formData = new FormData();
      formData.append('search_block_form', 'operation condor');
      formData.append('form_build_id', document.querySelector('input[name="form_build_id"]')?.value || '');
      formData.append('form_id', 'search_block_form');
      formData.append('op', 'Search');
      
      const res = await fetch('/readingroom/', {
        method: 'POST',
        body: formData,
        headers: { 'Accept': 'text/html' }
      });
      const html = await res.text();
      return {
        status: res.status,
        htmlLen: html.length,
        hasResults: html.includes('Search found'),
        sample: html.slice(500, 1500),
      };
    } catch(e) {
      return { error: e.message };
    }
  });
  console.log(`  POST result:`, JSON.stringify(postResult, null, 2).slice(0, 500));

  await browser.close();
  fs.writeFileSync('/tmp/cia_network.json', JSON.stringify(networkRequests, null, 2));
}

main().catch(e => { console.error('ERROR:', e.message); process.exit(1); });
