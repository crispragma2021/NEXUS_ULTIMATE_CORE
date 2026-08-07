#!/usr/bin/env node
/**
 * ═══ CIA FOIA — TEST RÁPIDO DE ESTRUCTURA ═══
 * Navega 1 página de resultado, extrae estructura,
 * navega 1 documento, extrae PDF URLs
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
  await page.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36');

  // 1. Buscar
  console.log('[1] Navegando a resultados de búsqueda...');
  await page.goto('https://www.cia.gov/readingroom/search/site/operation%20condor', {
    waitUntil: 'networkidle0', timeout: 30000
  });
  await new Promise(r => setTimeout(r, 3000));

  // Extraer estructura detallada
  const structure = await page.evaluate(() => {
    const results = [];
    
    // Listar TODOS los enlaces con su texto y href
    document.querySelectorAll('a').forEach(a => {
      const text = a.innerText.trim();
      if (text.length > 3 || a.href.includes('/document/') || a.href.includes('.pdf')) {
        results.push({
          text: a.innerText.trim().slice(0, 80),
          href: a.href.slice(0, 150),
          classes: (a.className || '').slice(0, 50),
        });
      }
    });

    // Buscar específicamente en listas ordenadas/no ordenadas
    const listItems = [];
    document.querySelectorAll('ol > li, ul > li').forEach(li => {
      const link = li.querySelector('a');
      const text = li.innerText.trim().slice(0, 100);
      if (text.length > 10) {
        listItems.push({
          text,
          href: link ? link.href.slice(0, 150) : '(none)',
          children: li.children.length,
        });
      }
    });

    return { results, listItems };
  });

  console.log(`\nLinks totales: ${structure.results.length}`);
  console.log(`\nLinks con /document/ en href:`);
  structure.results.filter(r => r.href.includes('/document/')).forEach((r, i) => {
    console.log(`  ${i+1}. [${r.text.slice(0, 60)}]`);
    console.log(`       ${r.href.slice(0, 120)}`);
  });

  console.log(`\nLinks con .pdf en href:`);
  structure.results.filter(r => r.href.includes('.pdf')).forEach((r, i) => {
    console.log(`  ${i+1}. [${r.text.slice(0, 60)}]`);
    console.log(`       ${r.href.slice(0, 120)}`);
  });

  console.log(`\nList items (li):`);
  structure.listItems.slice(0, 15).forEach((li, i) => {
    console.log(`  ${i+1}. ${li.text.slice(0, 70)}`);
    console.log(`       href: ${li.href.slice(0, 120)}`);
  });

  // 2. Si hay algún documento link, navegar a él
  const docLinks = structure.results.filter(r => r.href.includes('/document/') && r.text.length > 5);
  
  // También buscar en list items
  const docListItems = structure.listItems.filter(li => li.href.includes('/document/'));
  
  const targetUrl = docLinks[0]?.href || docListItems[0]?.href;
  
  if (targetUrl) {
    console.log(`\n[2] Navegando a documento: ${targetUrl.slice(0, 120)}`);
    await page.goto(targetUrl, { waitUntil: 'networkidle0', timeout: 30000 });
    await new Promise(r => setTimeout(r, 3000));

    // Extraer estructura del documento
    const docStructure = await page.evaluate(() => {
      // Buscar si el contenido está en un iframe
      const iframes = Array.from(document.querySelectorAll('iframe'))
        .map(f => ({ src: f.src, title: f.title }));
      
      // Buscar todas las etiquetas que puedan contener PDFs
      const pdfElements = [];
      
      // Embeds
      document.querySelectorAll('embed, object').forEach(el => {
        pdfElements.push({
          tag: el.tagName,
          src: el.src || el.getAttribute('data') || '(none)',
          type: el.type || '',
        });
      });

      // Enlaces PDF
      document.querySelectorAll('a').forEach(a => {
        if (a.href.match(/\.pdf/i) || a.innerText.match(/pdf|download/i)) {
          pdfElements.push({
            tag: 'a',
            text: a.innerText.trim().slice(0, 60),
            href: a.href.slice(0, 200),
          });
        }
      });

      // Buscar field--name-field-document-file (Drupal file field)
      const fileFields = Array.from(document.querySelectorAll('[class*="document-file"], [class*="field-file"], .file, [class*="file--"]'))
        .map(el => ({
          html: el.innerHTML.slice(0, 400),
          link: el.querySelector('a')?.href || '(none)',
        }));

      // Contenido principal
      const mainContent = document.querySelector('.node__content, .content, main, .region-content, article');
      const mainHtml = mainContent ? mainContent.innerHTML.slice(0, 2000) : '(no main content found)';

      return { iframes, pdfElements, fileFields, mainHtml };
    });

    console.log(`\nIframes: ${docStructure.iframes.length}`);
    docStructure.iframes.forEach(f => console.log(`  src: ${f.src.slice(0, 120)}`));
    
    console.log(`\nPDF elements: ${docStructure.pdfElements.length}`);
    docStructure.pdfElements.forEach(p => console.log(`  <${p.tag}> ${p.text || ''} → ${(p.src || p.href).slice(0, 120)}`));
    
    console.log(`\nFile fields: ${docStructure.fileFields.length}`);
    docStructure.fileFields.forEach(f => console.log(`  ${f.html.slice(0, 200)}`));

    console.log(`\nMain content HTML (primeros 1500 chars):`);
    console.log(docStructure.mainHtml.slice(0, 1500));
  }

  await browser.close();
}

main().catch(e => { console.error('ERROR:', e.message); process.exit(1); });
