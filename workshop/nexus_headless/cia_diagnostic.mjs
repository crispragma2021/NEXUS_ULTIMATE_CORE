#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════╗
 * ║  🩺 CIA FOIA — DIAGNÓSTICO DE ESTRUCTURA            ║
 * ║  Descubre cómo se sirven realmente los PDFs          ║
 * ╚══════════════════════════════════════════════════════╝
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';
const PROXY = 'socks5://127.0.0.1:9050';

async function diagnose() {
  console.log('='.repeat(70));
  console.log('  🩺 DIAGNÓSTICO CIA FOIA — ESTRUCTURA DE DOCUMENTOS');
  console.log('='.repeat(70));

  const browser = await puppeteer.launch({
    headless: true,
    executablePath: CHROME_PATH,
    args: [
      '--no-sandbox', '--disable-setuid-sandbox',
      `--proxy-server=${PROXY}`,
    ],
  });

  const page = await browser.newPage();
  
  // Interceptar TODAS las respuestas para ver cómo se sirven los PDFs
  const responses = [];
  page.on('response', async (response) => {
    const url = response.url();
    const contentType = response.headers()['content-type'] || '';
    const status = response.status();
    
    if (contentType.includes('pdf') || url.includes('.pdf') || 
        (contentType.includes('application/') && status === 200)) {
      responses.push({
        url: url,
        status: status,
        contentType: contentType,
        contentLength: response.headers()['content-length'] || 'unknown',
      });
    }
  });

  // 1. Diagnosticar página de resultados de búsqueda
  console.log('\n[1/4] Navegando a resultados de búsqueda...');
  await page.goto('https://www.cia.gov/readingroom/search/site/operation%20condor', {
    waitUntil: 'networkidle0',
    timeout: 60000,
  });

  // Extraer estructura de la página
  const pageStructure = await page.evaluate(() => {
    // Encontrar todos los enlaces que sean documentos
    const docLinks = Array.from(document.querySelectorAll('a[href*="/document/"]'))
      .map(a => ({ title: a.innerText.trim().slice(0, 100), href: a.href }))
      .filter(a => a.title.length > 5);

    // Buscar selectores de resultados
    const selectors = ['.search-result', '.views-row', '.node', 'article', '.teaser', '.result', 'li.search-result'];
    const found = selectors.filter(sel => document.querySelector(sel));
    
    // Ver elementos de resultado real
    const allResults = [];
    document.querySelectorAll('ol, ul').forEach(list => {
      const items = list.querySelectorAll('li');
      if (items.length > 2) {
        items.forEach(li => {
          const link = li.querySelector('a[href]');
          if (link && link.href.includes('/document/')) {
            allResults.push({
              title: li.innerText.trim().slice(0, 150),
              href: link.href,
              html: li.innerHTML.slice(0, 500),
            });
          }
        });
      }
    });

    return { docLinks, selectorsFound: found, allResults };
  });

  console.log(`     └─ Links con /document/: ${pageStructure.docLinks.length}`);
  console.log(`     └─ Selectores encontrados: ${pageStructure.selectorsFound.join(', ') || 'NINGUNO'}`);
  console.log(`     └─ Resultados detectados: ${pageStructure.allResults.length}`);
  
  if (pageStructure.docLinks.length > 0) {
    console.log('\n     Primeros 5 documentos:');
    pageStructure.docLinks.slice(0, 5).forEach((d, i) => {
      console.log(`       ${i+1}. ${d.title.slice(0, 70)}`);
      console.log(`          ${d.href}`);
    });
  }

  // 2. Navegar a un documento individual
  console.log('\n[2/4] Navegando al PRIMER documento...');
  const firstDoc = pageStructure.allResults[0] || pageStructure.docLinks[0];
  if (firstDoc) {
    console.log(`     └─ URL: ${firstDoc.href}`);
    await page.goto(firstDoc.href, { waitUntil: 'networkidle0', timeout: 60000 });
    await new Promise(r => setTimeout(r, 3000));

    // Extraer estructura de la página del documento
    const docStructure = await page.evaluate(() => {
      const pdfLinks = Array.from(document.querySelectorAll('a[href*=".pdf"], a[href*="/download"], a[href*="media/"]'))
        .map(a => ({ text: a.innerText.trim().slice(0, 100), href: a.href }));
      
      // Buscar botones con texto Download/PDF
      const downloadButtons = Array.from(document.querySelectorAll('a, button'))
        .filter(el => el.innerText.match(/pdf|download|descargar|document|file/i))
        .map(el => ({ text: el.innerText.trim().slice(0, 100), href: el.href || el.innerText }));
      
      // Buscar iframes o embeds
      const embeds = Array.from(document.querySelectorAll('iframe, embed, object'))
        .map(el => ({ tag: el.tagName, src: el.src || el.data || '(none)' }));
      
      // Título del documento
      const title = document.title;
      
      // Campo de "File" en metadata
      const fileField = document.querySelector('.file, .field--name-field-document-file, .field--type-file');
      const fileHtml = fileField ? fileField.innerHTML.slice(0, 500) : null;
      
      return { pdfLinks, downloadButtons, embeds, title, fileHtml };
    });

    console.log(`     └─ Título: ${docStructure.title.slice(0, 100)}`);
    console.log(`     └─ Links PDF encontrados: ${docStructure.pdfLinks.length}`);
    docStructure.pdfLinks.forEach((l, i) => {
      console.log(`       ${i+1}. [${l.text.slice(0, 50)}] → ${l.href.slice(0, 100)}`);
    });
    console.log(`     └─ Botones de descarga: ${docStructure.downloadButtons.length}`);
    docStructure.downloadButtons.forEach((b, i) => {
      console.log(`       ${i+1}. "${b.text.slice(0, 50)}" → ${(b.href || '(none)').slice(0, 100)}`);
    });
    console.log(`     └─ Embeds/iframes: ${docStructure.embeds.length}`);
    docStructure.embeds.forEach((e, i) => {
      console.log(`       ${i+1}. <${e.tag}> src=${e.src.slice(0, 100)}`);
    });

    if (docStructure.fileHtml) {
      console.log(`     └─ Campo File HTML (primeros 500 chars):`);
      console.log(`       ${docStructure.fileHtml.replace(/\n/g, '\n       ')}`);
    }

    // 3. Capturar HTML completo de la página del documento
    console.log('\n[3/4] HTML relevante de la página del documento...');
    const relevantHtml = await page.evaluate(() => {
      // Buscar el contenido principal
      const main = document.querySelector('main, .main-content, #content, .content, .node__content, article');
      return main ? main.innerHTML.slice(0, 3000) : document.body.innerHTML.slice(0, 3000);
    });
    console.log(`     └─ (primeros 2000 chars):`);
    console.log(`       ${relevantHtml.slice(0, 2000).replace(/\n/g, '\n       ')}`);
  }

  // 4. Probar DESCARGA real con Puppeteer CDP
  console.log('\n[4/4] Probando captura de PDF via CDP...');
  if (firstDoc) {
    // Configurar captura de peticiones
    const cdp = await page.createCDPSession();
    await cdp.send('Page.setDownloadBehavior', {
      behavior: 'allow',
      downloadPath: '/tmp/cia_diagnostic_downloads',
    });
    fs.mkdirSync('/tmp/cia_diagnostic_downloads', { recursive: true });

    // Buscar el link de descarga
    const pdfUrl = await page.evaluate(() => {
      // Intentar varios selectores comunes en Drupal (CIA usa Drupal)
      const selectors = [
        'a[href$=".pdf"]',
        'a[href*="/download"]', 
        'a.file-link--pdf',
        '.field--name-field-document-file a',
        '.file a',
        '.node__content a.button',
        'a[rel="media-no-preview"]',
        'a[href*="/media/"]',
        'a[href*="/document_file/"]',
      ];
      
      for (const sel of selectors) {
        const el = document.querySelector(sel);
        if (el && el.href) return el.href;
      }
      
      // Fallback: cualquier link con texto PDF
      const links = Array.from(document.querySelectorAll('a'));
      const pdfLink = links.find(l => 
        l.href && (l.href.match(/\.pdf$/i) || l.innerText.match(/pdf/i))
      );
      return pdfLink ? pdfLink.href : null;
    });

    if (pdfUrl) {
      console.log(`     └─ URL de descarga detectada: ${pdfUrl.slice(0, 120)}`);
      
      // Intentar navegar directamente al PDF
      console.log(`     └─ Navegando directamente al PDF...`);
      try {
        await page.goto(pdfUrl, { waitUntil: 'networkidle0', timeout: 60000 });
        await new Promise(r => setTimeout(r, 2000));
        
        // Ver qué pasó
        const currentUrl = page.url();
        const contentType = await page.evaluate(() => {
          return document.contentType || 'unknown';
        });
        console.log(`     └─ URL final: ${currentUrl.slice(0, 120)}`);
        console.log(`     └─ contentType: ${contentType}`);
        
        // Intentar capturar el PDF con CDP
        try {
          const pdfData = await cdp.send('Page.printToPDF', {});
          console.log(`     └─ printToPDF: ${pdfData.data.length} bytes (esto sería una captura, no el PDF original)`);
        } catch(e) {
          console.log(`     └─ printToPDF falló: ${e.message.slice(0, 100)}`);
        }
      } catch(e) {
        console.log(`     └─ Error navegando al PDF: ${e.message.slice(0, 100)}`);
      }
    } else {
      console.log(`     └─ ❌ No se detectó URL de PDF`);
      
      // Mostrar todos los enlaces en la página del documento
      const allLinks = await page.evaluate(() => {
        return Array.from(document.querySelectorAll('a[href]'))
          .filter(a => a.href)
          .map(a => ({ text: a.innerText.trim().slice(0, 50), href: a.href.slice(0, 150) }));
      });
      console.log(`     └─ Todos los enlaces en la página:`);
      allLinks.slice(0, 20).forEach((l, i) => {
        console.log(`       ${i+1}. "${l.text}" → ${l.href}`);
      });
    }
  }

  // 5. Ver las respuestas interceptadas
  console.log('\n[5/4] Respuestas con contenido descargable interceptadas:');
  responses.forEach((r, i) => {
    console.log(`  ${i+1}. [${r.status}] ${r.contentType.slice(0, 50)}`);
    console.log(`     ${r.url.slice(0, 120)}`);
    console.log(`     Size: ${r.contentLength}`);
  });

  console.log('\n' + '='.repeat(70));
  console.log('  🩺 DIAGNÓSTICO COMPLETADO');
  console.log('='.repeat(70));
  
  await browser.close();
}

diagnose().catch(console.error);
