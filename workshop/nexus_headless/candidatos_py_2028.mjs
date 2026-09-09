#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════════════════╗
 * ║  🗳️ RECONOCIMIENTO ELECTORAL PARAGUAY 2028                     ║
 * ║  Payo Cubas + Candidatos fuertes a presidente                   ║
 * ╚══════════════════════════════════════════════════════════════════╝
 *
 * Modo: SOLO RECONOCIMIENTO — capturar snippets visibles, sin descargas
 * Fuentes: Wikipedia, medios, TSJE
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPORTS_DIR = path.join(__dirname, 'reports');
const CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';

fs.mkdirSync(REPORTS_DIR, { recursive: true });

const sleep = ms => new Promise(r => setTimeout(r, ms));

/**
 * Busca en una URL, captura el contenido visible relevante
 */
async function reconnaissance(page, label, url, extractFn) {
  console.log(`\n  [🌐] ${label}`);
  console.log(`       ${url}`);
  
  try {
    await page.goto(url, { waitUntil: 'networkidle0', timeout: 30000 });
    await sleep(2000);
    
    // Scroll para cargar contenido dinámico
    for (let i = 0; i < 3; i++) {
      await page.evaluate(() => window.scrollBy(0, 800));
      await sleep(500);
    }

    const result = await page.evaluate(extractFn);
    const preview = result.content ? result.content.slice(0, 300) : '(sin contenido)';
    console.log(`     └─ ${result.links?.length || 0} enlaces, ${result.content?.length || 0} chars`);
    console.log(`     └─ Preview: ${preview.replace(/\n/g, ' ').slice(0, 150)}...`);
    
    return result;
  } catch (err) {
    console.log(`     └─ ❌ Error: ${err.message.slice(0, 80)}`);
    return { error: err.message, content: '', links: [] };
  }
}

async function main() {
  console.log('='.repeat(70));
  console.log('  🗳️ RECONOCIMIENTO ELECTORAL — PARAGUAY 2028');
  console.log('  Payo Cubas y candidatos fuertes');
  console.log('='.repeat(70));

  const browser = await puppeteer.launch({
    headless: true,
    executablePath: CHROME_PATH,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
  });

  const results = {};
  const page = await browser.newPage();
  await page.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36');

  // ─── 1. Payo Cubas — Wikipedia ───
  results.payo_cubas = await reconnaissance(page, 
    'Payo Cubas — Wikipedia',
    'https://es.wikipedia.org/wiki/Paraguayo_Cubas',
    () => {
      const content = document.querySelector('.mw-content-ltr')?.innerText || '';
      const infobox = document.querySelector('.infobox')?.innerText || '';
      const links = Array.from(document.querySelectorAll('.mw-content-ltr a[href^="/wiki/"]'))
        .slice(0, 20)
        .map(a => ({ text: a.innerText.trim(), href: a.href }));
      return { 
        content: content.slice(0, 5000),
        infobox: infobox.slice(0, 1500),
        links 
      };
    }
  );

  // ─── 2. Elecciones generales Paraguay 2028 ───
  results.elecciones_generales = await reconnaissance(page,
    'Elecciones generales Paraguay 2028 — Wikipedia',
    'https://es.wikipedia.org/wiki/Elecciones_generales_de_Paraguay_de_2028',
    () => {
      const content = document.querySelector('.mw-content-ltr')?.innerText || '';
      const links = Array.from(document.querySelectorAll('.mw-content-ltr a[href^="/wiki/"]'))
        .slice(0, 30)
        .map(a => ({ text: a.innerText.trim(), href: a.href }));
      return { content: content.slice(0, 5000), links };
    }
  );

  // ─── 3. Elecciones Paraguay 2023 (histórico, para ver quiénes compitieron) ───
  results.elecciones_2023 = await reconnaissance(page,
    'Elecciones generales Paraguay 2023 — Wikipedia',
    'https://es.wikipedia.org/wiki/Elecciones_generales_de_Paraguay_de_2023',
    () => {
      const content = document.querySelector('.mw-content-ltr')?.innerText || '';
      const infobox = document.querySelector('.infobox')?.innerText || '';
      const links = Array.from(document.querySelectorAll('.mw-content-ltr a[href^="/wiki/"]'))
        .slice(0, 30)
        .map(a => ({ text: a.innerText.trim(), href: a.href }));
      return { content: content.slice(0, 5000), infobox: infobox.slice(0, 1500), links };
    }
  );

  // ─── 4. Buscar Payo Cubas 2028 noticias ───
  results.payo_noticias = await reconnaissance(page,
    'Payo Cubas 2028 — DuckDuckGo',
    'https://duckduckgo.com/?q=payo+cubas+2028+candidato+presidente+paraguay&ia=web',
    () => {
      const articles = Array.from(document.querySelectorAll('article[data-testid="result"]'));
      const results = articles.slice(0, 15).map(a => ({
        title: a.querySelector('h2')?.innerText || '',
        snippet: a.querySelector('.Ogdw')?.innerText || a.innerText.slice(0, 200),
        link: a.querySelector('a')?.href || '',
      }));
      return { content: results.map(r => `${r.title}: ${r.snippet}`).join('\n').slice(0, 5000), links: results.map(r => ({ text: r.title, href: r.link })) };
    }
  );

  // ─── 5. Candidatos presidenciales Paraguay 2028 ───
  results.candidatos_2028 = await reconnaissance(page,
    'Candidatos presidenciales Paraguay 2028 — DuckDuckGo',
    'https://duckduckgo.com/?q=candidatos+presidenciales+paraguay+2028&ia=web',
    () => {
      const articles = Array.from(document.querySelectorAll('article[data-testid="result"]'));
      const results = articles.slice(0, 15).map(a => ({
        title: a.querySelector('h2')?.innerText || '',
        snippet: a.querySelector('.Ogdw')?.innerText || a.innerText.slice(0, 200),
        link: a.querySelector('a')?.href || '',
      }));
      return { content: results.map(r => `${r.title}: ${r.snippet}`).join('\n').slice(0, 5000), links: results.map(r => ({ text: r.title, href: r.link })) };
    }
  );

  // ─── 6. Partidos políticos Paraguay ───
  results.partidos_politicos = await reconnaissance(page,
    'Partidos políticos Paraguay — Wikipedia',
    'https://es.wikipedia.org/wiki/Partidos_pol%C3%ADticos_de_Paraguay',
    () => {
      const content = document.querySelector('.mw-content-ltr')?.innerText || '';
      const links = Array.from(document.querySelectorAll('.mw-content-ltr a[href^="/wiki/"]'))
        .slice(0, 30)
        .map(a => ({ text: a.innerText.trim(), href: a.href }));
      return { content: content.slice(0, 5000), links };
    }
  );

  // ─── 7. Buscar "precandidatos" Paraguay 2027 2028 ───
  results.precandidatos = await reconnaissance(page,
    'Precandidatos Paraguay 2028 — DuckDuckGo',
    'https://duckduckgo.com/?q=precandidatos+presidenciales+paraguay+2028&ia=web',
    () => {
      const articles = Array.from(document.querySelectorAll('article[data-testid="result"]'));
      const results = articles.slice(0, 15).map(a => ({
        title: a.querySelector('h2')?.innerText || '',
        snippet: a.querySelector('.Ogdw')?.innerText || a.innerText.slice(0, 200),
        link: a.querySelector('a')?.href || '',
      }));
      return { content: results.map(r => `${r.title}: ${r.snippet}`).join('\n').slice(0, 5000), links: results.map(r => ({ text: r.title, href: r.link })) };
    }
  );

  // ─── 8. TSJE - Candidatos ───
  results.tsje = await reconnaissance(page,
    'TSJE — Tribunal Superior de Justicia Electoral',
    'https://tsje.gov.py/',
    () => {
      const content = document.body?.innerText?.slice(0, 5000) || '';
      const links = Array.from(document.querySelectorAll('a[href]'))
        .slice(0, 30)
        .map(a => ({ text: a.innerText.trim(), href: a.href }));
      return { content, links };
    }
  );

  // ─── 9. Buscar encuestas electorales Paraguay 2028 ───
  results.encuestas = await reconnaissance(page,
    'Encuestas electorales Paraguay 2028 — DuckDuckGo',
    'https://duckduckgo.com/?q=encuestas+electorales+paraguay+2028+presidente&ia=web',
    () => {
      const articles = Array.from(document.querySelectorAll('article[data-testid="result"]'));
      const results = articles.slice(0, 15).map(a => ({
        title: a.querySelector('h2')?.innerText || '',
        snippet: a.querySelector('.Ogdw')?.innerText || a.innerText.slice(0, 200),
        link: a.querySelector('a')?.href || '',
      }));
      return { content: results.map(r => `${r.title}: ${r.snippet}`).join('\n').slice(0, 5000), links: results.map(r => ({ text: r.title, href: r.link })) };
    }
  );

  // ─── 10. Santiago Peña reelección? ───
  results.pena_reeleccion = await reconnaissance(page,
    'Santiago Peña reelección 2028 — DuckDuckGo',
    'https://duckduckgo.com/?q=santiago+pe%C3%B1a+reelecci%C3%B3n+2028+paraguay&ia=web',
    () => {
      const articles = Array.from(document.querySelectorAll('article[data-testid="result"]'));
      const results = articles.slice(0, 10).map(a => ({
        title: a.querySelector('h2')?.innerText || '',
        snippet: a.querySelector('.Ogdw')?.innerText || a.innerText.slice(0, 200),
        link: a.querySelector('a')?.href || '',
      }));
      return { content: results.map(r => `${r.title}: ${r.snippet}`).join('\n').slice(0, 5000), links: results.map(r => ({ text: r.title, href: r.link })) };
    }
  );

  // ─── GUARDAR REPORTE ───
  const timestamp = Date.now();
  const reportPath = path.join(REPORTS_DIR, `candidatos_py_2028_${timestamp}.json`);
  fs.writeFileSync(reportPath, JSON.stringify(results, null, 2));
  
  console.log('\n' + '='.repeat(70));
  console.log('  📊 RECONOCIMIENTO COMPLETADO');
  console.log('='.repeat(70));
  console.log(`  💾 ${reportPath}`);

  await browser.close();
}

main().catch(err => {
  console.error('💥 FATAL:', err.message);
  process.exit(1);
});
