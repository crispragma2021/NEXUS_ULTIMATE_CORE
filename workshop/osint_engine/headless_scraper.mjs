#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════════════════╗
 * ║  NEXUS OSINT ENGINE — Headless Scraper v1.0                     ║
 * ║  Puppeteer + Chromium para portales .gov.py con JavaScript      ║
 * ║  © 2026 NEXUS — Soberanía Técnica                               ║
 * ╚══════════════════════════════════════════════════════════════════╝
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const BASE_DIR = path.resolve(__dirname, '../..');
const OUTPUT_DIR = path.join(BASE_DIR, 'downloads', 'scraper_results');
const TOR_PROXY = 'socks5://127.0.0.1:9050';

// ─── CONFIG ────────────────────────────────────────────────────
const TARGETS = {
  'Aldo Francisco Coronel Torres': {
    personas: [
      'Aldo Francisco Coronel Torres',
      'Aldo Coronel Torres', 
      'Coronel Torres Aldo'
    ],
  },
  'Mauricio Cañete': {
    personas: [
      'Oscar Mauricio Cañete',
      'Mauricio Cañete',
    ],
  }
};

const PORTALS = {
  'Corte Suprema': {
    url: 'https://www.csj.gov.py/',
    searchUrl: (q) => `https://www.csj.gov.py/busqueda-causas?q=${encodeURIComponent(q)}`,
    selectors: {
      results: '.resultado, .causa-item, .row.result, table tr, [class*="result"]',
      title: 'h2, h3, h4, .titulo, .nombre, a',
    }
  },
  'PJ (Poder Judicial)': {
    url: 'https://www.pj.gov.py/',
    searchUrl: (q) => `https://www.pj.gov.py/busqueda?q=${encodeURIComponent(q)}`,
    selectors: {
      results: '.resultado, .item, article, .post, [class*="result"]',
    }
  },
  'Ministerio Público': {
    url: 'https://www.ministeriopublico.gov.py/',
    searchUrl: (q) => `https://www.ministeriopublico.gov.py/buscar?q=${encodeURIComponent(q)}`,
  },
  'SET (Registro Vehicular)': {
    url: 'https://www.set.gov.py/',
    searchUrl: (q) => `https://www.set.gov.py/portal/busqueda?q=${encodeURIComponent(q)}`,
  },
  'Registro Civil': {
    url: 'https://www.registrocivil.gov.py/',
    searchUrl: (q) => `https://www.registrocivil.gov.py/buscar?q=${encodeURIComponent(q)}`,
  },
  'CAMUS Blog': {
    url: 'https://camusclick.blogspot.com/',
    searchUrl: (q) => `https://camusclick.blogspot.com/search?q=${encodeURIComponent(q)}`,
  },
};


async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}


async function launchBrowser() {
  console.log('[🚀] Lanzando Chromium headless...');
  
  const browser = await puppeteer.launch({
    headless: true,
    executablePath: '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome',
    args: [
      '--no-sandbox',
      '--disable-setuid-sandbox',
      '--disable-dev-shm-usage',
      '--disable-gpu',
      '--disable-web-security',
      '--disable-features=IsolateOrigins,site-per-process',
      `--proxy-server=socks5://127.0.0.1:9050`,
      '--window-size=1280,1024',
    ],
  });
  
  console.log('[✅] Browser listo');
  return browser;
}


async function scrapePortal(page, portalName, portalConfig, query) {
  console.log(`\n  [🏛️] ${portalName}: "${query.slice(0,60)}"`);
  
  const url = portalConfig.searchUrl(query);
  console.log(`     URL: ${url.slice(0, 120)}`);
  
  try {
    await page.goto(url, { 
      waitUntil: 'networkidle0', 
      timeout: 25000 
    });
    
    // Esperar contenido dinámico
    await sleep(2000);
    
    // Obtener texto completo de la página
    const pageText = await page.evaluate(() => document.body.innerText);
    
    // Buscar menciones del query
    const mentions = [];
    const lines = pageText.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      if (line.length < 10) continue;
      
      const words = query.toLowerCase().split(' ');
      const matchCount = words.filter(w => line.toLowerCase().includes(w)).length;
      
      if (matchCount >= 1) {
        // Extraer contexto
        const start = Math.max(0, i - 2);
        const end = Math.min(lines.length, i + 3);
        const context = lines.slice(start, end).join(' | ').slice(0, 300);
        mentions.push({
          line: i,
          matchScore: matchCount / words.length,
          context: context,
        });
      }
    }
    
    // Ordenar por relevancia
    mentions.sort((a, b) => b.matchScore - a.matchScore);
    
    // Extraer enlaces
    const links = await page.evaluate(() => {
      return Array.from(document.querySelectorAll('a[href]'))
        .map(a => ({ text: a.innerText.trim().slice(0, 100), href: a.href }))
        .filter(a => a.text.length > 5 && a.href.length > 10)
        .slice(0, 15);
    });
    
    // Screenshot
    const screenshotPath = path.join(OUTPUT_DIR, `${portalName.replace(/\s+/g, '_')}_${Date.now()}.png`);
    await page.screenshot({ path: screenshotPath, fullPage: false });
    
    const result = {
      portal: portalName,
      query: query,
      url: url,
      mentionsCount: mentions.length,
      relevantMentions: mentions.slice(0, 5),
      links: links.slice(0, 10),
      pageTextPreview: pageText.slice(0, 1000),
      screenshot: screenshotPath,
    };
    
    if (mentions.length > 0) {
      console.log(`     └─ ${mentions.length} menciones encontradas`);
      for (const m of mentions.slice(0, 3)) {
        console.log(`        [${Math.round(m.matchScore*100)}%] ${m.context.slice(0, 100)}`);
      }
    } else {
      console.log(`     └─ Sin menciones directas`);
    }
    
    return result;
    
  } catch (err) {
    console.log(`     └─ [ERROR] ${err.message.slice(0, 100)}`);
    return {
      portal: portalName,
      query: query,
      url: url,
      error: err.message,
      mentionsCount: 0,
      relevantMentions: [],
      links: [],
    };
  }
}


async function searchDuckDuckGo(page, query) {
  console.log(`\n  [🌐] DuckDuckGo: "${query.slice(0,60)}..."`);
  
  try {
    await page.goto(`https://lite.duckduckgo.com/lite/?q=${encodeURIComponent(query)}`, {
      waitUntil: 'networkidle0',
      timeout: 20000
    });
    
    await sleep(2000);
    
    // Extraer resultados
    const results = await page.evaluate(() => {
      const items = [];
      const links = document.querySelectorAll('a.result-link');
      const snippets = document.querySelectorAll('.result-snippet');
      
      links.forEach((a, i) => {
        items.push({
          title: a.innerText.trim(),
          url: a.href,
          snippet: snippets[i] ? snippets[i].innerText.trim() : '',
        });
      });
      
      // Fallback: extraer cualquier enlace
      if (items.length === 0) {
        document.querySelectorAll('a').forEach(a => {
          if (a.href.startsWith('http') && !a.href.includes('duckduckgo.com')) {
            items.push({ title: a.innerText.trim(), url: a.href, snippet: '' });
          }
        });
      }
      
      return items.slice(0, 15);
    });
    
    console.log(`     └─ ${results.length} resultados`);
    for (const r of results.slice(0, 3)) {
      console.log(`        ${r.title.slice(0, 80)}`);
    }
    
    return results;
    
  } catch (err) {
    console.log(`     └─ [ERROR] ${err.message.slice(0, 80)}`);
    return [];
  }
}


async function searchSocialMedia(page, query) {
  console.log(`\n  [📱] Redes: "${query.slice(0,50)}..."`);
  
  const socialSites = [
    { name: 'Facebook', url: `https://www.facebook.com/search/top/?q=${encodeURIComponent(query)}` },
    { name: 'LinkedIn', url: `https://www.linkedin.com/search/results/all/?keywords=${encodeURIComponent(query)}` },
    { name: 'Instagram', url: `https://www.instagram.com/web/search/topsearch/?query=${encodeURIComponent(query)}` },
  ];
  
  const results = {};
  
  for (const site of socialSites) {
    try {
      await page.goto(site.url, { waitUntil: 'networkidle0', timeout: 15000 });
      await sleep(3000);
      
      const text = await page.evaluate(() => document.body.innerText);
      const hasContent = text.toLowerCase().includes(query.split(' ')[0].toLowerCase());
      
      results[site.name] = {
        url: site.url,
        hasContent: hasContent,
        preview: text.slice(0, 300),
      };
      
      console.log(`     ${site.name}: ${hasContent ? '✅ Contenido encontrado' : '❌ Sin acceso/bloqueado'}`);
      
    } catch (err) {
      results[site.name] = { url: site.url, error: err.message.slice(0, 80) };
      console.log(`     ${site.name}: ⚠️ Error`);
    }
  }
  
  return results;
}


async function searchGoogleViaTor(page, query) {
  console.log(`\n  [🌐] Google via Chromium: "${query.slice(0,50)}..."`);
  
  try {
    await page.goto(`https://www.google.com/search?q=${encodeURIComponent(query)}`, {
      waitUntil: 'networkidle0',
      timeout: 20000
    });
    
    await sleep(2000);
    
    // Extraer resultados
    const results = await page.evaluate(() => {
      const items = [];
      // Google moderno
      document.querySelectorAll('div.g, div[data-hveid]').forEach(div => {
        const link = div.querySelector('a[href^="http"]');
        const snippet = div.querySelector('.VwiC3b, .lEBKkf, span.aCOpRe');
        if (link) {
          items.push({
            title: link.innerText.trim().slice(0, 100),
            url: link.href,
            snippet: snippet ? snippet.innerText.trim().slice(0, 200) : '',
          });
        }
      });
      return items.slice(0, 15);
    });
    
    // Si el selector moderno no funciona, intentar extract todo texto
    if (results.length === 0) {
      const textResults = await page.evaluate(() => {
        const body = document.body.innerText;
        const links = Array.from(document.querySelectorAll('a'))
          .filter(a => a.href.startsWith('http') && !a.href.includes('google.com'))
          .map(a => ({ title: a.innerText.trim().slice(0, 80), url: a.href }));
        return links.slice(0, 15);
      });
      results.push(...textResults);
    }
    
    console.log(`     └─ ${results.length} resultados`);
    for (const r of results.slice(0, 3)) {
      console.log(`        ${r.title.slice(0, 80)}`);
    }
    
    return results;
    
  } catch (err) {
    console.log(`     └─ [ERROR] ${err.message.slice(0, 80)}`);
    return [];
  }
}


// ═══════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════
async function main() {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  
  console.log('╔══════════════════════════════════════════════════════════╗');
  console.log('║   NEXUS OSINT ENGINE — Headless Scraper v1.0            ║');
  console.log('║   Puppeteer + Chromium 148 · Portales Paraguay + Web    ║');
  console.log('╚══════════════════════════════════════════════════════════╝');
  
  let browser;
  try {
    browser = await launchBrowser();
    const page = await browser.newPage();
    
    // Configurar headers de navegador real
    await page.setUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36');
    await page.setViewport({ width: 1280, height: 1024 });
    
    const allResults = {};
    
    // ─── FASE 1: Google + DuckDuckGo ─────────────────────
    console.log(`\n${'='.repeat(60)}`);
    console.log('🔍 FASE 1: BÚSQUEDA EN BUSCADORES VÍA CHROMIUM');
    console.log(`${'='.repeat(60)}`);
    
    for (const [targetName, targetData] of Object.entries(TARGETS)) {
      console.log(`\n${'─'.repeat(50)}`);
      console.log(`[🎯] ${targetName}`);
      
      for (const persona of targetData.personas) {
        // Google
        const googleResults = await searchGoogleViaTor(page, persona);
        // DuckDuckGo
        const ddgResults = await searchDuckDuckGo(page, persona);
        // Redes
        const socialResults = await searchSocialMedia(page, persona);
        
        if (!allResults[targetName]) allResults[targetName] = {};
        allResults[targetName][persona] = {
          google: googleResults,
          duckduckgo: ddgResults,
          social: socialResults,
        };
      }
    }
    
    // ─── FASE 2: PORTALES .GOV.PY ────────────────────────
    console.log(`\n${'='.repeat(60)}`);
    console.log('🏛️ FASE 2: PORTALES PARAGUAY (.gov.py)');
    console.log(`${'='.repeat(60)}`);
    
    for (const [targetName, targetData] of Object.entries(TARGETS)) {
      for (const persona of targetData.personas) {
        console.log(`\n[🎯] ${persona}`);
        
        if (!allResults[targetName]) allResults[targetName] = {};
        if (!allResults[targetName][persona]) allResults[targetName][persona] = {};
        allResults[targetName][persona].portales = {};
        
        for (const [portalName, portalConfig] of Object.entries(PORTALS)) {
          const result = await scrapePortal(page, portalName, portalConfig, persona);
          allResults[targetName][persona].portales[portalName] = result;
        }
      }
    }
    
    // ─── FASE 3: CHAPA ───────────────────────────────────
    console.log(`\n${'='.repeat(60)}`);
    console.log('🚗 FASE 3: BÚSQUEDA DE CHAPA');
    console.log(`${'='.repeat(60)}`);
    
    // Leer candidatos del OCR
    const ocrReportPath = path.join(BASE_DIR, 'downloads', 'videos', 'chapa_extracts', 'ocr_analysis_report.json');
    let chapaCandidates = [];
    try {
      const ocrData = JSON.parse(fs.readFileSync(ocrReportPath, 'utf8'));
      chapaCandidates = Object.keys(ocrData.candidates || {});
    } catch {}
    
    // También buscar por patrón
    for (const candidate of chapaCandidates) {
      const clean = candidate.replace(/\s+/g, '');
      console.log(`\n[🚗] Buscando chapa: ${clean}`);
      
      const googleResults = await searchGoogleViaTor(page, `"${clean}" paraguay vehiculo automovil chapa`);
      const ddgResults = await searchDuckDuckGo(page, `"${clean}" paraguay automovil`);
      
      if (!allResults['chapas']) allResults['chapas'] = {};
      allResults['chapas'][clean] = { google: googleResults, duckduckgo: ddgResults };
    }
    
    // ─── REPORTE FINAL ───────────────────────────────────
    const timestamp = Date.now();
    const outputFile = path.join(OUTPUT_DIR, `headless_scraper_${timestamp}.json`);
    fs.writeFileSync(outputFile, JSON.stringify(allResults, null, 2));
    
    console.log(`\n${'='.repeat(60)}`);
    console.log('📊 REPORTE FINAL');
    console.log(`${'='.repeat(60)}\n`);
    
    let totalFindings = 0;
    
    for (const [target, targetData] of Object.entries(allResults)) {
      if (target === 'chapas') {
        console.log(`\n🚗 CHAPAS:`);
        for (const [chapa, data] of Object.entries(targetData)) {
          const total = (data.google?.length || 0) + (data.duckduckgo?.length || 0);
          console.log(`  ${chapa}: ${total} resultados`);
          totalFindings += total;
        }
        continue;
      }
      
      console.log(`\n🎯 ${target}:`);
      
      for (const [persona, data] of Object.entries(targetData)) {
        const googleCount = data.google?.length || 0;
        const ddgCount = data.duckduckgo?.length || 0;
        const portalCount = data.portales ? 
          Object.values(data.portales).filter(p => p.mentionsCount > 0).length : 0;
        
        console.log(`  📍 ${persona}`);
        console.log(`     Google: ${googleCount} | DDG: ${ddgCount} | Portales con datos: ${portalCount}`);
        totalFindings += googleCount + ddgCount + portalCount;
        
        // Detalles de portales
        if (data.portales) {
          for (const [portalName, portalResult] of Object.entries(data.portales)) {
            if (portalResult.mentionsCount > 0) {
              console.log(`     🏛️ ${portalName}: ${portalResult.mentionsCount} menciones`);
              for (const m of portalResult.relevantMentions.slice(0, 2)) {
                console.log(`        └─ ${m.context.slice(0, 120)}`);
              }
            } else if (portalResult.error) {
              console.log(`     🏛️ ${portalName}: ⚠️ ${portalResult.error.slice(0, 60)}`);
            } else {
              console.log(`     🏛️ ${portalName}: Sin resultados`);
            }
          }
        }
        
        // Detalles sociales
        if (data.social) {
          for (const [site, result] of Object.entries(data.social)) {
            if (result.hasContent) {
              console.log(`     📱 ${site}: ✅ Contenido encontrado`);
            }
          }
        }
      }
    }
    
    console.log(`\n📊 Total hallazgos: ${totalFindings}`);
    console.log(`\n[💾] Resultados: ${outputFile}`);
    
    return allResults;
    
  } catch (err) {
    console.error(`\n[❌] Error crítico: ${err.message}`);
    throw err;
  } finally {
    if (browser) await browser.close();
    console.log('\n[✅] Chromium cerrado');
  }
}


main().catch(err => {
  console.error(err);
  process.exit(1);
});
