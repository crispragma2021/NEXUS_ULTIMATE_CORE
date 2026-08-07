#!/usr/bin/env node
/**
 * ═══ CIA FOIA — TEST CON TOR (como engine exitoso) ═══
 */
import puppeteer from 'puppeteer';
import fs from 'fs';

const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';
const PROXY = 'socks5://127.0.0.1:9050';

async function main() {
  // VERIFICAR TOR ACTIVO
  try {
    const response = await fetch('http://httpbin.org/ip');
    const data = await response.json();
    console.log(`[🔍] IP actual: ${data.origin}`);
  } catch(e) {
    console.log(`[🔍] No se pudo verificar IP`);
  }

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
      `--proxy-server=${PROXY}`,
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

  console.log('\n[1] Navegando a búsqueda (vía Tor)...');
  const response = await page.goto('https://www.cia.gov/readingroom/search/site/operation%20condor', {
    waitUntil: 'networkidle0',
    timeout: 120000,
  });
  console.log(`  Final URL: ${page.url()}`);
  console.log(`  Status: ${response?.status()}`);

  await new Promise(r => setTimeout(r, 5000));

  const info = await page.evaluate(() => {
    const text = document.body.innerText;
    const foundMatch = text.match(/Search found (\d+) items/);
    
    // Encontrar TODOS los enlaces
    const allLinks = Array.from(document.querySelectorAll('a')).map(a => ({
      text: a.innerText.trim().slice(0, 60),
      href: a.href.slice(0, 150),
    }));

    // Buscar específicamente /document/ en href
    const docLinks = allLinks.filter(l => l.href.includes('/document/'));
    
    // Buscar .pdf en href
    const pdfLinks = allLinks.filter(l => l.href.includes('.pdf'));

    return {
      url: window.location.href,
      bodyLength: text.length,
      hasSearchFound: foundMatch !== null,
      searchCount: foundMatch ? foundMatch[1] : null,
      totalLinks: allLinks.length,
      docLinks,
      pdfLinks,
      textSample: text.slice(500, 1500),
      // Extraer lo que parece un documento (líneas con uppercase seguidas de RDP)
      docMatches: text.match(/[A-Z][A-Z\s\-.,:;'"]{10,100}\n/g)?.slice(0, 10) || [],
    };
  });

  console.log(`\n  URL final: ${info.url}`);
  console.log(`  Body: ${info.bodyLength} chars`);
  console.log(`  Search found: ${info.hasSearchFound} (count: ${info.searchCount})`);
  console.log(`  Total links: ${info.totalLinks}`);
  console.log(`  Doc links (/document/): ${info.docLinks.length}`);
  console.log(`  PDF links: ${info.pdfLinks.length}`);
  
  info.docLinks.forEach(d => console.log(`    "${d.text}" → ${d.href}`));
  info.pdfLinks.forEach(d => console.log(`    "${d.text}" → ${d.href}`));

  console.log(`\n  Text sample (500-1500):`);
  console.log(info.textSample);

  console.log(`\n  Document title matches:`);
  info.docMatches.forEach((m, i) => console.log(`  ${i+1}. ${m.slice(0, 80)}`));

  // CAPTURAR HTML
  fs.writeFileSync('/tmp/cia_search_tor.html', await page.content());
  console.log('\nHTML guardado en /tmp/cia_search_tor.html');

  await browser.close();
}

main().catch(e => { console.error('ERROR:', e.message); process.exit(1); });
