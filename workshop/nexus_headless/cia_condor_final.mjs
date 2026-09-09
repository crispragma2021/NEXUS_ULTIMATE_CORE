#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════════════════╗
 * ║  🕵️ CIA FOIA — OPERACIÓN CÓNDOR: EXTRACTOR FINAL v2            ║
 * ║  Home→cookies → Search → Doc → PDF fetch() en browser context   ║
 * ║  Sin Tor · Sin https.get() · fetch() dentro del browser         ║
 * ╚══════════════════════════════════════════════════════════════════╝
 *
 * ESTRUCTURA CONFIRMADA:
 *   Search:     /readingroom/search/site/{query}
 *   Document:   /readingroom/document/{node_id}
 *   PDF link:   /readingroom/docs/{FILENAME}[{node_id}].pdf
 *   Selector:   span.file a[href$=".pdf"]
 *   KEY FIX:    fetch() DENTRO del browser → lleva cookies de sesión
 *               https.get() desde Node.js NO lleva cookies → 0 bytes
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DOWNLOAD_DIR = path.join(__dirname, 'downloads', 'cia_condor_final');
const REPORT_DIR = path.join(__dirname, 'reports');
const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';

fs.mkdirSync(DOWNLOAD_DIR, { recursive: true });
fs.mkdirSync(REPORT_DIR, { recursive: true });

const sleep = ms => new Promise(r => setTimeout(r, ms));

// ─── QUERIES ──────────────────────────────────────────────────
const QUERIES = [
  { name: 'Operation_Condor',          query: 'operation condor',          max: 50 },
  { name: 'TESEO_CONDOR_1977',         query: 'TESEO CONDOR 1977',         max: 50 },
  { name: 'Paraguay_intelligence',     query: 'paraguay intelligence',     max: 50 },
  { name: 'Stroessner_CIA',            query: 'stroessner intelligence',   max: 50 },
  { name: 'Coronel_Paraguay',          query: 'coronel paraguay',          max: 50 },
  { name: 'Argentina_Dirty_War',       query: 'argentina dirty war condor',max: 50 },
  { name: 'Condor_Assassination',      query: '"Operation Condor" assassination', max: 50 },
  { name: 'Paraguay_CIA',              query: 'paraguay cia',              max: 50 },
];

/**
 * ─── CONSTANTES DE PAGINACIÓN ───
 * CIA FOIA muestra 10 resultados por página.
 * Usamos &page=N para navegar páginas adicionales.
 */
const RESULTS_PER_PAGE = 10;

/**
 * FASE 1: Buscar y extraer documentos (con paginación)
 * Usa una página que ya tiene cookies de sesión
 */
async function extractDocuments(page, searchQuery, maxDocs) {
  const baseUrl = `https://www.cia.gov/readingroom/search/site/${encodeURIComponent(searchQuery)}`;
  console.log(`  [🔍] "${searchQuery}"`);

  const allDocuments = [];
  let pageNum = 0;

  while (allDocuments.length < maxDocs) {
    pageNum++;
    const url = pageNum === 1 ? baseUrl : `${baseUrl}?page=${pageNum - 1}`;

    await page.goto(url, { waitUntil: 'networkidle0', timeout: 30000 });
    await sleep(2000);

    // Scroll para cargar resultados dinámicos
    for (let i = 0; i < 3; i++) {
      await page.evaluate(() => window.scrollBy(0, 1000));
      await sleep(800);
    }

    const result = await page.evaluate(() => {
      const docs = [];
      const seen = new Set();

      const ol = document.querySelector('ol.search-results');
      if (ol) {
        ol.querySelectorAll('li').forEach(li => {
          const h3 = li.querySelector('h3.title');
          const a = h3?.querySelector('a');
          const snippet = li.querySelector('.search-snippet-info p');
          const snippetText = snippet ? snippet.innerText.trim().slice(0, 200) : '';

          if (a && a.href.includes('/document/')) {
            const href = a.href;
            if (!seen.has(href)) {
              seen.add(href);
              docs.push({
                title: a.innerText.trim(),
                url: href,
                nodeId: href.match(/\/document\/(\d+)/)?.[1] || 'unknown',
                snippet: snippetText,
              });
            }
          }
        });
      }

      return {
        documents: docs,
        totalResults: (document.body.innerText.match(/Search found (\d+) items/) || [])[1] || '?',
      };
    });

    console.log(`     └─ Página ${pageNum}: ${result.documents.length} documentos (${result.totalResults} total)`);
    result.documents.slice(0, 3).forEach((d, i) =>
      console.log(`       ${i+1}. [${d.nodeId}] ${d.title.slice(0, 70)}`));

    // Evitar duplicados entre páginas
    const existingIds = new Set(allDocuments.map(d => d.nodeId));
    for (const doc of result.documents) {
      if (!existingIds.has(doc.nodeId) && allDocuments.length < maxDocs) {
        allDocuments.push(doc);
        existingIds.add(doc.nodeId);
      }
    }

    // Si la página actual tiene menos de RESULTS_PER_PAGE, no hay más páginas
    if (result.documents.length < RESULTS_PER_PAGE) break;
    // Safety: máx 10 páginas
    if (pageNum >= 10) break;

    await sleep(1000);
  }

  console.log(`     └─ Total: ${allDocuments.length} documentos únicos`);
  return allDocuments.slice(0, maxDocs);
}

/**
 * ─── DESCARGA DENTRO DEL BROWSER CONTEXT ───
 * Usa fetch() dentro de la página Puppeteer que YA TIENE las cookies de sesión.
 * Esto resuelve el problema de https.get() que producía PDFs de 0 bytes.
 */
async function downloadPDFviaBrowser(page, pdfUrl) {
  return await page.evaluate(async (url) => {
    try {
      const response = await fetch(url, {
        method: 'GET',
        credentials: 'include', // ← lleva las cookies de sesión
      });

      if (!response.ok) {
        return { error: `HTTP ${response.status}`, data: null };
      }

      const buffer = await response.arrayBuffer();
      const bytes = new Uint8Array(buffer);

      // Verificar que el PDF tiene cabecera válida
      if (bytes.length < 100 ||
          !(bytes[0] === 0x25 && bytes[1] === 0x50 && bytes[2] === 0x44 && bytes[3] === 0x46)) {
        return { error: `No es PDF válido (${bytes.length} bytes)`, data: null };
      }

      // Convertir a array plano para serialización
      return { error: null, data: Array.from(bytes) };
    } catch (err) {
      return { error: err.message, data: null };
    }
  }, pdfUrl);
}

/**
 * FASE 2+3: Visitar documento y descargar PDF
 * Extrae el link PDF de la página del documento y descarga
 * usando fetch() dentro del browser context
 */
async function downloadDocument(page, doc, searchDir, index, total) {
  const safeTitle = doc.title
    .replace(/[^a-zA-Z0-9_áéíóúñü\s-]/g, '').trim().slice(0, 60)
    .replace(/\s+/g, '_');
  const filename = `${String(index).padStart(3, '0')}_cia_${doc.nodeId}_${safeTitle}.pdf`;
  const filepath = path.join(searchDir, filename);

  if (fs.existsSync(filepath) && fs.statSync(filepath).size > 500) {
    console.log(`     └─ ⏭️ Ya existe (${Math.round(fs.statSync(filepath).size/1024)} KB)`);
    return true;
  }

  process.stdout.write(`  [📄] (${index}/${total}) ${doc.title.slice(0, 50)}... `);

  try {
    await page.goto(doc.url, { waitUntil: 'networkidle0', timeout: 30000 });
    await sleep(1500);

    // Extraer URL del PDF
    const pdfUrl = await page.evaluate(() => {
      // Selector exacto: span.file > a[href$=".pdf"]
      const fileLink = document.querySelector('span.file a[href$=".pdf"]');
      if (fileLink && fileLink.href) return fileLink.href;

      // Selector: table.sticky-enabled a[href$=".pdf"]
      const tableLink = document.querySelector('table.sticky-enabled a[href$=".pdf"]');
      if (tableLink && tableLink.href) return tableLink.href;

      // Selector: cualquier a[href$=".pdf"]
      const anyPdf = document.querySelector('a[href$=".pdf"]');
      if (anyPdf && anyPdf.href) return anyPdf.href;

      // Selector: .field-items a[href*="/docs/"]
      const docsLink = document.querySelector('.field-items a[href*="/docs/"]');
      if (docsLink && docsLink.href) return docsLink.href;

      return null;
    });

    if (!pdfUrl) {
      console.log(`❌ Sin PDF link`);
      return false;
    }

    // Descargar usando fetch() DENTRO del browser context (con cookies)
    const result = await downloadPDFviaBrowser(page, pdfUrl);

    if (result.error) {
      console.log(`⚠️ ${result.error}`);
      return false;
    }

    if (!result.data || result.data.length < 500) {
      console.log(`⚠️ PDF muy pequeño o vacío (${result.data?.length || 0} bytes)`);
      return false;
    }

    // Escribir a disco
    const buffer = Buffer.from(result.data);
    fs.writeFileSync(filepath, buffer);
    const sizeKB = Math.round(buffer.length / 1024);
    console.log(`✅ ${sizeKB} KB`);
    return true;

  } catch (err) {
    console.log(`❌ ${err.message.slice(0, 60)}`);
    return false;
  }
}

async function main() {
  console.log('='.repeat(70));
  console.log('  🕵️ CIA FOIA — OPERACIÓN CÓNDOR: EXTRACTOR FINAL v2');
  console.log('  📥 Home → cookies → Search → Doc → PDF (fetch en browser)');
  console.log('='.repeat(70));

  const browser = await puppeteer.launch({
    headless: true,
    executablePath: CHROME_PATH,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
  });

  // ─── FASE 0: Home para cookies ───
  console.log('\n[🌐] Obteniendo cookies de sesión...');
  const page = await browser.newPage();
  await page.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36');
  await page.goto('https://www.cia.gov/readingroom/', { waitUntil: 'networkidle0', timeout: 30000 });
  await sleep(2000);
  const cookies = await page.cookies();
  console.log(`     └─ ${cookies.length} cookies obtenidas: ${cookies.map(c => c.name).join(', ')}`);

  // ─── PROCESAR CADA QUERY ───
  const allResults = {};
  let totalDownloaded = 0;
  let totalFound = 0;

  for (const q of QUERIES) {
    const searchDir = path.join(DOWNLOAD_DIR, q.name);
    fs.mkdirSync(searchDir, { recursive: true });

    try {
      // FASE 1: Extraer documentos
      const documents = await extractDocuments(page, q.query, q.max);
      totalFound += documents.length;

      if (documents.length === 0) {
        console.log(`     └─ ⏭️ Sin documentos`);
        allResults[q.name] = { query: q.query, documentsFound: 0, downloaded: 0 };
        continue;
      }

      // FASE 2+3: Descargar cada documento
      let downloaded = 0;
      for (let i = 0; i < documents.length; i++) {
        const ok = await downloadDocument(page, documents[i], searchDir, i + 1, documents.length);
        if (ok) downloaded++;
        await sleep(600); // pausa corta entre docs
      }

      totalDownloaded += downloaded;
      allResults[q.name] = {
        query: q.query,
        documentsFound: documents.length,
        downloaded,
      };
      console.log(`\n  📊 ${q.name}: ${downloaded}/${documents.length} PDFs\n`);

    } catch (err) {
      console.error(`\n  ❌ Error "${q.name}": ${err.message}`);
      allResults[q.name] = { query: q.query, error: err.message };
    }
  }

  // ─── RESUMEN ───
  console.log('\n' + '='.repeat(70));
  console.log('  📊 RESUMEN FINAL');
  console.log('='.repeat(70));

  let totalPDFs = 0;
  let totalSizeKB = 0;

  for (const q of QUERIES) {
    const dir = path.join(DOWNLOAD_DIR, q.name);
    if (fs.existsSync(dir)) {
      const files = fs.readdirSync(dir).filter(f => f.endsWith('.pdf') && fs.statSync(path.join(dir, f)).size > 500);
      const size = files.reduce((sum, f) => sum + fs.statSync(path.join(dir, f)).size, 0);
      console.log(`  📁 ${q.name}: ${files.length} PDFs, ${Math.round(size/1024)} KB`);
      totalPDFs += files.length;
      totalSizeKB += Math.round(size/1024);
    }
  }

  console.log(`\n  📊 TOTAL: ${totalDownloaded}/${totalFound} descargados, ${totalPDFs} PDFs válidos, ${totalSizeKB} KB`);
  console.log(`  📁 ${DOWNLOAD_DIR}`);

  const reportPath = path.join(REPORT_DIR, `cia_condor_final_${Date.now()}.json`);
  fs.writeFileSync(reportPath, JSON.stringify(allResults, null, 2));
  console.log(`  💾 ${reportPath}`);

  await browser.close();
  console.log('\n✅ Misión completada');
}

main().catch(err => {
  console.error('\n💥 FATAL:', err.message);
  process.exit(1);
});
