#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════╗
 * ║  🩺 CIA FOIA — DIAGNÓSTICO V3 (RÁPIDO Y DIRECTO)    ║
 * ║  Usa waitUntil:'load' y sin Tor                      ║
 * ╚══════════════════════════════════════════════════════╝
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';

async function timeout(ms) { return new Promise(r => setTimeout(r, ms)); }

async function diagnose() {
  console.log('='.repeat(70));
  console.log('  🩺 CIA FOIA DIAGNÓSTICO V3');
  console.log('='.repeat(70));

  const browser = await puppeteer.launch({
    headless: true,
    executablePath: CHROME_PATH,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
  });

  console.log('\n[1] 🔍 Resultados "operation condor"...');
  const page = await browser.newPage();
  await page.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36');
  await page.goto('https://www.cia.gov/readingroom/search/site/operation%20condor', {
    waitUntil: 'load', timeout: 30000
  });
  await timeout(3000); // extra wait for JS rendering

  // Snapshot the page structure
  const info = await page.evaluate(() => ({
    url: window.location.href,
    title: document.title,
    bodyLen: document.body.innerText.length,
    docLinks: Array.from(document.querySelectorAll('a')).filter(a => a.href.includes('/document/')).map(a => ({ text: a.innerText.trim().slice(0, 100), href: a.href })),
    searchCount: (document.body.innerText.match(/Search found (\d+) items/) || [])[1] || 'unknown',
    selectors: ['ol', 'ul', '.views-row', '.search-result', '.node', 'article', 'li a[href*="/document/"]']
      .filter(sel => document.querySelector(sel))
  }));
  console.log(`     └─ URL: ${info.url}`);
  console.log(`     └─ Título: ${info.title}`);
  console.log(`     └─ Cuerpo: ${info.bodyLen} chars`);
  console.log(`     └─ Search count: ${info.searchCount}`);
  console.log(`     └─ Doc links: ${info.docLinks.length}`);
  console.log(`     └─ Selectores encontrados: ${info.selectors.join(', ')}`);
  info.docLinks.slice(0, 8).forEach((d, i) => console.log(`       ${i+1}. ${d.text.slice(0, 70)}`));

  if (info.docLinks.length > 0) {
    const docUrl = info.docLinks[0].href;
    console.log(`\n[2] 📄 Primer documento: ${docUrl}`);
    await page.goto(docUrl, { waitUntil: 'load', timeout: 30000 });
    await timeout(3000);

    const docInfo = await page.evaluate(() => {
      const allLinks = Array.from(document.querySelectorAll('a')).map(a => ({
        text: a.innerText.trim().slice(0, 60),
        href: a.href.slice(0, 180),
        cls: (a.className || '').slice(0, 40)
      }));

      const pdfLinks = allLinks.filter(l =>
        l.href.match(/\.pdf$/i) || l.href.match(/\/download\b/) || l.href.match(/\/media\//)
      );

      const downloadBtns = allLinks.filter(l =>
        l.text.match(/pdf|download/i) || l.cls.match(/button|download|pdf/i)
      );

      return {
        title: document.title,
        bodyLen: document.body.innerText.length,
        totalLinks: allLinks.length,
        pdfLinks,
        downloadBtns,
        htmlSample: document.body.innerHTML.slice(0, 3000)
      };
    });

    console.log(`     └─ Título: ${docInfo.title.slice(0, 80)}`);
    console.log(`     └─ Cuerpo: ${docInfo.bodyLen} chars`);
    console.log(`     └─ PDF links: ${docInfo.pdfLinks.length}`);
    docInfo.pdfLinks.forEach((p, i) => console.log(`       ${i+1}. "${p.text}" → ${p.href}`));
    console.log(`     └─ Download buttons: ${docInfo.downloadBtns.length}`);
    docInfo.downloadBtns.forEach((b, i) => console.log(`       ${i+1}. "${b.text}" → ${b.href}`));
    
    // Guardar HTML de la página del documento
    fs.writeFileSync('/tmp/cia_doc_page.html', docInfo.htmlSample);
    console.log(`\n     HTML guardado en /tmp/cia_doc_page.html (3000 chars)`);
  }

  await browser.close();
  console.log('\n✅ Diagnóstico completado');
}

diagnose().catch(console.error);
