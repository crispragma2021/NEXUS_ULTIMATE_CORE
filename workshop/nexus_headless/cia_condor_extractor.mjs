#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════════════════╗
 * ║  🕵️ CIA FOIA — OPERACIÓN CÓNDOR PDF EXTRACTOR                  ║
 * ║  Extrae links de documentos desclasificados y descarga PDFs     ║
 * ╚══════════════════════════════════════════════════════════════════╝
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import https from 'https';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR = path.join(__dirname, 'downloads', 'cia_condor');
const LOG_DIR = path.join(__dirname, 'reports');
const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';
const PROXY = 'socks5://127.0.0.1:9050';

fs.mkdirSync(OUTPUT_DIR, { recursive: true });
fs.mkdirSync(LOG_DIR, { recursive: true });

/**
 * Busca documentos en CIA FOIA y extrae los links de resultados
 */
async function extractDocumentLinks(browser, searchQuery) {
  const page = await browser.newPage();
  const url = `https://www.cia.gov/readingroom/search/site/${encodeURIComponent(searchQuery)}`;

  console.log(`\n[🔍] Buscando: "${searchQuery}"`);
  await page.goto(url, { waitUntil: 'networkidle0', timeout: 60000 });

  // Scroll para cargar más resultados (si hay paginación infinita)
  for (let i = 0; i < 3; i++) {
    await page.evaluate(() => window.scrollBy(0, 800));
    await new Promise(r => setTimeout(r, 1500));
  }

  // Extraer TODOS los links que parezcan documentos
  const docLinks = await page.evaluate(() => {
    // Buscar links que contengan documentos (patrón de CIA readingroom)
    const allLinks = Array.from(document.querySelectorAll('a[href*="/document/"]'));
    const results = allLinks.map(a => ({
      title: a.innerText.trim().slice(0, 200),
      url: a.href,
    })).filter(r => r.title.length > 5);

    // También buscar resultados de search que puedan tener documentos
    const searchItems = Array.from(document.querySelectorAll('.search-result, .views-row, .node, article, .teaser'));
    searchItems.forEach(item => {
      const link = item.querySelector('a[href]');
      const text = item.innerText.trim().slice(0, 300);
      if (link && text.length > 10) {
        // Verificar si ya existe
        if (!results.find(r => r.url === link.href)) {
          results.push({
            title: text.split('\n')[0].slice(0, 200),
            url: link.href,
          });
        }
      }
    });

    return results.slice(0, 50); // Top 50
  });

  console.log(`     └─ ${docLinks.length} documentos encontrados`);

  // Extraer también metadatos del contador
  const countText = await page.evaluate(() => {
    const el = document.querySelector('.search-results-count, .results-count, .summary');
    return el ? el.innerText : 'unknown';
  });
  console.log(`     └─ Total según página: ${countText}`);

  // Mostrar primeros 10
  docLinks.slice(0, 10).forEach((d, i) => {
    console.log(`       ${i + 1}. ${d.title.slice(0, 80)}`);
  });

  await page.close();
  return docLinks;
}

/**
 * Descarga un PDF individual
 */
function downloadPDF(url, filepath) {
  return new Promise((resolve) => {
    const file = fs.createWriteStream(filepath);
    const timeout = setTimeout(() => {
      file.close();
      resolve(false);
    }, 30000);

    https.get(url, { rejectUnauthorized: false }, (response) => {
      // Si redirige a documento real
      if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        clearTimeout(timeout);
        file.close();
        downloadPDF(response.headers.location, filepath).then(resolve);
        return;
      }

      const contentType = response.headers['content-type'] || '';
      if (contentType.includes('text/html') || response.statusCode !== 200) {
        clearTimeout(timeout);
        file.close();
        fs.unlinkSync(filepath);
        resolve(false);
        return;
      }

      response.pipe(file);
      file.on('finish', () => {
        clearTimeout(timeout);
        file.close();
        const stats = fs.statSync(filepath);
        if (stats.size > 1000) {
          resolve(true);
        } else {
          fs.unlinkSync(filepath);
          resolve(false);
        }
      });
    }).on('error', () => {
      clearTimeout(timeout);
      file.close();
      fs.unlinkSync(filepath);
      resolve(false);
    });
  });
}

/**
 * Navega a cada documento y descarga su PDF
 */
async function downloadDocuments(browser, docLinks, searchName) {
  const searchDir = path.join(OUTPUT_DIR, safeName(searchName));
  fs.mkdirSync(searchDir, { recursive: true });
  let downloaded = 0;

  for (let i = 0; i < Math.min(docLinks.length, 30); i++) {
    const doc = docLinks[i];
    console.log(`\n  [📄] (${i + 1}/${Math.min(docLinks.length, 30)}) ${doc.title.slice(0, 60)}`);
    
    try {
      const page = await browser.newPage();
      await page.goto(doc.url, { waitUntil: 'networkidle0', timeout: 30000 });
      await new Promise(r => setTimeout(r, 2000));

      // Buscar link de descarga PDF
      const pdfLink = await page.evaluate(() => {
        // Buscar múltiples patrones de enlaces de descarga
        const downloadBtn = document.querySelector('a[href$=".pdf"], a[href*="/download"], a[href*="document_file"], a.button, .download-link a, a[href*="media/"]');
        if (downloadBtn) return downloadBtn.href;

        // Buscar enlaces con texto "PDF" o "Download"
        const allLinks = Array.from(document.querySelectorAll('a'));
        const pdfAnchor = allLinks.find(a => 
          a.href.match(/\.pdf$/i) || 
          a.innerText.match(/pdf|download|descargar|document/i)
        );
        return pdfAnchor ? pdfAnchor.href : null;
      });

      if (pdfLink) {
        const pdfFilename = `cia_${safeName(doc.title.slice(0, 50))}.pdf`;
        const pdfPath = path.join(searchDir, pdfFilename);
        
        console.log(`       └─ 📥 Descargando PDF...`);
        const success = await downloadPDF(pdfLink, pdfPath);
        if (success) {
          const sizeKB = Math.round(fs.statSync(pdfPath).size / 1024);
          console.log(`       └─ ✅ ${sizeKB} KB`);
          downloaded++;
        } else {
          console.log(`       └─ ⚠️ No se pudo descargar (link: ${pdfLink.slice(0, 80)})`);
        }
      } else {
        console.log(`       └─ ❌ No se encontró link PDF en la página`);
      }

      await page.close();
    } catch (err) {
      console.log(`       └─ ❌ Error: ${err.message.slice(0, 80)}`);
    }

    // Pequeña pausa entre descargas
    await new Promise(r => setTimeout(r, 1000));
  }

  return downloaded;
}

function safeName(s) {
  return s.replace(/[^a-zA-Z0-9_áéíóúñü\s-]/g, '').trim().replace(/\s+/g, '_').slice(0, 60);
}

async function main() {
  console.log('='.repeat(70));
  console.log('  🕵️ CIA FOIA — OPERACIÓN CÓNDOR PDF EXTRACTOR');
  console.log('='.repeat(70));

  // Lanzar browser
  const browser = await puppeteer.launch({
    headless: false,
    executablePath: CHROME_PATH,
    args: [
      '--no-sandbox', '--disable-setuid-sandbox',
      `--proxy-server=${PROXY}`,
      '--window-size=1280,1024',
    ],
  });

  const queries = [
    { name: 'Operation Condor', query: 'operation condor' },
    { name: 'TESEO CONDOR 1977', query: 'TESEO CONDOR 1977' },
    { name: 'Paraguay Intelligence', query: 'paraguay intelligence' },
    { name: 'Stroessner CIA', query: 'stroessner intelligence' },
    { name: 'Coronel Paraguay', query: 'coronel paraguay' },
  ];

  const allResults = {};

  for (const q of queries) {
    try {
      const docLinks = await extractDocumentLinks(browser, q.query);
      allResults[q.name] = { query: q.query, count: docLinks.length, documents: docLinks };

      if (docLinks.length > 0) {
        console.log(`\n  ⬇️  Descargando PDFs para "${q.name}"...`);
        const dlCount = await downloadDocuments(browser, docLinks, q.name);
        console.log(`  ✅ ${dlCount} PDFs descargados para "${q.name}"`);
      }
    } catch (err) {
      console.error(`  ❌ Error en "${q.name}": ${err.message}`);
      allResults[q.name] = { query: q.query, error: err.message };
    }
  }

  // Guardar reporte
  const reportPath = path.join(LOG_DIR, `cia_condor_pdfs_${Date.now()}.json`);
  fs.writeFileSync(reportPath, JSON.stringify(allResults, null, 2));
  console.log(`\n💾 Reporte guardado: ${reportPath}`);

  await browser.close();

  // Resumen
  console.log('\n' + '='.repeat(70));
  console.log('  📊 RESUMEN FINAL');
  console.log('='.repeat(70));
  for (const [name, data] of Object.entries(allResults)) {
    const docs = data.count || 0;
    console.log(`  📍 ${name}: ${docs} documentos encontrados`);
    const dir = path.join(OUTPUT_DIR, safeName(name));
    if (fs.existsSync(dir)) {
      const files = fs.readdirSync(dir).filter(f => f.endsWith('.pdf'));
      console.log(`     └─ ${files.length} PDFs descargados en: ${dir}`);
    }
  }
  console.log(`\n📁 Todos los PDFs en: ${OUTPUT_DIR}`);
}

main().catch(console.error);
