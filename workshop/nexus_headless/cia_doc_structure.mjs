#!/usr/bin/env node
/**
 * ═══ CIA FOIA — ESTRUCTURA DE PÁGINA DE DOCUMENTO ═══
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

  // Home → cookies
  await page.goto('https://www.cia.gov/readingroom/', { waitUntil: 'networkidle0', timeout: 30000 });
  await new Promise(r => setTimeout(r, 2000));

  // Ir a UN documento
  const docUrl = 'https://www.cia.gov/readingroom/document/15754309';
  console.log(`[1] Navegando a documento: ${docUrl}`);
  await page.goto(docUrl, { waitUntil: 'networkidle0', timeout: 30000 });
  await new Promise(r => setTimeout(r, 3000));

  // Extraer estructura detallada
  const info = await page.evaluate(() => {
    // PDF links
    const pdfSources = [];
    
    document.querySelectorAll('a').forEach(a => {
      if (a.href.match(/\.pdf$/i) || a.innerText.match(/pdf|download|descargar/i)) {
        pdfSources.push({
          text: a.innerText.trim().slice(0, 60),
          href: a.href,
          parentClass: a.parentElement?.className || '',
        });
      }
    });

    // Iframes
    document.querySelectorAll('iframe').forEach(f => {
      pdfSources.push({tag: 'iframe', href: f.src});
    });
    // Embeds
    document.querySelectorAll('embed').forEach(f => {
      pdfSources.push({tag: 'embed', href: f.src || f.getAttribute('data') || ''});
    });
    // Objects
    document.querySelectorAll('object').forEach(f => {
      pdfSources.push({tag: 'object', href: f.data || f.getAttribute('data') || ''});
    });

    // Buscar links a /download/ o /media/
    document.querySelectorAll('a[href*="/download/"], a[href*="/media/"]').forEach(a => {
      pdfSources.push({
        text: a.innerText.trim().slice(0, 60),
        href: a.href,
      });
    });

    // File fields (Drupal)
    const fileFields = [];
    document.querySelectorAll('[class*="file"], [class*="document-file"], [class*="field-file"]').forEach(el => {
      fileFields.push({
        html: el.innerHTML.slice(0, 500),
        href: el.querySelector('a')?.href || '',
      });
    });

    // Contenido principal
    const main = document.querySelector('.node__content, .content, main, .region-content, article');
    
    return {
      title: document.title,
      url: window.location.href,
      bodyLen: document.body.innerText.length,
      pdfSources,
      fileFields,
      mainHtml: main?.innerHTML?.slice(0, 3000) || '(no main)',
      // Buscar en el HTML total
      htmlPdfMatches: document.body.innerHTML.match(/href="([^"]+\.pdf)"/gi)?.slice(0, 5) || [],
    };
  });

  console.log(`Title: ${info.title}`);
  console.log(`Body: ${info.bodyLen} chars`);
  
  console.log(`\nPDF Sources (${info.pdfSources.length}):`);
  info.pdfSources.forEach((s, i) => console.log(`  ${i+1}. ${JSON.stringify(s, null, 2)}`));

  console.log(`\nFile Fields (${info.fileFields.length}):`);
  info.fileFields.forEach((f, i) => console.log(`  ${i+1}. ${f.html.slice(0, 300)}`));

  console.log(`\nHTML PDF matches: ${info.htmlPdfMatches.join(', ')}`);

  console.log(`\nMain Content HTML (first 2000 chars):`);
  console.log(info.mainHtml.slice(0, 2000));

  // Guardar HTML
  fs.writeFileSync('/tmp/cia_doc_page.html', await page.content());
  console.log('\nHTML guardado en /tmp/cia_doc_page.html');

  await browser.close();
}

main().catch(e => { console.error('ERROR:', e.message); process.exit(1); });
