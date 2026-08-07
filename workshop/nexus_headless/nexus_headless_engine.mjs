#!/usr/bin/env node
/**
 * ╔══════════════════════════════════════════════════════════════════╗
 * ║    🌐 NEXUS HEADLESS ENGINE v1.0                                ║
 * ║    Motor de scraping headless reutilizable con Puppeteer         ║
 * ║    Configurable vía JSON · Proxy nativo · Sobre Tor por defecto ║
 * ║                                                                  ║
 * ║  Uso: node nexus_headless_engine.mjs <config.json>               ║
 * ║                                                                  ║
 * ║  Ejemplo:                                                       ║
 * ║    node nexus_headless_engine.mjs configs/operacion_ejemplo.json ║
 * ╚══════════════════════════════════════════════════════════════════╝
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import crypto from 'crypto';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const BASE_DIR = path.resolve(__dirname, '..');
const OUTPUT_DIR = path.join(__dirname, 'reports');
const SCREENSHOTS_DIR = path.join(__dirname, 'reports', 'screenshots');

// ─── CONFIG BLOQUEADA ────────────────────────────────────────
const DEFAULT_CHROME_PATH = '/home/soberano/.cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome';
const DEFAULT_PROXY = 'socks5://127.0.0.1:9050';
const DEFAULT_USER_AGENT = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36';

// ═══════════════════════════════════════════════════════════════
//  UTILIDADES
// ═══════════════════════════════════════════════════════════════

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function now() {
  return new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
}

function safeFilename(s) {
  return s.replace(/[^a-zA-Z0-9_áéíóúñü\s-]/g, '').trim().replace(/\s+/g, '_').slice(0, 80);
}

// ═══════════════════════════════════════════════════════════════
//  BROWSER
// ═══════════════════════════════════════════════════════════════

async function launchBrowser(config) {
  const proxy = config.proxy || DEFAULT_PROXY;
  const chromePath = config.chromePath || DEFAULT_CHROME_PATH;
  
  console.log(`[🚀] Lanzando Chromium headless...`);
  console.log(`     Proxy: ${proxy}`);
  
  const args = [
    '--no-sandbox',
    '--disable-setuid-sandbox',
    '--disable-dev-shm-usage',
    '--disable-gpu',
    '--disable-web-security',
    '--disable-features=IsolateOrigins,site-per-process',
    '--window-size=1280,1024',
    '--disable-blink-features=AutomationControlled',
  ];

  if (config.useProxy !== false) {
    args.push(`--proxy-server=${proxy}`);
  }

  // Usar perfil persistente si está configurado
  if (config.userDataDir) {
    args.push(`--user-data-dir=${config.userDataDir}`);
    console.log(`     Perfil: ${config.userDataDir}`);
  }

  const browser = await puppeteer.launch({
    headless: config.headless !== false,
    executablePath: chromePath,
    args,
  });

  console.log(`[✅] Browser listo (PID: ${browser.process().pid})`);
  return { browser };
}

async function newPage(browser, config) {
  const page = await browser.newPage();
  await page.setUserAgent(config?.userAgent || DEFAULT_USER_AGENT);
  await page.setViewport({ width: 1280, height: 1024 });
  await page.evaluateOnNewDocument(() => {
    Object.defineProperty(navigator, 'webdriver', { get: () => false });
    Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
    Object.defineProperty(navigator, 'languages', { get: () => ['es-ES', 'es', 'en'] });
  });
  return page;
}

// ═══════════════════════════════════════════════════════════════
//  FUNCIONES DE BÚSQUEDA
// ═══════════════════════════════════════════════════════════════

async function searchGoogle(browser, query, config, timeout = 20000) {
  const page = await newPage(browser, config);
  console.log(`  [🌐] Google: "${query.slice(0, 80)}..."`);
  try {
    await page.goto(`https://www.google.com/search?q=${encodeURIComponent(query)}&hl=es`, {
      waitUntil: config.googleWaitUntil || 'networkidle0', 
      timeout: config.googleTimeout || timeout
    });
    await sleep(2000 + Math.random() * 1000);
    
    const results = await page.evaluate(() => {
      const items = [];
      // Selector moderno Google
      const searchDivs = document.querySelectorAll('div.g, div[data-hveid], div[data-sokoban-container]');
      searchDivs.forEach(div => {
        const link = div.querySelector('a[href^="http"]');
        const titleEl = div.querySelector('h3');
        const snippetEl = div.querySelector('.VwiC3b, .lEBKkf, span.aCOpRe, [data-sncf]');
        if (link && titleEl) {
          items.push({
            title: titleEl.innerText.trim().slice(0, 150),
            url: link.href,
            snippet: snippetEl ? snippetEl.innerText.trim().slice(0, 300) : '',
          });
        }
      });
      // Fallback: cualquier enlace externo
      if (items.length === 0) {
        document.querySelectorAll('a').forEach(a => {
          if (a.href.startsWith('http') && !a.href.includes('google.com') && a.innerText.trim()) {
            items.push({ title: a.innerText.trim().slice(0, 100), url: a.href, snippet: '' });
          }
        });
      }
      return items.slice(0, 20);
    });

    console.log(`     └─ ${results.length} resultados`);
    return results;
  } catch (err) {
    console.log(`     └─ ⚠️ ${err.message.slice(0, 80)}`);
    return [];
  }
}

async function searchDuckDuckGo(browser, query, config, timeout = 20000) {
  const page = await newPage(browser, config);
  console.log(`  [🦆] DuckDuckGo: "${query.slice(0, 80)}..."`);
  try {
    await page.goto(`https://lite.duckduckgo.com/lite/?q=${encodeURIComponent(query)}`, {
      waitUntil: 'networkidle0', timeout
    });
    await sleep(2000 + Math.random() * 1000);

    const results = await page.evaluate(() => {
      const items = [];
      document.querySelectorAll('a.result-link, a[href^="http"]').forEach(a => {
        if (a.href && !a.href.includes('duckduckgo.com/y.js')) {
          const snippet = a.closest('tr')?.querySelector('.result-snippet') || 
                          a.closest('tr')?.querySelector('td:last-child');
          items.push({
            title: a.innerText.trim().slice(0, 150),
            url: a.href,
            snippet: snippet ? snippet.innerText.trim().slice(0, 300) : '',
          });
        }
      });
      return items.slice(0, 20);
    });

    console.log(`     └─ ${results.length} resultados`);
    for (const r of results.slice(0, 3)) {
      console.log(`        ${r.title.slice(0, 80)}`);
    }
    return results;
  } catch (err) {
    console.log(`     └─ ⚠️ ${err.message.slice(0, 80)}`);
    return [];
  }
}

async function scrapePortal(browser, portalName, portalConfig, query, config, timeout = 25000) {
  const page = await newPage(browser, config);
  const searchUrl = typeof portalConfig.searchUrl === 'function'
    ? portalConfig.searchUrl(query)
    : `${portalConfig.baseUrl}${portalConfig.searchPath || '/search'}?q=${encodeURIComponent(query)}`;

  console.log(`  [🏛️] ${portalName}: "${query.slice(0, 60)}"`);
  console.log(`     ${searchUrl.slice(0, 120)}`);

  try {
    await page.goto(searchUrl, { waitUntil: portalConfig.waitUntil || 'networkidle0', timeout });
    await sleep(portalConfig.waitAfterLoad || 2000);

    // Extraer texto
    const pageText = await page.evaluate(() => document.body.innerText);

    // Scoring de menciones
    const queryWords = query.toLowerCase().split(' ').filter(w => w.length > 2);
    const lines = pageText.split('\n');
    const mentions = [];

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      if (line.length < 10) continue;
      const matchCount = queryWords.filter(w => line.toLowerCase().includes(w)).length;
      if (matchCount >= (portalConfig.minMatchWords || 1)) {
        const start = Math.max(0, i - 2);
        const end = Math.min(lines.length, i + 3);
        mentions.push({
          line: i,
          matchScore: queryWords.length > 0 ? matchCount / queryWords.length : 0,
          context: lines.slice(start, end).join(' | ').slice(0, 300),
        });
      }
    }
    mentions.sort((a, b) => b.matchScore - a.matchScore);

    // Extraer enlaces
    const links = await page.evaluate(() => {
      return Array.from(document.querySelectorAll('a[href]'))
        .map(a => ({ text: a.innerText.trim().slice(0, 100), href: a.href }))
        .filter(a => a.text.length > 5)
        .slice(0, 20);
    });

    // Screenshot
    const ssName = `${safeFilename(portalName)}_${Date.now()}.png`;
    const ssPath = path.join(SCREENSHOTS_DIR, ssName);
    await page.screenshot({ path: ssPath, fullPage: false });

    const result = {
      portal: portalName,
      query, url: searchUrl,
      mentionsCount: mentions.length,
      relevantMentions: mentions.slice(0, 5),
      links: links.slice(0, 10),
      pageTextPreview: pageText.slice(0, 800),
      screenshot: ssPath,
    };

    if (mentions.length > 0) {
      console.log(`     └─ ${mentions.length} menciones`);
    } else if (pageText.length < 100) {
      console.log(`     └─ Página vacía o bloqueada`);
    } else {
      console.log(`     └─ Sin menciones directas`);
    }
    return result;

  } catch (err) {
    console.log(`     └─ ⚠️ ${err.message.slice(0, 100)}`);
    return { portal: portalName, query, url: searchUrl, error: err.message, mentionsCount: 0, relevantMentions: [], links: [] };
  }
}

async function scrapeURL(browser, urlOrObj, label, config, timeout = 30000) {
  // Soporta tanto string plano como objeto con propiedades por URL
  const isObj = typeof urlOrObj === 'object' && urlOrObj !== null && urlOrObj.url;
  const url       = isObj ? urlOrObj.url                : urlOrObj;
  const urlTimeout = isObj ? (urlOrObj.timeout || config.urlTimeout || timeout) : (config.urlTimeout || timeout);
  const urlWait   = isObj ? (urlOrObj.waitUntil || config.urlWaitUntil || 'networkidle0') : (config.urlWaitUntil || 'networkidle0');
  const shouldScroll = isObj ? (urlOrObj.scroll !== false) : true;
  const noProxy   = isObj ? (urlOrObj.noProxy === true)  : false;

  // Si la URL requiere NO proxy (ej: sitios que bloquean Tor), lanzar browser temporal
  let ownBrowser = null;
  let targetBrowser = browser;
  if (noProxy) {
    console.log(`     └─ 🔓 Sin proxy (sitio bloquea Tor)`);
    const tempDir = `/tmp/nexus_no_proxy_${Date.now()}`;
    fs.mkdirSync(tempDir, { recursive: true });
    const tempConfig = { ...config, useProxy: false, userDataDir: tempDir };
    const launched = await launchBrowser(tempConfig);
    ownBrowser = launched.browser;
    targetBrowser = ownBrowser;
  }

  const page = await newPage(targetBrowser, config);
  console.log(`  [🔗] ${label || url.slice(0, 80)}`);
  try {
    await page.goto(url, {
      waitUntil: urlWait,
      timeout: urlTimeout
    });

    if (shouldScroll) {
      // Scroll gradual para cargar contenido dinámico
      await page.evaluate(async () => {
        const delay = ms => new Promise(r => setTimeout(r, ms));
        for (let i = 0; i < 5; i++) {
          window.scrollBy(0, window.innerHeight);
          await delay(800);
        }
      });
      await sleep(1000);
    } else {
      await sleep(2000);
    }

    const pageText = await page.evaluate(() => document.body.innerText);
    const links = await page.evaluate(() => {
      return Array.from(document.querySelectorAll('a[href]'))
        .map(a => ({ text: a.innerText.trim().slice(0, 100), href: a.href }))
        .filter(a => a.text.length > 5)
        .slice(0, 20);
    });

    const ssName = `url_${crypto.createHash('md5').update(url).digest('hex').slice(0, 8)}_${Date.now()}.png`;
    const ssPath = path.join(SCREENSHOTS_DIR, ssName);
    await page.screenshot({ path: ssPath, fullPage: false });

    console.log(`     └─ ${pageText.length} caracteres extraídos`);
    return { url, label, content: pageText.slice(0, 2000), links: links.slice(0, 10), screenshot: ssPath };
  } catch (err) {
    console.log(`     └─ ⚠️ ${err.message.slice(0, 100)}`);
    return { url, label, error: err.message };
  } finally {
    if (page) await page.close().catch(() => {});
    if (ownBrowser) await ownBrowser.close().catch(() => {});
  }
}

// ═══════════════════════════════════════════════════════════════
//  ORQUESTADOR PRINCIPAL
// ═══════════════════════════════════════════════════════════════

async function runOperation(config) {
  console.log(`\n${'='.repeat(60)}`);
  console.log(`🌐 NEXUS HEADLESS ENGINE v1.0`);
  console.log(`📋 Operación: ${config.name || 'Sin nombre'}`);
  console.log(`⏰ ${new Date().toISOString()}`);
  console.log(`${'='.repeat(60)}`);

  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });

  const { browser } = await launchBrowser(config);
  const allResults = {};
  let totalFindings = 0;

  try {
    for (const [targetName, targetData] of Object.entries(config.targets)) {
      console.log(`\n${'─'.repeat(50)}`);
      console.log(`[🎯] ${targetName}`);
      allResults[targetName] = {};
      const queries = targetData.queries || [targetName];

      for (const query of queries) {
        const resultKey = query.replace(/\s+/g, '_').slice(0, 40);
        allResults[targetName][resultKey] = {};

        // ─── FASE 1: BUSCADORES ──────────────────────────
        if (config.phases?.searchEngines !== false) {
          if (config.engines?.google !== false) {
            allResults[targetName][resultKey].google = await searchGoogle(browser, query, config);
          }
          if (config.engines?.duckduckgo !== false) {
            allResults[targetName][resultKey].duckduckgo = await searchDuckDuckGo(browser, query, config);
          }
        }

        // ─── FASE 2: PORTALES ────────────────────────────
        if (config.phases?.portals !== false && config.portals) {
          allResults[targetName][resultKey].portales = {};
          for (const [portalName, portalConfig] of Object.entries(config.portals)) {
            allResults[targetName][resultKey].portales[portalName] =
              await scrapePortal(browser, portalName, portalConfig, query, config);
          }
        }

        // ─── FASE 3: URLs DIRECTAS ──────────────────────
        if (config.phases?.directURLs !== false && targetData.urls) {
          allResults[targetName][resultKey].urls = {};
          for (const [label, urlEntry] of Object.entries(targetData.urls)) {
            allResults[targetName][resultKey].urls[label] = await scrapeURL(browser, urlEntry, label, config);
          }
        }
      }
    }

    // ─── REPORTE ──────────────────────────────────────────
    const ts = Date.now();
    const outputPath = path.join(OUTPUT_DIR, `${safeFilename(config.name || 'operacion')}_${ts}.json`);
    fs.writeFileSync(outputPath, JSON.stringify(allResults, null, 2));

    // Resumen
    console.log(`\n${'='.repeat(60)}`);
    console.log('📊 RESUMEN DE HALLAZGOS');
    console.log(`${'='.repeat(60)}`);

    for (const [targetName, targetData] of Object.entries(allResults)) {
      for (const [queryKey, data] of Object.entries(targetData)) {
        let targetTotals = 0;
        if (data.google) targetTotals += data.google.length;
        if (data.duckduckgo) targetTotals += data.duckduckgo.length;
        if (data.portales) {
          for (const p of Object.values(data.portales)) {
            if (p.mentionsCount > 0) targetTotals += p.mentionsCount;
          }
        }
        if (data.urls) {
          for (const [urlLabel, urlData] of Object.entries(data.urls)) {
            if (urlData.content) {
              targetTotals++;
              const preview = urlData.content.slice(0, 80).replace(/\n/g, ' ');
              console.log(`     └─ [📄] ${urlLabel}: ${urlData.content.length} chars → "${preview}..."`);
            } else if (urlData.error) {
              console.log(`     └─ [⚠️] ${urlLabel}: ${urlData.error.slice(0, 120)}`);
            }
          }
        }
        totalFindings += targetTotals;
        const queryDisplay = queryKey.replace(/_/g, ' ');
        console.log(`  📍 ${queryDisplay}: ${targetTotals} páginas cargadas`);
      }
    }

    console.log(`\n📊 Total: ${totalFindings} hallazgos`);
    console.log(`💾 Reporte: ${outputPath}`);
    return allResults;

  } catch (err) {
    console.error(`\n[❌] Error crítico: ${err.message}`);
    throw err;
  } finally {
    await browser.close();
    console.log('[✅] Chromium cerrado');
  }
}

// ═══════════════════════════════════════════════════════════════
//  CLI
// ═══════════════════════════════════════════════════════════════

async function main() {
  const configPath = process.argv[2];
  if (!configPath) {
    console.log(`\n🌐 NEXUS HEADLESS ENGINE v1.0`);
    console.log(`Uso: node nexus_headless_engine.mjs <config.json>\n`);
    console.log(`Ejemplo de config.json:`);
    console.log(JSON.stringify({
      name: "Investigación Ejemplo",
      proxy: "socks5://127.0.0.1:9050",
      phases: { searchEngines: true, portals: true, directURLs: true },
      engines: { google: true, duckduckgo: true },
      targets: {
        "Objetivo 1": {
          queries: ["Nombre Completo", "Apodo"],
          urls: { "Facebook": "https://facebook.com/..." }
        }
      },
      portals: {
        "Portal 1": { baseUrl: "https://ejemplo.gov.py", searchPath: "/buscar", minMatchWords: 1 }
      }
    }, null, 2));
    return;
  }

  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  
  // Merge con defaults
  config.phases = Object.assign({ searchEngines: true, portals: true, directURLs: true }, config.phases);
  config.engines = Object.assign({ google: true, duckduckgo: true }, config.engines);

  await runOperation(config);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
