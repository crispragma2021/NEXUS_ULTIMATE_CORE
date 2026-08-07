#!/usr/bin/env node
/**
 * ═══ CIA FOIA — EXTRAER DOCUMENTOS ═══
 * Estrategia: home → cookies → search → extraer links
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

  // 1. Home para cookies
  console.log('[1] Home → cookies');
  await page.goto('https://www.cia.gov/readingroom/', { waitUntil: 'networkidle0', timeout: 30000 });
  await new Promise(r => setTimeout(r, 2000));

  // 2. Search
  console.log('[2] Search operation condor');
  await page.goto('https://www.cia.gov/readingroom/search/site/operation%20condor', {
    waitUntil: 'networkidle0', timeout: 30000
  });
  await new Promise(r => setTimeout(r, 3000));

  // EXTRAER estructura DOM de los resultados
  const domStructure = await page.evaluate(() => {
    // 1. Buscar todos los elementos que contengan "/document/"
    const docLinks = [];
    document.querySelectorAll('a').forEach(a => {
      if (a.href.includes('/document/')) {
        docLinks.push({
          text: a.innerText.trim().slice(0, 100),
          href: a.href,
          parentTag: a.parentElement?.tagName || '',
          parentClass: a.parentElement?.className?.slice(0, 80) || '',
        });
      }
    });

    // 2. Buscar elementos con clase node o views-row (Drupal)
    const viewRows = [];
    document.querySelectorAll('[class*="node"], [class*="views-row"], [class*="search-result"], .result-item, article').forEach(el => {
      const html = el.innerHTML.slice(0, 500);
      const text = el.innerText.trim().slice(0, 100);
      const link = el.querySelector('a');
      viewRows.push({
        class: el.className?.slice(0, 80),
        tag: el.tagName,
        text,
        hasLink: !!link,
        linkHref: link?.href?.slice(0, 120) || '',
        html: html.replace(/\s+/g, ' ').slice(0, 300),
      });
    });

    return { docLinks, viewRows };
  });

  console.log(`\nDoc links (/document/): ${domStructure.docLinks.length}`);
  domStructure.docLinks.slice(0, 10).forEach(d => {
    console.log(`  "${d.text}"`);
    console.log(`    → ${d.href.slice(0, 120)}`);
    console.log(`    <${d.parentTag} class="${d.parentClass}">`);
  });

  console.log(`\nView rows / nodes: ${domStructure.viewRows.length}`);
  domStructure.viewRows.slice(0, 8).forEach(v => {
    console.log(`  [${v.tag}] class="${v.class}"`);
    console.log(`  text: "${v.text.slice(0, 80)}"`);
    console.log(`  html: ${v.html.slice(0, 200)}`);
    console.log();
  });

  // Guardar HTML completo
  fs.writeFileSync('/tmp/cia_search_dom.html', await page.content());
  console.log('\nHTML completo guardado en /tmp/cia_search_dom.html');

  await browser.close();
}

main().catch(e => { console.error('ERROR:', e.message); process.exit(1); });
