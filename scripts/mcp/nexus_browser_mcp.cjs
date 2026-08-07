#!/usr/bin/env node
/**
 * NEXUS Browser MCP — Agente de Navegación Humana y Extracción Visual OMEGA
 *
 * Supera al navegador nativo de Antigravity incorporando:
 *   - Grabación de Video en tiempo real
 *   - Stealth Engine (Evasión de anti-bots / Cloudflare)
 *   - Digitación con comportamiento humano
 *   - Desplazamiento y Hover suave
 */

const { chromium } = require('playwright');
const { StealthEngine } = require('../../scripts/nexus_stealth_engine.cjs');
const { CaptchaBridge } = require('../../scripts/nexus_captcha_bridge.cjs');

const stealthEngine = new StealthEngine();
const captchaApiKey = process.env.CAPSOLVER_API_KEY || null;
const captchaBridge = captchaApiKey ? new CaptchaBridge(captchaApiKey) : null;
const readline = require('readline');
const path = require('path');
const fs = require('fs');

function send(obj) { process.stdout.write(JSON.stringify(obj) + '\n'); }
function error(id, msg) { send({ jsonrpc: '2.0', id, error: { code: -32000, message: msg } }); }
function ok(id, result) { send({ jsonrpc: '2.0', id, result }); }

let browser = null;
let context = null;
let page = null;

const OUTPUT_DIR = '/tmp/nexus_browser';
const SCREENSHOT_DIR = path.join(OUTPUT_DIR, 'screenshots');
const VIDEO_DIR = path.join(OUTPUT_DIR, 'videos');
const VAULT_DIR = path.join('/home/soberano/NEXUS_ULTIMATE_CORE/data/browser_vault');

if (!fs.existsSync(SCREENSHOT_DIR)) fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
if (!fs.existsSync(VIDEO_DIR)) fs.mkdirSync(VIDEO_DIR, { recursive: true });
if (!fs.existsSync(VAULT_DIR)) fs.mkdirSync(VAULT_DIR, { recursive: true });

async function ensureBrowser(recordVideo = false) {
  const launchOptions = stealthEngine.getLaunchOptions();

  // Evolución: Usar contexto persistente para mantener sesiones/cookies (Browser Vault)
  if (!context) {
    const contextOptions = {
      ...launchOptions,
      headless: true,
      args: [
        ...launchOptions.args,
        '--window-size=' + launchOptions.viewport.width + ',' + launchOptions.viewport.height
      ],
      viewport: launchOptions.viewport,
      userAgent: launchOptions.userAgent,
      locale: launchOptions.locale,
      timezoneId: launchOptions.timezoneId,
    };

    if (recordVideo) {
      contextOptions.recordVideo = { dir: VIDEO_DIR, size: launchOptions.viewport };
    }

    // launchPersistentContext combina launch y newContext, guardando estado en VAULT_DIR
    context = await chromium.launchPersistentContext(VAULT_DIR, contextOptions);
    
    // Inyección de Stealth en la carga de página (Bypass Anti-Bots)
    await context.addInitScript(stealthEngine.getInitScript());

    // En contexto persistente, una página ya viene abierta por defecto
    const pages = context.pages();
    page = pages.length > 0 ? pages[0] : await context.newPage();
  }

  if (!page || page.isClosed()) {
    page = await context.newPage();
  }

  return page;
}

const TOOLS = {
  browser_navigate: {
    description: 'Navega a una URL con simulación stealth de navegador real.',
    inputSchema: {
      type: 'object',
      properties: { 
        url: { type: 'string' },
        record: { type: 'boolean', description: 'Grabar video de la sesión (default: false)' }
      },
      required: ['url']
    }
  },
  browser_screenshot: {
    description: 'Captura de pantalla en alta resolución. Guarda en tmp y devuelve ruta.',
    inputSchema: {
      type: 'object',
      properties: {
        name: { type: 'string' },
        fullPage: { type: 'boolean' }
      }
    }
  },
  browser_click: {
    description: 'Hace clic en un elemento realizando un hover suave previo.',
    inputSchema: {
      type: 'object',
      properties: { selector: { type: 'string' } },
      required: ['selector']
    }
  },
  browser_type_human: {
    description: 'Escribe simulando digitación humana en un elemento (con delays aleatorios).',
    inputSchema: {
      type: 'object',
      properties: {
        selector: { type: 'string' },
        text: { type: 'string' }
      },
      required: ['selector', 'text']
    }
  },
  browser_hover: {
    description: 'Mueve el cursor del mouse de forma suave hacia un selector.',
    inputSchema: {
      type: 'object',
      properties: { selector: { type: 'string' } },
      required: ['selector']
    }
  },
  browser_scroll: {
    description: 'Realiza scroll suave hacia abajo o hacia un elemento específico.',
    inputSchema: {
      type: 'object',
      properties: {
        selector: { type: 'string', description: 'Selector opcional de destino' },
        distance: { type: 'number', description: 'Pixeles a desplazar si no se provee selector' }
      }
    }
  },
  browser_eval: {
    description: 'Ejecuta JS y retorna resultado.',
    inputSchema: {
      type: 'object',
      properties: { script: { type: 'string' } },
      required: ['script']
    }
  },
  browser_get_dom: {
    description: 'Devuelve la estructura HTML limpia del body.',
    inputSchema: { type: 'object', properties: {} }
  },
  browser_close: {
    description: 'Cierra la sesión del navegador y consolida grabaciones de video.',
    inputSchema: { type: 'object', properties: {} }
  },
  browser_resolve_captcha: {
    description: '🧬 Detecta y resuelve CAPTCHA en la página actual vía evasión biométrica + Capsolver API',
    inputSchema: {
      type: 'object',
      properties: {
        strategy: {
          type: 'string',
          description: 'Estrategia: evade_only | api_only | evade_then_api (default)',
          enum: ['evade_only', 'api_only', 'evade_then_api']
        }
      }
    }
  },
  browser_captcha_detect: {
    description: '🔍 Solo detecta CAPTCHA en página sin resolverlo',
    inputSchema: { type: 'object', properties: {} }
  },
  browser_captcha_balance: {
    description: '💰 Verifica saldo disponible en Capsolver',
    inputSchema: { type: 'object', properties: {} }
  }
};

async function handleTool(name, args) {
  const p = await ensureBrowser(args.record || false);

  switch (name) {
    case 'browser_navigate': {
      await p.goto(args.url, { waitUntil: 'networkidle', timeout: 20000 });
      const title = await p.title();
      let videoMsg = '';
      if (args.record) {
        const video = await p.video();
        if (video) {
          videoMsg = `\n🎥 Video grabándose activamente en el directorio de salida.`;
        }
      }
      return { content: [{ type: 'text', text: `Navegado exitosamente a: ${args.url}\nTítulo: ${title}${videoMsg}` }] };
    }
    
    case 'browser_screenshot': {
      const ts = Date.now();
      const filename = `${args.name || 'nexus_screen_' + ts}.png`;
      const filePath = path.join(SCREENSHOT_DIR, filename);
      await p.screenshot({ path: filePath, fullPage: args.fullPage || false });
      return { content: [{ type: 'text', text: `Imagen guardada en: ${filePath}` }] };
    }

    case 'browser_click': {
      // Usar StealthEngine v3: movimiento Perlin + delay decisión + hover + click
      await stealthEngine.clickBiometric(p, args.selector);
      return { content: [{ type: 'text', text: `Clic biométrico ejecutado en: ${args.selector}` }] };
    }

    case 'browser_type_human': {
      // Usar StealthEngine v3: click biométrico + escritura con distribución gaussiana + 2% error rate
      await stealthEngine.typeBiometric(p, args.selector, args.text);
      return { content: [{ type: 'text', text: `Texto biométrico inyectado en: ${args.selector}` }] };
    }

    case 'browser_hover': {
      // Usar StealthEngine v3: trayectoria Perlin hasta el elemento
      const hoverLocator = p.locator(args.selector).first();
      const box = await hoverLocator.boundingBox();
      if (box) {
        const cx = box.x + box.width / 2;
        const cy = box.y + box.height / 2;
        await stealthEngine.mouseMoveBiometric(p, cx, cy);
      }
      await hoverLocator.hover({ timeout: 5000 });
      return { content: [{ type: 'text', text: `Cursor posicionado sobre: ${args.selector}` }] };
    }

    case 'browser_scroll': {
      if (args.selector) {
        await p.locator(args.selector).first().scrollIntoViewIfNeeded({ timeout: 5000 });
        await p.waitForTimeout(150);
      } else {
        const dist = args.distance || 400;
        // Usar StealthEngine v3: scroll biométrico con curva ease-in-out
        await stealthEngine.scrollBiometric(p, dist);
      }
      return { content: [{ type: 'text', text: `Scroll biométrico completado.` }] };
    }

    case 'browser_eval': {
      const result = await p.evaluate(args.script);
      return { content: [{ type: 'text', text: `Resultado: ${JSON.stringify(result, null, 2)}` }] };
    }

    case 'browser_get_dom': {
      const html = await p.innerHTML('body');
      return { content: [{ type: 'text', text: html.slice(0, 50000) }] };
    }

    case 'browser_close': {
      let videoPath = null;
      if (page) {
        const video = await page.video();
        if (video) videoPath = await video.path();
      }
      if (context) await context.close();
      if (browser) await browser.close();
      
      browser = null; context = null; page = null;
      
      let videoMsg = '';
      if (videoPath && fs.existsSync(videoPath)) {
        const targetPath = path.join(VIDEO_DIR, path.basename(videoPath));
        fs.renameSync(videoPath, targetPath);
        videoMsg = `\n🎥 Video de interacción consolidado en: ${targetPath}`;
      }
      return { content: [{ type: 'text', text: `Navegador cerrado con éxito.${videoMsg}` }] };
    }

    case 'browser_captcha_detect': {
      if (!captchaBridge) {
        return { content: [{ type: 'text', text: '❌ CAPSOLVER_API_KEY no configurada. Usa: export CAPSOLVER_API_KEY=CAP-...' }] };
      }
      const detection = await captchaBridge.detectCaptcha(p);
      return { content: [{ type: 'text', text: detection.message }] };
    }

    case 'browser_captcha_balance': {
      if (!captchaBridge) {
        return { content: [{ type: 'text', text: '❌ CAPSOLVER_API_KEY no configurada' }] };
      }
      const result = await captchaBridge.checkBalance();
      return { content: [{ type: 'text', text: result.message }] };
    }

    case 'browser_resolve_captcha': {
      if (!captchaBridge) {
        return { content: [{ type: 'text', text: '❌ CAPSOLVER_API_KEY no configurada' }] };
      }

      const strategy = args.strategy || 'evade_then_api';
      const url = p.url();
      const startMs = Date.now();

      // FASE 1: Detectar
      const detection = await captchaBridge.detectCaptcha(p);
      
      if (!detection.captcha) {
        return { content: [{ type: 'text', text: detection.message }] };
      }

      let output = `🔍 CAPTCHA detectado: ${detection.captcha}\n`;

      // FASE 2: Evasión biométrica (si estrategia lo permite)
      if (strategy === 'evade_only' || strategy === 'evade_then_api') {
        output += `🔄 Intentando evasión biométrica...\n`;
        
        // Scroll biométrico + movimiento aleatorio
        await stealthEngine.scrollBiometric(p, 300);
        await p.waitForTimeout(200);
        
        // Mover mouse aleatoriamente para simular comportamiento
        const randomX = 100 + Math.floor(Math.random() * 500);
        const randomY = 100 + Math.floor(Math.random() * 300);
        await stealthEngine.mouseMoveBiometric(p, randomX, randomY);
        await p.waitForTimeout(300);

        // Re-detectar después de evasión
        const postEvasion = await captchaBridge.detectCaptcha(p);
        if (!postEvasion.captcha) {
          return { content: [{ type: 'text', text: output + '✅ Evasión biométrica exitosa — CAPTCHA desapareció' }] };
        }
        
        output += `⚠️ Evasión biométrica no fue suficiente, CAPTCHA persiste\n`;
        
        if (strategy === 'evade_only') {
          return { content: [{ type: 'text', text: output + '❌ Estrategia evade_only — no se pudo evadir' }] };
        }
      }

      // FASE 3: Resolución vía Capsolver
      if (strategy === 'api_only' || strategy === 'evade_then_api') {
        output += `🤖 Resolviendo con Capsolver...\n`;
        
        const result = await captchaBridge.resolveCaptcha(p, url);
        
        if (result.success) {
          output += `✅ CAPTCHA resuelto (${result.elapsed}ms)\n🔑 Token: ${result.token.slice(0, 40)}...`;
        } else {
          output += `❌ Falló: ${result.message}`;
        }
      }

      const totalTime = Date.now() - startMs;
      output += `\n⏱️ Tiempo total: ${totalTime}ms`;

      return { content: [{ type: 'text', text: output }] };
    }

    default:
      throw new Error(`Acción desconocida: ${name}`);
  }
}

// Loop de lectura estándar JSON-RPC
const rl = readline.createInterface({ input: process.stdin });
rl.on('line', async (line) => {
  if (!line.trim()) return;
  let msg;
  try { msg = JSON.parse(line); } catch { return; }
  
  const { id, method, params } = msg;

  try {
    if (method === 'initialize') {
      ok(id, {
        protocolVersion: '2024-11-05',
        capabilities: { tools: {} },
        serverInfo: { name: 'nexus-browser-stealth', version: '2.0.0' }
      });
    } else if (method === 'tools/list') {
      ok(id, {
        tools: Object.entries(TOOLS).map(([name, def]) => ({
          name, description: def.description, inputSchema: def.inputSchema
        }))
      });
    } else if (method === 'tools/call') {
      const result = await handleTool(params.name, params.arguments || {});
      ok(id, result);
    } else if (method === 'notifications/initialized') {
      // no-op
    } else {
      error(id, `Método desconocido: ${method}`);
    }
  } catch (e) {
    error(id, e.message);
  }
});
