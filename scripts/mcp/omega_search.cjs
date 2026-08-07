#!/usr/bin/env node
/**
 * OMEGA-SEARCH v3: Motor de Búsqueda Científica Multihilo para NEXUS
 * Optimizado para máxima calidad/relevancia y mínima navegación inútil.
 *
 * Cambios v3 (Soberanía Total — Cero API keys):
 *  - Scraping headless como ÚNICA fuente de búsqueda (Brave + fallback DuckDuckGo).
 *  - Eliminada por completo la dependencia de Brave API (sin barreras, sin claves).
 *  - Ranking por reputación de dominio + coincidencia de términos de la query.
 *  - Filtro de relevancia ANTES de la extracción profunda (ahorra ~80% de navegación).
 *  - Deduplicación de URLs normalizadas.
 *  - Concurrencia controlada en la extracción profunda (worker pool).
 *  - Extracción de metadatos (og:title, og:description) y snippets relevantes.
 */

const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');
const { StealthEngine } = require('../nexus_stealth_engine.cjs');
const proxyMesh = require('../nexus_proxy_mesh.cjs');

const stealthEngine = new StealthEngine();

const CACHE_DIR = '/tmp/nexus_omega_search_cache';
if (!fs.existsSync(CACHE_DIR)) fs.mkdirSync(CACHE_DIR, { recursive: true });

/**
 * Firewall de Inyección Indirecta: Sanitiza fragmentos extraídos para proteger al Orquestador.
 * Bloquea delimitadores de sistema y patrones de comando.
 */
function sanitizeText(text) {
  if (!text) return '';
  return text
    .replace(/\[\/?(SYSTEM|KERNEL|DIRECTIVE|USER|ASSISTANT|HUMAN|ROOT)\]/gi, '[REDACTED]')
    .replace(/(assistant|human|user|system):\s/gi, '[ROLE_INJECTION_PROTECT]: ')
    .replace(/\b(ignore all previous|forget instructions|new rules)\b/gi, '[INJECTION_ATTEMPT_BLOCKED]')
    .trim();
}

// ──────────────────────────────────────────────
// Reputación de dominios (peso de calidad de fuente)
// ──────────────────────────────────────────────
const DOMAIN_RANK = {
  'github.com': 1.0,
  'stackoverflow.com': 1.0,
  'docs.rs': 1.0,
  'users.rust-lang.org': 1.0,
  'rust-lang.org': 1.0,
  'crates.io': 0.9,
  'developer.mozilla.org': 0.9,
  'react.dev': 0.9,
  'vitejs.dev': 0.9,
  'nodejs.org': 0.9,
  'typescriptlang.org': 0.9,
  'stackexchange.com': 0.85,
  'medium.com': 0.6,
  'dev.to': 0.6,
  'blog.*': 0.5
};

function domainScore(url) {
  let host;
  try { host = new URL(url).hostname.replace(/^www\./, ''); } catch { return 0.3; }
  for (const [pat, score] of Object.entries(DOMAIN_RANK)) {
    if (pat.startsWith('blog.')) { if (host.startsWith('blog.')) return score; continue; }
    if (host === pat || host.endsWith('.' + pat)) return score;
  }
  // GitHub repos genéricos → score alto (usado en issues/PRs)
  if (host === 'github.com') return 1.0;
  return 0.5;
}

/**
 * Puntúa un resultado por relevancia respecto a la query.
 * Combina reputación de dominio + coincidencia de términos en título/snippet.
 */
function scoreResult(url, title, snippet, queryTerm) {
  const ds = domainScore(url);
  const terms = queryTerm.toLowerCase().split(/\s+/).filter(t => t.length > 2);
  const haystack = `${title} ${snippet || ''}`.toLowerCase();
  let matches = 0;
  for (const t of terms) if (haystack.includes(t)) matches++;
  const termRatio = terms.length > 0 ? matches / terms.length : 0;
  // Relevancia: 60% coincidencia de términos, 40% reputación de dominio
  return termRatio * 0.6 + ds * 0.4;
}

/** Normaliza una URL para deduplicación (quita fragmentos, tracking, trailing slash). */
function normalizeUrl(url) {
  try {
    const u = new URL(url);
    u.hash = '';
    u.search = '';
    let p = u.pathname.replace(/\/+$/, '');
    if (p === '') p = '/';
    return `${u.hostname.replace(/^www\./, '')}${p}`.toLowerCase();
  } catch { return url; }
}

/** Limpia y normaliza el término de búsqueda. */
function cleanQuery(q) {
  return encodeURIComponent(q.replace(/['"“”]/g, ''));
}

/**
 * Ejecuta tareas asíncronas con límite de concurrencia (worker pool).
 */
async function mapWithConcurrency(items, worker, concurrency = 4) {
  const results = new Array(items.length);
  let next = 0;
  async function runner() {
    while (next < items.length) {
      const idx = next++;
      results[idx] = await worker(items[idx], idx);
    }
  }
  const runners = Array.from({ length: Math.min(concurrency, items.length) }, runner);
  await Promise.all(runners);
  return results;
}

/**
 * Búsqueda científica profunda optimizada.
 */
async function performScientificSearch(queryTerm, limit = 5, useTor = false) {
  // --- Caché: evitar re-rastreos de queries recientes (TTL 30 min) ---
  const cacheKey = `${useTor ? 'tor:' : ''}${queryTerm.trim().toLowerCase()}`;
  const cacheFile = path.join(CACHE_DIR, `${Buffer.from(cacheKey).toString('base64url').slice(0, 64)}.json`);
  const cacheTTL = 30 * 60 * 1000;

  if (fs.existsSync(cacheFile)) {
    try {
      const cached = JSON.parse(fs.readFileSync(cacheFile, 'utf-8'));
      if (Date.now() - cached.ts < cacheTTL && cached.results) {
        console.log(`⚡ [Omega Search] Caché HIT para: "${queryTerm}"`);
        return cached.results;
      }
    } catch (e) { /* caché corrupto → re-buscar */ }
  }

  const launchOptions = stealthEngine.getLaunchOptions();
  let proxyConfig = null;
  if (useTor) {
    await proxyMesh.init();
    proxyConfig = await proxyMesh.getProxyConfig();
    if (!proxyConfig) console.warn('⚠️ [Omega Search] Sin proxy Tor. Procediendo directo.');
  }

  const browserConfig = { ...launchOptions };
  if (proxyConfig) browserConfig.proxy = proxyConfig;

  // Los resultados intermedios de búsqueda (antes de extracción profunda)
  let candidates = [];

  // ──────────────────────────────────────────────
  // FUENTE ÚNICA Y SOBERANA: scraping headless del buscador
  // Sin API keys. Cero dependencias externas de pago.
  // ──────────────────────────────────────────────
  const browser = await chromium.launch(browserConfig);
  const context = await browser.newContext({
    ...launchOptions,
    userAgent: launchOptions.userAgent,
    locale: launchOptions.locale,
    timezoneId: launchOptions.timezoneId
  });
  await context.addInitScript(stealthEngine.getInitScript());

  const scrapeBrave = async () => {
    console.log(`📡 [Omega] Brave headless scraping → "${queryTerm}"`);
    const page = await context.newPage();
    await page.route('**/*.{png,jpg,jpeg,gif,css,woff,woff2,svg,webp}', r => r.abort());
    if (useTor) await proxyMesh.humanDelay();
    await page.goto(`https://search.brave.com/search?q=${cleanQuery(queryTerm)}`, { waitUntil: 'domcontentloaded', timeout: 15000 });
    await page.waitForTimeout(2000);
    const links = await page.evaluate(() => {
      const res = [];
      document.querySelectorAll('.snippet, .result-wrapper').forEach(block => {
        const a = block.querySelector('a[href]');
        if (!a) return;
        const href = a.href;
        const title = (block.querySelector('.title, .search-snippet-title') || a).textContent.trim();
        const snippet = (block.querySelector('.snippet-description, .snippet-content, p')?.textContent || '').trim();
        if (href && href.startsWith('http') && !href.includes('brave.com') && title.length > 10) {
          res.push({ url: href, title, description: snippet || title });
        }
      });
      return res;
    });
    await page.close();
    return links;
  };

  // Fallback gratuito: DuckDuckGo HTML (sin JS, más tolerante a bloqueos)
  const scrapeDuckDuckGo = async () => {
    console.log(`📡 [Omega] DuckDuckGo headless scraping → "${queryTerm}"`);
    const page = await context.newPage();
    await page.route('**/*.{png,jpg,jpeg,gif,css,woff,woff2,svg,webp}', r => r.abort());
    if (useTor) await proxyMesh.humanDelay();
    await page.goto(`https://html.duckduckgo.com/html/?q=${cleanQuery(queryTerm)}`, { waitUntil: 'domcontentloaded', timeout: 15000 });
    await page.waitForTimeout(1500);
    const links = await page.evaluate(() => {
      const res = [];
      document.querySelectorAll('.result').forEach(block => {
        const a = block.querySelector('.result__a[href]');
        if (!a) return;
        const href = a.href;
        const title = a.textContent.trim();
        const snippet = (block.querySelector('.result__snippet')?.textContent || '').trim();
        if (href && href.startsWith('http') && title.length > 5) {
          res.push({ url: href, title, description: snippet || title });
        }
      });
      return res;
    });
    await page.close();
    return links;
  };

  try {
    let links = [];
    try {
      links = await scrapeBrave();
    } catch (e) {
      console.warn(`⚠️ [Omega] Brave scraping falló (${e.message}). Intentando DuckDuckGo...`);
    }
    if (links.length === 0) {
      try {
        links = await scrapeDuckDuckGo();
      } catch (e) {
        console.error(`❌ [Omega] DuckDuckGo scraping error: ${e.message}`);
      }
    }
    for (const l of links) {
      candidates.push({ ...l, score: scoreResult(l.url, l.title, l.description, queryTerm) });
    }
    console.log(`✅ [Omega] Scraping headless: ${links.length} candidatos.`);
  } finally {
    await context.close();
    await browser.close();
  }

  // ──────────────────────────────────────────────
  // Ranking + Deduplicación + Filtro de relevancia
  // ──────────────────────────────────────────────
  const seen = new Set();
  const ranked = [];
  for (const c of candidates) {
    const key = normalizeUrl(c.url);
    if (seen.has(key)) continue; // dedupe
    seen.add(key);
    ranked.push(c);
  }
  ranked.sort((a, b) => b.score - a.score);

  // Solo extraemos deep de resultados con relevancia aceptable (umbral 0.45)
  const relevant = ranked.filter(r => r.score >= 0.45).slice(0, limit);
  console.log(`🎯 [Omega] ${ranked.length} únicos → ${relevant.length} relevantes para extracción profunda.`);

  // ──────────────────────────────────────────────
  // Extracción profunda con concurrencia controlada
  // ──────────────────────────────────────────────
  if (relevant.length > 0) {
    const browser = await chromium.launch(browserConfig);
    const context = await browser.newContext({
      ...launchOptions,
      userAgent: launchOptions.userAgent,
      locale: launchOptions.locale,
      timezoneId: launchOptions.timezoneId
    });
    await context.addInitScript(stealthEngine.getInitScript());

    const extractedIntel = await mapWithConcurrency(relevant, async (target) => {
      const page = await context.newPage();
      await page.route('**/*.{png,jpg,jpeg,gif,css,woff,woff2,svg,webp}', r => r.abort());
      try {
        if (useTor) await proxyMesh.humanDelay();
        let intel = null;
        let lastErr = null;
        // Reducir timeout a 8s y quitar reintentos para evitar el timeout global de 60s del MCP
        try {
          await page.goto(target.url, { waitUntil: 'domcontentloaded', timeout: 8000 });
          intel = await page.evaluate((ctx) => {
              const { src, existingData, queryTerm } = ctx;
              const codeBlocks = [];
              document.querySelectorAll('pre code, div.highlight pre, pre').forEach(el => {
                const code = el.textContent.trim();
                if (code.length > 30 && code.length < 5000) codeBlocks.push(code);
              });

              const discussions = [];
              
              // Helper interno de sanitización (copia funcional para el contexto del navegador)
              const clean = (txt) => {
                if (!txt) return '';
                return txt.replace(/\[\/?(SYSTEM|KERNEL|DIRECTIVE|USER|ASSISTANT|HUMAN|ROOT)\]/gi, '[REDACTED]')
                          .replace(/(assistant|human|user|system):\s/gi, '[ROLE_INJECTION_PROTECT]: ')
                          .trim();
              };

              // Priorizar snippets existentes (de API) → contexto inmediato
              if (existingData && existingData.description && existingData.description.length > 3) {
                discussions.push(clean(existingData.description));
              }

              if (src.includes('GitHub')) {
                document.querySelectorAll('.comment-body, .markdown-body p').forEach(el => {
                  const t = el.textContent.trim();
                  // Poda por densidad: evitar párrafos con demasiados enlaces (menús)
                  if (el.querySelectorAll('a').length > 3) return;
                  if (t.length > 40 && t.length < 800) discussions.push(clean(t));
                });
              } else if (src.includes('StackOverflow') || src.includes('stackexchange')) {
                document.querySelectorAll('.js-post-body, .answercell p, .s-prose p').forEach(el => {
                  const t = el.textContent.trim();
                  if (t.length > 40 && t.length < 800) discussions.push(clean(t));
                });
              } else {
                // Poda heurística: densidad textual vs enlaces
                const terms = queryTerm.toLowerCase().split(/\s+/).filter(x => x.length > 3);
                document.querySelectorAll('p').forEach(el => {
                  const t = el.textContent.trim();
                  if (t.length < 60 || t.length > 800) return;
                  if (el.querySelectorAll('a').length > 2) return; // Probable menú o lista de tags
                  
                  const tl = t.toLowerCase();
                  if (terms.length > 0 && !terms.some(term => tl.includes(term))) return;
                  discussions.push(clean(t));
                });
              }

              // Metadatos sanitizados
              const meta = {
                ogTitle: clean(document.querySelector('meta[property="og:title"]')?.content || ''),
                ogDesc: clean(document.querySelector('meta[property="og:description"]')?.content || ''),
                h1: clean(document.querySelector('h1')?.textContent.trim() || '')
              };
              return { codeBlocks: codeBlocks.slice(0, 3), discussions: discussions.slice(0, 4), meta };
            }, { src: target.source || '', existingData: target, queryTerm });
          } catch (err) {
            lastErr = err;
          }
        if (intel === null) {
          return { url: target.url, source: target.source || 'Web', title: target.title, error: lastErr ? lastErr.message : 'extracción fallida' };
        }
        return { url: target.url, source: target.source || 'Web', title: target.title, score: target.score, data: intel };
      } catch (err) {
        return { url: target.url, source: target.source || 'Web', title: target.title, error: err.message };
      } finally {
        await page.close();
      }
    }, 6); // Aumentar concurrencia a 6 para procesar casi todos en paralelo

    await browser.close();

    // Cachear resultados con score
    try {
      fs.writeFileSync(cacheFile, JSON.stringify({ ts: Date.now(), results: extractedIntel }), 'utf-8');
      console.log(`💾 [Omega] Resultados cacheados para: "${queryTerm}"`);
    } catch (e) { /* caché no crítico */ }

    return extractedIntel;
  }

  // Sin resultados relevantes → devolver candidatos sin extraer (para no perder info)
  console.warn(`⚠️ [Omega] Ningún resultado superó el umbral de relevancia para "${queryTerm}".`);
  return ranked.slice(0, limit).map(r => ({
    url: r.url, source: 'Candidato (bajo score)', title: r.title,
    data: { codeBlocks: [], discussions: [r.description || r.title], meta: {} }
  }));
}

// Interfaz CLI para invocaciones independientes
if (require.main === module) {
  const torArgs = process.argv.slice(2).filter(a => a !== '--tor');
  const query = torArgs.join(' ');
  const useTor = process.argv.includes('--tor');

  if (!query) {
    console.error('Uso: node omega_search.cjs [--tor] "<término de búsqueda>"');
    process.exit(1);
  }

  console.log(`📡 [Omega v2] Búsqueda científica para: "${query}"${useTor ? ' (vía Tor)' : ''}...`);
  performScientificSearch(query, 5, useTor)
    .then(results => {
      console.log(JSON.stringify(results, null, 2));
      process.exit(0);
    })
    .catch(err => {
      console.error('Error en OMEGA-SEARCH:', err);
      process.exit(1);
    });
}

module.exports = { performScientificSearch };
