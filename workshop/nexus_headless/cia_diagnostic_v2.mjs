#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════╗
 * ║  🩺 CIA FOIA — DIAGNÓSTICO V2 (SIN TOR, RÁPIDO)     ║
 * ║  CIA.gov es clearnet — no necesita proxy             ║
 * ╚══════════════════════════════════════════════════════╝
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';

async function diagnose() {
  console.log('='.repeat(70));
  console.log('  🩺 CIA FOIA DIAGNÓSTICO V2 — SIN PROXY');
  console.log('='.repeat(70));

  const browser = await puppeteer.launch({
    headless: true,
    executablePath: CHROME_PATH,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  // 1. RESULTADOS DE BÚSQUEDA
  console.log('\n[1/4] Buscando "operation condor" en CIA FOIA...');
  const searchPage = await browser.newPage();
  await searchPage.goto('https://www.cia.gov/readingroom/search/site/operation%20condor', {
    waitUntil: 'networkidle0',
    timeout: 30000,
  });

  // Extraer documentos
  const searchResults = await searchPage.evaluate(() => {
    // Buscar TODOS los patrones posibles de selectores de resultados
    const docLinks = new Map();

    // 1. Search results con enlaces a /document/
    document.querySelectorAll('a[href*="/document/"]').forEach(a => {
      const title = a.innerText.trim();
      if (title.length > 5 && !docLinks.has(a.href)) {
        docLinks.set(a.href, { title: title.slice(0, 200), href: a.href });
      }
    });

    // 2. Elementos listados como resultados (Drupal views)
    document.querySelectorAll('.views-row, .search-result, .node--type-document, li.search-result, .result-item').forEach(el => {
      const link = el.querySelector('a[href*="/document/"]');
      const title = el.innerText.trim();
      if (link && title.length > 5) {
        docLinks.set(link.href, { title: title.split('\n')[0].slice(0, 200), href: link.href });
      }
    });

    // 3. OL/LI results (lista ordenada)
    document.querySelectorAll('ol li, ul li').forEach(li => {
      const link = li.querySelector('a[href*="/document/"]');
      if (link) {
        const title = li.innerText.trim();
        if (title.length > 5) {
          docLinks.set(link.href, { title: title.split('\n')[0].slice(0, 200), href: link.href });
        }
      }
    });

    return Array.from(docLinks.values()).slice(0, 20);
  });

  console.log(`     └─ ${searchResults.length} documentos encontrados`);
  searchResults.slice(0, 10).forEach((d, i) => {
    console.log(`       ${i+1}. ${d.title.slice(0, 70)}`);
    console.log(`          ${d.href.slice(0, 100)}`);
  });

  // 2. Diagnosticar página de documento individual
  if (searchResults.length > 0) {
    const docUrl = searchResults[0].href;
    console.log(`\n[2/4] Diagnosticando documento: ${docUrl.slice(0, 100)}`);
    
    const docPage = await browser.newPage();
    await docPage.goto(docUrl, { waitUntil: 'networkidle0', timeout: 30000 });
    await new Promise(r => setTimeout(r, 2000));

    // Extraer todo lo relacionado a PDF/download
    const docInfo = await docPage.evaluate(() => {
      const result = {
        title: document.title,
        url: window.location.href,
        allLinks: [],
        pdfUrls: [],
        downloadButtons: [],
        mediaItems: [],
        possibleSelectors: {}
      };

      // Todos los enlaces
      document.querySelectorAll('a[href]').forEach(a => {
        const href = a.href;
        const text = a.innerText.trim().slice(0, 80);
        const classes = a.className;
        result.allLinks.push({ text, href: href.slice(0, 200), classes: classes.slice(0, 50) });

        // Detectar PDF
        if (href.match(/\.pdf$/i) || href.match(/\/download/) || href.match(/\/media\//)) {
          result.pdfUrls.push({ text, href, classes });
        }
        // Detectar botones de descarga
        if (text.match(/pdf|download|descargar|document|file/i) || classes.match(/button|download|pdf/i)) {
          result.downloadButtons.push({ text, href, classes });
        }
      });

      // Media/File fields (Drupal)
      document.querySelectorAll('.field--name-field-document-file, .field--type-file, .file, .file-link, [class*="file"]').forEach(el => {
        const link = el.querySelector('a');
        result.mediaItems.push({
          html: el.innerHTML.slice(0, 300),
          href: link ? link.href : null,
          text: link ? link.innerText.trim().slice(0, 80) : null,
        });
      });

      // Buscar en el HTML completo por patrones de archivo
      const bodyHtml = document.body.innerHTML;
      const pdfMatches = bodyHtml.match(/href="([^"]+\.pdf)"/gi);
      result.possibleSelectors.pdfHrefs = pdfMatches ? pdfMatches.slice(0, 5) : [];

      // Buscar data-* attributes con file info
      const fileDataMatches = bodyHtml.match(/data-[^=]+="[^"]*\.pdf[^"]*"/gi);
      result.possibleSelectors.fileData = fileDataMatches ? fileDataMatches.slice(0, 5) : [];

      // Ver meta tags
      const metaDesc = document.querySelector('meta[name="description"]');
      result.metaDescription = metaDesc ? metaDesc.content.slice(0, 200) : null;

      return result;
    });

    console.log(`     └─ Título: ${docInfo.title.slice(0, 100)}`);
    console.log(`     └─ Total enlaces: ${docInfo.allLinks.length}`);
    console.log(`     └─ URLs PDF detectadas: ${docInfo.pdfUrls.length}`);
    docInfo.pdfUrls.forEach((p, i) => {
      console.log(`       ${i+1}. [${p.text}] ${p.href}`);
    });
    console.log(`     └─ Botones de descarga: ${docInfo.downloadButtons.length}`);
    docInfo.downloadButtons.forEach((b, i) => {
      console.log(`       ${i+1}. "${b.text}" → ${b.href.slice(0, 120)}`);
    });
    console.log(`     └─ Media/File fields: ${docInfo.mediaItems.length}`);
    docInfo.mediaItems.forEach((m, i) => {
      console.log(`       ${i+1}. ${m.html.slice(0, 200)}`);
    });

    // 3. CAPTURAR HTML RELEVANTE
    console.log('\n[3/4] HTML de la sección de contenido del documento (primeros 2000 chars):');
    const mainContent = await docPage.evaluate(() => {
      const main = document.querySelector('main, .main-content, #content, .content, .node__content, article, .region-content');
      if (main) return main.innerHTML.slice(0, 3000);
      return 'NO MAIN CONTENT FOUND';
    });
    console.log(mainContent.slice(0, 2000));

    // 4. PROBAR DESCARGA DIRECTA con CDP
    console.log('\n[4/4] Configurando CDP download...');
    const cdp = await docPage.createCDPSession();
    const downloadDir = '/tmp/cia_diag_dl';
    fs.mkdirSync(downloadDir, { recursive: true });
    
    await cdp.send('Page.setDownloadBehavior', {
      behavior: 'allow',
      downloadPath: downloadDir,
    });

    // Si hay URL de PDF, intentar navegar directamente
    if (docInfo.pdfUrls.length > 0) {
      const pdfUrl = docInfo.pdfUrls[0].href;
      console.log(`     └─ Navegando a URL de PDF: ${pdfUrl.slice(0, 120)}`);
      
      // Verificar que existe con una petición HEAD primero
      try {
        await docPage.goto(pdfUrl, { waitUntil: 'networkidle0', timeout: 30000 });
        console.log(`     └─ Navegación exitosa. URL final: ${docPage.url().slice(0, 120)}`);
        
        const contentType = await docPage.evaluate(() => document.contentType || 'unknown');
        console.log(`     └─ Content-Type: ${contentType}`);

        // Si es PDF, capturar el buffer via CDP
        const cdp2 = await docPage.createCDPSession();
        const result = await cdp2.send('Page.captureSnapshot', { format: 'pdf' });
        console.log(`     └─ Snapshot PDF: ${result.data.length} bytes`);
        
      } catch(e) {
        console.log(`     └─ Error: ${e.message.slice(0, 100)}`);
      }
    }

    await docPage.close();
  }

  await browser.close();
  console.log('\n✅ Diagnóstico completado');
}

diagnose().catch(console.error);
