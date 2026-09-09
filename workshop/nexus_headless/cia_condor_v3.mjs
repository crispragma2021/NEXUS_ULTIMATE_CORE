#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════════════════╗
 * ║  🕵️ CIA FOIA — OPERACIÓN CÓNDOR: EXTRACTOR V3                  ║
 * ║  CDP Network Capture: intercepta PDFs dentro del browser         ║
 * ║  Sin Tor · Sin https.get externo · Captura binaria real          ║
 * ╚══════════════════════════════════════════════════════════════════╝
 *
 * ARQUITECTURA:
 *   Fase 1 → Puppeteer navega a resultados, extrae node-IDs
 *   Fase 2 → Por cada doc, navega y captura respuesta PDF via CDP
 *   Fase 3 → Guarda el binario a disco
 *
 * CLAVE: Usa CDP Network.enable + responseReceived para capturar
 * el binario del PDF tal como lo recibe el browser (con cookies y sesión)
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DOWNLOAD_DIR = path.join(__dirname, 'downloads', 'cia_condor_v3');
const REPORT_DIR = path.join(__dirname, 'reports');
const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';

fs.mkdirSync(DOWNLOAD_DIR, { recursive: true });
fs.mkdirSync(REPORT_DIR, { recursive: true });

const sleep = ms => new Promise(r => setTimeout(r, ms));

const QUERIES = [
  { name: 'Operation_Condor',          query: 'operation condor',          max: 30 },
  { name: 'TESEO_CONDOR_1977',         query: 'TESEO CONDOR 1977',         max: 30 },
  { name: 'Paraguay_intelligence',     query: 'paraguay intelligence',     max: 30 },
  { name: 'Stroessner_CIA',            query: 'stroessner intelligence',   max: 30 },
  { name: 'Coronel_Paraguay',          query: 'coronel paraguay',          max: 30 },
  { name: 'Argentina_Dirty_War',       query: 'argentina dirty war condor',max: 30 },
];

/**
 * FASE 1: Extraer documentos de resultados de búsqueda
 */
async function extractDocNodes(page, searchQuery) {
  const url = `https://www.cia.gov/readingroom/search/site/${encodeURIComponent(searchQuery)}`;
  console.log(`  [🔍] "${searchQuery}"`);
  
  await page.goto(url, { waitUntil: 'networkidle0', timeout: 60000 });
  await sleep(3000);

  // Scroll para cargar más
  for (let i = 0; i < 3; i++) {
    await page.evaluate(() => window.scrollBy(0, 1000));
    await sleep(1500);
  }

  const result = await page.evaluate(() => {
    const docs = [];
    const seen = new Set();

    // Buscar TODOS los enlaces a /document/
    document.querySelectorAll('a[href*="/document/"]').forEach(a => {
      const href = a.href;
      if (seen.has(href)) return;
      seen.add(href);
      const title = a.innerText.trim() || a.title || a.getAttribute('aria-label') || '';
      const parentText = a.closest('div, li, article, h2, h3, h4')?.innerText?.trim() || '';
      const combined = (title.length > 3 ? title : parentText).slice(0, 250);
      if (combined.length > 3) {
        docs.push({
          title: combined,
          url: href,
          nodeId: href.match(/\/document\/(\d+)/)?.[1] || 'unknown',
        });
      }
    });

    // Si no hay suficientes, buscar títulos con estructura Drupal
    if (docs.length < 5) {
      document.querySelectorAll('.views-row, .search-result, li.search-result, .result-item').forEach(el => {
        const link = el.querySelector('a[href*="/document/"]');
        const text = el.innerText.trim().split('\n')[0].slice(0, 250);
        if (link && !seen.has(link.href) && text.length > 5) {
          seen.add(link.href);
          docs.push({
            title: text,
            url: link.href,
            nodeId: link.href.match(/\/document\/(\d+)/)?.[1] || 'unknown',
          });
        }
      });
    }

    return {
      documents: docs,
      totalResults: (document.body.innerText.match(/Search found (\d+) items/) || [])[1] || '?',
    };
  });

  console.log(`     └─ ${result.documents.length} docs (${result.totalResults} totales)`);
  result.documents.slice(0, 5).forEach((d, i) =>
    console.log(`       ${i+1}. [${d.nodeId}] ${d.title.slice(0, 70)}`));

  return result;
}

/**
 * FASE 2+3: Navegar a un documento y capturar su PDF via CDP
 */
async function captureDocPDF(page, doc, searchDir, index, total) {
  const safeTitle = doc.title.replace(/[^a-zA-Z0-9_áéíóúñü\s-]/g, '').trim().slice(0, 60).replace(/\s+/g, '_');
  const filename = `cia_${String(index).padStart(3, '0')}_${safeTitle}.pdf`;
  const filepath = path.join(searchDir, filename);

  if (fs.existsSync(filepath) && fs.statSync(filepath).size > 1000) {
    console.log(`     └─ ⏭️ Ya existe (${Math.round(fs.statSync(filepath).size/1024)} KB)`);
    return true;
  }

  console.log(`  [📄] (${index}/${total}) ${doc.title.slice(0, 60)}`);

  try {
    // ─── Configurar CDP para capturar respuestas ───
    const cdp = await page.createCDPSession();
    await cdp.send('Network.enable');

    let capturedPDF = null;
    let capturedURL = null;
    let pdfRequestId = null;

    const onResponse = async (params) => {
      const { response, requestId } = params;
      const ct = (response.headers['content-type'] || response.mimeType || '').toLowerCase();
      
      if (ct.includes('pdf') || response.url.match(/\.pdf/i)) {
        console.log(`       └─ 🎯 PDF detectado: ${response.url.slice(0, 100)}`);
        capturedPDF = response;
        capturedURL = response.url;
        pdfRequestId = requestId;
      }
    };

    cdp.on('Network.responseReceived', onResponse);

    // ─── Navegar al documento ───
    await page.goto(doc.url, { waitUntil: 'networkidle0', timeout: 30000 });
    await sleep(3000);

    cdp.removeListener('Network.responseReceived', onResponse);

    // ─── Buscar PDF en la página si no se capturó via CDP ───
    if (!capturedPDF) {
      const pdfInfo = await page.evaluate(() => {
        // Buscar iframe/embed/object
        const pdfSrc = [];
        document.querySelectorAll('iframe, embed, object').forEach(el => {
          const src = el.src || el.getAttribute('data') || '';
          if (src.match(/\.pdf/i)) pdfSrc.push(src);
        });
        // Buscar enlaces a PDFs
        document.querySelectorAll('a[href]').forEach(a => {
          if (a.href.match(/\.pdf/i)) pdfSrc.push(a.href);
          if (a.href.match(/\/download\//) || a.href.match(/\/media\//)) pdfSrc.push(a.href);
        });
        // Buscar en HTML
        const htmlMatches = document.body.innerHTML.match(/href="([^"]*\.pdf)"/gi) || [];
        htmlMatches.forEach(m => {
          const url = m.replace(/href="/i, '').replace(/"$/, '');
          if (url.startsWith('http') || url.startsWith('/')) pdfSrc.push(url);
        });
        return [...new Set(pdfSrc)];
      });

      for (const url of pdfInfo) {
        const fullUrl = url.startsWith('http') ? url : `https://www.cia.gov${url}`;
        console.log(`     └─ 📎 PDF link encontrado: ${fullUrl.slice(0, 100)}`);
        
        // Navegar directamente al PDF para que CDP lo capture
        cdp.on('Network.responseReceived', onResponse);
        try {
          await page.goto(fullUrl, { waitUntil: 'load', timeout: 30000 });
          await sleep(3000);
        } catch(e) {
          // Ignorar error de navegación (Timeout para algunos PDFs es normal)
        }
        cdp.removeListener('Network.responseReceived', onResponse);
        
        if (capturedPDF) break;
      }
    }

    // ─── Capturar el binario del PDF ───
    if (pdfRequestId) {
      try {
        const body = await cdp.send('Network.getResponseBody', { requestId: pdfRequestId });
        const buffer = Buffer.from(body.body, body.base64Encoded ? 'base64' : 'utf8');
        
        if (buffer.length > 500) {
          fs.writeFileSync(filepath, buffer);
          console.log(`     └─ ✅ ${Math.round(buffer.length/1024)} KB`);
          await cdp.send('Network.disable');
          return true;
        } else {
          console.log(`     └─ ⚠️ Buffer muy pequeño: ${buffer.length} bytes`);
        }
      } catch(e) {
        console.log(`     └─ ⚠️ Error getResponseBody: ${e.message.slice(0, 80)}`);
      }
    } else {
      // Fallback: page.pdf() - capturar pantalla como PDF
      console.log(`     └─ 📄 Sin PDF binario, capturando como HTML...`);
      try {
        await page.pdf({
          path: filepath,
          format: 'Letter',
          printBackground: true,
          margin: { top: '0.5in', right: '0.5in', bottom: '0.5in', left: '0.5in' },
        });
        const size = fs.statSync(filepath).size;
        if (size > 1000) {
          console.log(`     └─ ✅ ${Math.round(size/1024)} KB (PDF generado desde HTML)`);
          return true;
        }
      } catch(e) {
        console.log(`     └─ ❌ page.pdf() falló: ${e.message.slice(0, 80)}`);
      }
    }

    await cdp.send('Network.disable');
    return false;

  } catch (err) {
    console.log(`     └─ ❌ Error: ${err.message.slice(0, 100)}`);
    return false;
  }
}

async function main() {
  console.log('='.repeat(70));
  console.log('  🕵️ CIA FOIA — OPERACIÓN CÓNDOR EXTRACTOR V3');
  console.log('  📥 CDP Network Capture — captura binaria real');
  console.log('='.repeat(70));

  const browser = await puppeteer.launch({
    headless: true,
    executablePath: CHROME_PATH,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
  });

  const searchPage = await browser.newPage();
  await searchPage.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36');

  const allResults = {};
  let totalDownloaded = 0;
  let totalFound = 0;

  for (const q of QUERIES) {
    const searchDir = path.join(DOWNLOAD_DIR, q.name);
    fs.mkdirSync(searchDir, { recursive: true });

    try {
      const { documents, totalResults } = await extractDocNodes(searchPage, q.query);
      totalFound += documents.length;

      let downloaded = 0;
      for (let i = 0; i < Math.min(documents.length, q.max); i++) {
        const dlPage = await browser.newPage();
        await dlPage.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36');
        
        const ok = await captureDocPDF(dlPage, documents[i], searchDir, i + 1, Math.min(documents.length, q.max));
        if (ok) downloaded++;
        
        await dlPage.close();
        await sleep(1500);
      }

      totalDownloaded += downloaded;
      allResults[q.name] = {
        query: q.query, totalResults, documentsFound: documents.length, downloaded
      };
      console.log(`\n  📊 ${q.name}: ${downloaded}/${Math.min(documents.length, q.max)} PDFs\n`);

    } catch (err) {
      console.error(`\n  ❌ Error "${q.name}": ${err.message}`);
      allResults[q.name] = { query: q.query, error: err.message };
    }
  }

  // ─── Mostrar estructura de archivos descargados ───
  console.log('\n' + '='.repeat(70));
  console.log('  📊 ESTRUCTURA DE ARCHIVOS');
  console.log('='.repeat(70));
  for (const q of QUERIES) {
    const dir = path.join(DOWNLOAD_DIR, q.name);
    if (fs.existsSync(dir)) {
      const files = fs.readdirSync(dir).filter(f => f.endsWith('.pdf'));
      const totalSize = files.reduce((sum, f) => sum + (fs.statSync(path.join(dir, f)).size || 0), 0);
      console.log(`  📁 ${q.name}: ${files.length} PDFs, ${Math.round(totalSize/1024)} KB total`);
    }
  }
  console.log(`\n  📊 TOTAL: ${totalDownloaded}/${totalFound} PDFs`);
  console.log(`  📁 ${DOWNLOAD_DIR}`);

  const reportPath = path.join(REPORT_DIR, `cia_condor_v3_${Date.now()}.json`);
  fs.writeFileSync(reportPath, JSON.stringify(allResults, null, 2));
  console.log(`  💾 ${reportPath}`);

  await browser.close();
}

main().catch(err => {
  console.error('💥 FATAL:', err.message);
  process.exit(1);
});
