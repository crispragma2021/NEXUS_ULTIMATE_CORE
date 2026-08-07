#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════════════════╗
 * ║  🕵️ CIA FOIA — OPERACIÓN CÓNDOR: EXTRACTOR MASIVO V2           ║
 * ║  Sin Tor (CIA.gov es clearnet) · Puppeteer CDP para PDFs        ║
 * ╚══════════════════════════════════════════════════════════════════╝
 *
 * ARQUITECTURA:
 *   Fase 1 → Buscar en CIA FOIA (vía SPA) → extraer node-IDs de documentos
 *   Fase 2 → Visitar cada documento → extraer URL de PDF
 *   Fase 3 → Descargar PDF via CDP (dentro del browser) o via https.get()
 *
 * CLAVE: CIA FOIA usa Drupal con SPA. Los resultados de búsqueda
 * se renderizan vía JS. NO necesita Tor.
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import https from 'https';
import http from 'http';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DOWNLOAD_DIR = path.join(__dirname, 'downloads', 'cia_condor_v2');
const REPORT_DIR = path.join(__dirname, 'reports');
const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';

fs.mkdirSync(DOWNLOAD_DIR, { recursive: true });
fs.mkdirSync(REPORT_DIR, { recursive: true });

const sleep = ms => new Promise(r => setTimeout(r, ms));

// ─── QUERIES A BUSCAR ──────────────────────────────────────────
const QUERIES = [
  { name: 'Operation_Condor',          query: 'operation condor' },
  { name: 'TESEO_CONDOR_1977',          query: 'TESEO CONDOR 1977' },
  { name: 'Paraguay_intelligence',      query: 'paraguay intelligence' },
  { name: 'Stroessner_CIA',             query: 'stroessner intelligence' },
  { name: 'Coronel_Paraguay',           query: 'coronel paraguay' },
  { name: 'Argentina_Dirty_War',        query: 'argentina dirty war condor' },
  { name: 'Condor_Plan',                query: '"Operation Condor" assassination' },
];

/**
 * FASE 1: Extraer node-IDs de documentos desde resultados de búsqueda
 */
async function extractDocNodes(page, searchQuery, maxDocs = 50) {
  const url = `https://www.cia.gov/readingroom/search/site/${encodeURIComponent(searchQuery)}`;
  console.log(`  [🔍] ${searchQuery}`);
  
  await page.goto(url, { waitUntil: 'networkidle0', timeout: 60000 });
  await sleep(3000); // dar tiempo al SPA para renderizar completamente

  // Intentar scroll para cargar más resultados
  for (let i = 0; i < 3; i++) {
    await page.evaluate(() => window.scrollBy(0, 1000));
    await sleep(1500);
  }

  // Extraer nodos de documentos
  const documents = await page.evaluate(() => {
    const docs = [];

    // Método 1: Buscar enlaces que contengan /document/ en el href
    document.querySelectorAll('a[href*="/document/"]').forEach(a => {
      const href = a.href;
      const title = a.innerText.trim() || a.title || a.getAttribute('aria-label') || '';
      if (title.length > 3) {
        docs.push({
          title: title.slice(0, 250),
          url: href,
          nodeId: href.match(/\/document\/(\d+)/)?.[1] || 'unknown',
        });
      }
    });

    // Método 2: Buscar elementos de resultado que contengan títulos
    document.querySelectorAll('.views-row, .search-result, .node--type-document, li.search-result, .result-item').forEach(el => {
      const link = el.querySelector('a[href*="/document/"]');
      const text = el.innerText.trim();
      if (link && !docs.find(d => d.url === link.href) && text.length > 5) {
        docs.push({
          title: text.split('\n')[0].slice(0, 250),
          url: link.href,
          nodeId: link.href.match(/\/document\/(\d+)/)?.[1] || 'unknown',
        });
      }
    });

    // Método 3: Extraer del texto "CIA-RDP" IDs (identificadores únicos)
    const rdpMatches = document.body.innerText.match(/CIA-RDP\S+/g) || [];
    const uniqueRDPs = [...new Set(rdpMatches)].slice(0, 10);
    
    // Método 4: Buscar elementos que parezcan títulos de documentos (h2, h3 con texto largo)
    document.querySelectorAll('h2, h3, h4, .node__title, .field--name-title').forEach(h => {
      const text = h.innerText.trim();
      const link = h.closest('a') || h.querySelector('a');
      if (text.length > 15 && !docs.find(d => d.title.includes(text.slice(0, 30)))) {
        // Intentar encontrar un enlace cercano
        const nearbyLink = h.closest('div, li, article')?.querySelector('a[href*="/document/"]');
        if (nearbyLink && !docs.find(d => d.url === nearbyLink.href)) {
          docs.push({
            title: text.slice(0, 250),
            url: nearbyLink.href,
            nodeId: nearbyLink.href.match(/\/document\/(\d+)/)?.[1] || 'unknown',
          });
        }
      }
    });

    return {
      documents: docs.slice(0, 60),
      totalResults: (document.body.innerText.match(/Search found (\d+) items/) || [])[1] || 'unknown',
      rdpIds: uniqueRDPs,
    };
  });

  // Deduplicar por URL
  const uniqueDocs = [];
  const seenUrls = new Set();
  for (const doc of documents.documents) {
    if (!seenUrls.has(doc.url)) {
      seenUrls.add(doc.url);
      uniqueDocs.push(doc);
    }
  }

  console.log(`     └─ ${uniqueDocs.length} docs únicos (de ${documents.totalResults} totales)`);
  if (uniqueDocs.length > 0) {
    console.log(`     └─ Primeros:`);
    uniqueDocs.slice(0, 5).forEach((d, i) => console.log(`       ${i+1}. [${d.nodeId}] ${d.title.slice(0, 70)}`));
  }
  if (documents.rdpIds.length > 0) {
    console.log(`     └─ CIA-RDP IDs: ${documents.rdpIds.slice(0, 5).join(', ')}`);
  }

  return { documents: uniqueDocs.slice(0, maxDocs), totalResults: documents.totalResults, rdpIds: documents.rdpIds };
}

/**
 * FASE 2+3: Navegar a un documento y descargar su PDF
 */
async function downloadDocPDF(page, doc, searchDir, index, total) {
  const safeTitle = doc.title.replace(/[^a-zA-Z0-9_áéíóúñü\s-]/g, '').trim().slice(0, 60).replace(/\s+/g, '_');
  const filename = `cia_${index}_${safeTitle}.pdf`;
  const filepath = path.join(searchDir, filename);

  // Saltar si ya existe
  if (fs.existsSync(filepath) && fs.statSync(filepath).size > 1000) {
    console.log(`     └─ ⏭️ Ya existe (${Math.round(fs.statSync(filepath).size/1024)} KB)`);
    return true;
  }

  console.log(`  [📄] (${index}/${total}) ${doc.title.slice(0, 60)}`);

  try {
    await page.goto(doc.url, { waitUntil: 'networkidle0', timeout: 30000 });
    await sleep(2000);

    // Buscar URL de PDF
    const pdfUrl = await page.evaluate(() => {
      // Selectores específicos de Drupal / CIA readingroom
      const selectors = [
        'a[href$=".pdf"]',
        'a[href*="/download"]',
        'a.file-link--pdf',
        '.field--name-field-document-file a',
        '.field--type-file a',
        '.file a',
        'a[href*="/media/"]',
        'a[href*="/document_file/"]',
        'a[rel="media-no-preview"]',
        // Cualquier enlace que contenga "pdf" en href
        'a[href*="pdf"]',
      ];
      
      for (const sel of selectors) {
        const el = document.querySelector(sel);
        if (el && el.href) return el.href;
      }

      // Fallback: buscar enlaces con texto Download o PDF
      const allLinks = Array.from(document.querySelectorAll('a'));
      const found = allLinks.find(a =>
        a.href && (a.innerText.match(/pdf|download|document/i) || a.className.match(/button|download/i))
      );
      return found ? found.href : null;
    });

    if (pdfUrl) {
      const fullUrl = pdfUrl.startsWith('http') ? pdfUrl : `https://www.cia.gov${pdfUrl}`;
      console.log(`     └─ 📥 PDF URL: ${fullUrl.slice(0, 100)}`);

      // DESCARGAR: usar https.get directamente (CIA.gov no necesita proxy)
      const success = await downloadHTTPS(fullUrl, filepath);
      if (success) {
        const size = Math.round(fs.statSync(filepath).size / 1024);
        console.log(`     └─ ✅ ${size} KB`);
        return true;
      } else {
        // Fallback: si https.get falla, intentar puppeteer capture
        console.log(`     └─ ⚠️ HTTPS falló, intentando puppeteer...`);
        return await downloadViaPuppeteer(page, fullUrl, filepath);
      }
    } else {
      console.log(`     └─ ❌ Sin PDF URL en página`);
      return false;
    }
  } catch (err) {
    console.log(`     └─ ❌ Error: ${err.message.slice(0, 100)}`);
    return false;
  }
}

/**
 * Descargar vía https.get (directa, CIA.gov no necesita proxy)
 */
function downloadHTTPS(url, filepath) {
  return new Promise((resolve) => {
    const file = fs.createWriteStream(filepath);
    const timer = setTimeout(() => {
      file.close();
      fs.unlinkSync(filepath);
      resolve(false);
    }, 30000);

    const httpModule = url.startsWith('https') ? https : http;
    
    httpModule.get(url, {
      headers: {
        'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36',
        'Accept': 'application/pdf,application/octet-stream,*/*',
      },
      rejectUnauthorized: false,
      followRedirect: true,
    }, (response) => {
      // Manejar redirect manualmente
      if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        clearTimeout(timer);
        file.close();
        fs.unlinkSync(filepath);
        downloadHTTPS(response.headers.location, filepath).then(resolve);
        return;
      }

      const contentType = response.headers['content-type'] || '';
      const contentLength = parseInt(response.headers['content-length'] || '0');

      // Si es HTML (en vez de PDF), probablemente requiere JS
      if (contentType.includes('text/html') && contentLength < 10000) {
        clearTimeout(timer);
        file.close();
        fs.unlinkSync(filepath);
        resolve(false);
        return;
      }

      response.pipe(file);
      file.on('finish', () => {
        clearTimeout(timer);
        file.close();
        if (fs.statSync(filepath).size > 500) {
          resolve(true);
        } else {
          fs.unlinkSync(filepath);
          resolve(false);
        }
      });
    }).on('error', () => {
      clearTimeout(timer);
      file.close();
      fs.unlinkSync(filepath);
      resolve(false);
    });
  });
}

/**
 * Fallback: Capturar PDF via Puppeteer (navegando directamente)
 */
async function downloadViaPuppeteer(page, url, filepath) {
  try {
    // Navegar directamente a la URL del PDF
    await page.goto(url, { waitUntil: 'load', timeout: 30000 });
    await sleep(2000);

    // Ver si renderizó algo
    const contentType = await page.evaluate(() => document.contentType || document.body.innerText.slice(0, 100));

    // Si el navegador muestra el PDF, podemos capturarlo via CDP
    if (page.url().includes('.pdf') || contentType === 'unknown') {
      // Capturar el buffer de la respuesta
      const cdp = await page.createCDPSession();
      
      // Enable Network domain to capture response
      await cdp.send('Network.enable');
      
      let pdfData = null;
      const handler = (response) => {
        if (response.url.includes('.pdf') || response.headers['content-type']?.includes('pdf')) {
          pdfData = response;
        }
      };
      
      page.on('response', handler);
      await page.reload({ waitUntil: 'load', timeout: 30000 });
      await sleep(3000);
      page.removeListener('response', handler);

      // Intentar getResponseBody
      try {
        const result = await cdp.send('Network.getResponseBody', {
          requestId: pdfData?.requestId
        });
        if (result && result.body && result.body.length > 500) {
          const buffer = Buffer.from(result.body, result.base64Encoded ? 'base64' : 'utf8');
          fs.writeFileSync(filepath, buffer);
          return true;
        }
      } catch(e) {}
    }

    return false;
  } catch(e) {
    return false;
  }
}

/**
 * EXTRAER links de documentos usando el buscador de NEXUS 
 * (el archivo de report existente tiene links en el content)
 * pero los href no están. Necesito usar Puppeteer para extraerlos.
 */
async function main() {
  console.log('='.repeat(70));
  console.log('  🕵️ CIA FOIA — OPERACIÓN CÓNDOR EXTRACTOR V2');
  console.log('  📥 Descargando documentos desclasificados');
  console.log('='.repeat(70));

  const browser = await puppeteer.launch({
    headless: true,
    executablePath: CHROME_PATH,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
  });

  // Crear páginas: 1 para búsqueda, 1 para descarga
  const searchPage = await browser.newPage();
  await searchPage.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36');
  
  const allResults = {};

  for (const q of QUERIES) {
    const searchDir = path.join(DOWNLOAD_DIR, q.name);
    fs.mkdirSync(searchDir, { recursive: true });

    try {
      // FASE 1: Extraer documentos
      const { documents, totalResults, rdpIds } = await extractDocNodes(searchPage, q.query);
      
      // FASE 2+3: Descargar cada documento
      let downloaded = 0;
      let failed = 0;
      
      for (let i = 0; i < documents.length; i++) {
        // Usar página nueva para cada descarga (evita estado residual)
        const dlPage = await browser.newPage();
        await dlPage.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36');
        
        const ok = await downloadDocPDF(dlPage, documents[i], searchDir, i + 1, documents.length);
        if (ok) downloaded++;
        else failed++;
        
        await dlPage.close();
        await sleep(1000); // pausa entre descargas
      }

      allResults[q.name] = {
        query: q.query,
        totalResults,
        documentsFound: documents.length,
        downloaded,
        failed,
        rdpIds,
        documents: documents.map(d => ({ title: d.title.slice(0, 100), url: d.url, nodeId: d.nodeId })),
      };

      console.log(`\n  📊 ${q.name}: ${downloaded}/${documents.length} PDFs descargados\n`);

    } catch (err) {
      console.error(`\n  ❌ Error en "${q.name}": ${err.message}`);
      allResults[q.name] = { query: q.query, error: err.message };
    }
  }

  await browser.close();

  // Guardar reporte
  const reportPath = path.join(REPORT_DIR, `cia_condor_v2_${Date.now()}.json`);
  fs.writeFileSync(reportPath, JSON.stringify(allResults, null, 2));
  
  console.log('\n' + '='.repeat(70));
  console.log('  📊 RESUMEN FINAL');
  console.log('='.repeat(70));
  let totalDocs = 0;
  let totalDL = 0;
  for (const [name, data] of Object.entries(allResults)) {
    console.log(`  📍 ${name}: ${data.documentsFound || 0} docs → ${data.downloaded || 0} PDFs ${data.failed ? `(${data.failed} fallos)` : ''}`);
    totalDocs += data.documentsFound || 0;
    totalDL += data.downloaded || 0;
  }
  console.log(`\n  📊 TOTAL: ${totalDL}/${totalDocs} PDFs descargados`);
  console.log(`  📁 Directorio: ${DOWNLOAD_DIR}`);
  console.log(`  💾 Reporte: ${reportPath}`);
  console.log('='.repeat(70));
}

main().catch(err => {
  console.error('💥 FATAL:', err);
  process.exit(1);
});
