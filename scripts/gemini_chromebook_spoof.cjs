/**
 * 🚀 NEXUS — Gemini Chromebook Perk Spoofer v4.0
 * Modo: Asistencia manual para ejecución desde navegador del usuario
 * 
 * INSTRUCCIONES:
 * ─────────────────────────────────────────────────────────────
 * 1. ABRE Firefox/Chrome CON proxy SOCKS5 activo:
 *    - Opción A (Tor): `./scripts/tor_on.sh` → proxy en 127.0.0.1:9050
 *    - Opción B (Residential): Configura proxy manual en el sistema
 * 
 * 2. PEGA en la barra de direcciones:
 *    https://www.google.com/chromebook/perks/?hl=en&gl=us&pli=1
 * 
 * 3. ABRE DevTools (F12 → Console) y PEGA el script de abajo
 * 
 * 4. SIGUE las instrucciones en pantalla
 * ─────────────────────────────────────────────────────────────
 */

const CHROMIUM_PATH = '/usr/bin/chromium';
const PERKS_URL = 'https://www.google.com/chromebook/perks/?hl=en&gl=us&pli=1';

const USER_AGENT_CROS = 'Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36';

// ============================================================
// PASO 1: Script para CONSOLA DEL NAVEGADOR (Pegar en F12)
// ============================================================
const CONSOLE_SCRIPT = `
// ================================================================
// 🧬 NEXUS EMULACIÓN CHROMEBOOK PLUS — PEGAR EN CONSOLA F12
// ================================================================

(async () => {
    console.log('🔧 NEXUS: Iniciando emulación Chromebook Plus...');

    // --- SPOOFING AGRESIVO ---
    Object.defineProperty(navigator, 'webdriver', { get: () => false });
    Object.defineProperty(navigator, 'platform', { get: () => 'X11; CrOS x86_64 14541.0.0' });
    Object.defineProperty(navigator, 'userAgent', { get: () => '${USER_AGENT_CROS}' });
    Object.defineProperty(navigator, 'vendor', { get: () => 'Google Inc.' });
    Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 8 });
    Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });
    window.chrome = { runtime: {}, loadTimes: () => ({}), csi: () => ({}) };

    console.log('✅ Spoofing CrOS activado. UserAgent:', navigator.userAgent);
    console.log('✅ Platform:', navigator.platform);

    // --- SCROLL CARGAR PERKS ---
    console.log('📜 Scroll progresivo para cargar perks...');
    for (let i = 0; i < 8; i++) {
        window.scrollBy(0, 1200);
        await new Promise(r => setTimeout(r, 600));
    }
    // Volver al inicio
    window.scrollTo({ top: 0, behavior: 'smooth' });
    await new Promise(r => setTimeout(r, 1000));

    // --- EXTRAER TODOS LOS ELEMENTOS CLICABLES ---
    const perks = [];
    document.querySelectorAll('a, button, [role="button"], [data-get-perk], [data-perk-id], [class*="perk"], [class*="offer"]').forEach(el => {
        const rect = el.getBoundingClientRect();
        const text = el.innerText?.trim() || '';
        const dataGetPerk = el.getAttribute('data-get-perk') || '';
        const dataPerkId = el.getAttribute('data-perk-id') || '';
        const href = el.getAttribute('href') || '';
        const ariaLabel = el.getAttribute('aria-label') || '';
        
        // Solo elementos con tamaño visible
        if (rect.width > 10 && rect.height > 10) {
            perks.push({
                tag: el.tagName,
                text: text.substring(0, 120),
                dataGetPerk,
                dataPerkId,
                href,
                ariaLabel,
                className: el.className?.substring(0, 80) || '',
                rect: {
                    top: Math.round(rect.top),
                    left: Math.round(rect.left),
                    width: Math.round(rect.width),
                    height: Math.round(rect.height)
                },
                // Indicar si está en viewport
                inViewport: rect.top < window.innerHeight && rect.bottom > 0
            });
        }
    });

    console.log('📋 PERKS ENCONTRADOS (' + perks.length + ' total):');
    console.table(perks.filter(p => 
        p.text.toLowerCase().includes('beneficio') || 
        p.text.toLowerCase().includes('gemini') || 
        p.text.toLowerCase().includes('google one') ||
        p.text.toLowerCase().includes('ai premium') ||
        p.text.toLowerCase().includes('chromebook') ||
        p.dataGetPerk ||
        p.ariaLabel.toLowerCase().includes('gemini') ||
        p.ariaLabel.toLowerCase().includes('perk')
    ));

    // --- BUSCAR Y DESTACAR OBJETIVO ---
    const objetivo = perks.find(p => 
        p.dataGetPerk === 'gamgee.2024' ||
        p.text.includes('Obtener beneficio') ||
        (p.text.includes('Google') && p.text.includes('AI') && p.text.includes('Pro')) ||
        (p.text.includes('Gemini') && p.text.includes('Advanced'))
    );

    if (objetivo) {
        console.log('🎯 OBJETIVO ENCONTRADO:', objetivo);
        console.log('📍 Posición:', objetivo.rect);
        
        // Destacar visualmente el botón
        const elementos = document.querySelectorAll('[data-get-perk="gamgee.2024"], a, button, [role="button"]');
        elementos.forEach(el => {
            if (el.innerText?.includes('Obtener beneficio') || el.getAttribute('data-get-perk') === 'gamgee.2024') {
                el.style.border = '4px solid red';
                el.style.boxShadow = '0 0 20px rgba(255,0,0,0.7)';
                el.style.transform = 'scale(1.05)';
                el.scrollIntoView({ behavior: 'smooth', block: 'center' });
                console.log('🔴 Elemento destacado:', el);
            }
        });
        
        console.log('👇 HAZ CLICK MANUALMENTE EN EL ELEMENTO DESTACADO EN ROJO');
        console.log('⚠️ Si no ves el botón, haz scroll manual cerca de la posición:', objetivo.rect);
    } else {
        console.log('❌ No se encontró el objetivo automáticamente.');
        console.log('📝 Busca manualmente entre los perks listados arriba.');
        console.log('📌 Busca: "Google AI Pro" o "Gemini Advanced" o "Obtener beneficio"');
        
        // Intentar con XPath como fallback
        const xpathResult = document.evaluate(
            '//*[contains(text(), "Obtener beneficio") or contains(text(), "Gemini") or contains(@data-get-perk, "gamgee")]',
            document,
            null,
            XPathResult.ORDERED_NODE_SNAPSHOT_TYPE,
            null
        );
        console.log('🔍 XPath fallback: ' + xpathResult.snapshotLength + ' resultados');
        for (let i = 0; i < xpathResult.snapshotLength; i++) {
            const node = xpathResult.snapshotItem(i);
            console.log('  →', node?.tagName, node?.innerText?.substring(0, 100));
        }
    }

    console.log('✅ Script de emulación completado.');
    console.log('👉 Próximo paso: Resolver CAPTCHA manualmente si aparece.');
})();
`;

// ============================================================
// PASO 2: Script para Node.js (Automatización completa con Playwright)
// ============================================================
async function runNodeScript() {
    console.log(`
╔══════════════════════════════════════════════════════════════╗
║     🚀 NEXUS — Gemini Chromebook Perk Exploit v4          ║
║     Modo: Automatización Asistida (Node.js + Playwright)   ║
╚══════════════════════════════════════════════════════════════╝
    `);

    let browser;
    try {
        const { chromium } = require('playwright');
        
        console.log('🔧 Lanzando Chromium con proxy SOCKS5 y spoofing CrOS...');
        browser = await chromium.launch({
            headless: false,  // MODO VISIBLE para que veas lo que pasa
            args: [
                '--disable-blink-features=AutomationControlled',
                '--no-sandbox',
                '--disable-infobars',
                '--proxy-server=socks5://127.0.0.1:9050'
            ]
        });

        const context = await browser.newContext({
            userAgent: USER_AGENT_CROS,
            viewport: { width: 1920, height: 1080 },
            deviceScaleFactor: 1,
            isMobile: false,
            hasTouch: false,
            locale: 'en-US',
            timezoneId: 'America/New_York',
            geolocation: { latitude: 40.7128, longitude: -74.0060 },
            permissions: ['geolocation'],
        });

        // Spoofing InitScript
        await context.addInitScript(() => {
            Object.defineProperty(navigator, 'webdriver', { get: () => false });
            Object.defineProperty(navigator, 'platform', { get: () => 'X11; CrOS x86_64 14541.0.0' });
            Object.defineProperty(navigator, 'userAgent', { 
                get: () => 'Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36' 
            });
            Object.defineProperty(navigator, 'vendor', { get: () => 'Google Inc.' });
            Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 8 });
            window.chrome = { runtime: {}, loadTimes: () => ({}), csi: () => ({}) };
            
            // Spoofear WebGL para CrOS
            const getParameter = WebGLRenderingContext.prototype.getParameter;
            WebGLRenderingContext.prototype.getParameter = function(parameter) {
                if (parameter === 37445) return 'Google Inc.';
                if (parameter === 37446) return 'ANGLE (Intel, Intel(R) HD Graphics 615 Direct3D11 vs_5_0, D3D11)';
                return getParameter(parameter);
            };
        });

        const page = await context.newPage();

        // Monitorear requests para debug
        page.on('request', req => {
            if (req.url().includes('perk') || req.url().includes('gamgee') || req.url().includes('googleone')) {
                console.log('🌐 Request:', req.method(), req.url());
            }
        });
        
        page.on('response', res => {
            if (res.url().includes('perk') || res.url().includes('gamgee') || res.url().includes('googleone')) {
                console.log('📥 Response:', res.status(), res.url());
            }
        });

        console.log(`📡 Navegando a ${PERKS_URL}...`);
        await page.goto(PERKS_URL, { waitUntil: 'networkidle', timeout: 90000 });
        console.log('✅ Página cargada. URL actual:', page.url());

        // Guardar DOM completo
        const html = await page.content();
        require('fs').writeFileSync('/tmp/gemini_perks_dump.html', html);
        console.log('📄 DOM guardado en /tmp/gemini_perks_dump.html');

        // Scroll progresivo para cargar perks
        console.log('📜 Scroll progresivo...');
        await page.evaluate(async () => {
            const delay = ms => new Promise(r => setTimeout(r, ms));
            for (let i = 0; i < 8; i++) {
                window.scrollBy(0, 1000);
                await delay(500);
            }
        });
        await page.waitForTimeout(2000);

        // Captura visual
        await page.screenshot({ path: '/tmp/gemini_perks_init.png', fullPage: true });
        console.log('📸 Screenshot guardado en /tmp/gemini_perks_init.png');

        // Extraer perks
        const perks = await page.evaluate(() => {
            return Array.from(document.querySelectorAll('a, button, [role="button"], [data-get-perk], [data-perk-id]'))
                .map(el => ({
                    tag: el.tagName,
                    text: el.innerText?.trim().substring(0, 150) || '',
                    dataGetPerk: el.getAttribute('data-get-perk') || '',
                    dataPerkId: el.getAttribute('data-perk-id') || '',
                    href: el.getAttribute('href') || '',
                    ariaLabel: el.getAttribute('aria-label') || '',
                    className: el.className?.substring(0, 80) || '',
                    visible: el.offsetWidth > 0 && el.offsetHeight > 0,
                    rect: el.getBoundingClientRect()
                }))
                .filter(el => el.text || el.dataGetPerk || el.ariaLabel);
        });

        console.log(`📋 Total elementos extraídos: ${perks.length}`);

        // Buscar objetivo
        const target = perks.find(p => 
            p.dataGetPerk === 'gamgee.2024' ||
            p.text.includes('Obtener beneficio') ||
            (p.text.includes('Google') && p.text.includes('AI')) ||
            p.ariaLabel.includes('Gemini')
        );

        if (target) {
            console.log('🎯 OBJETIVO ENCONTRADO:', JSON.stringify(target, null, 2));
            console.log('📍 Coordenadas:', target.rect);

            // Scroll al elemento
            await page.evaluate((id) => {
                const el = document.querySelector(`[data-get-perk="${id}"]`) || 
                           Array.from(document.querySelectorAll('a, button, [role="button"]'))
                               .find(e => e.innerText?.includes('Obtener beneficio'));
                if (el) {
                    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
                    el.style.border = '4px solid red';
                    el.style.boxShadow = '0 0 30px rgba(255,0,0,0.9)';
                }
            }, target.dataGetPerk || 'gamgee.2024');
            
            await page.waitForTimeout(1000);
            
            // Click forzado
            await page.evaluate((id) => {
                const el = document.querySelector(`[data-get-perk="${id}"]`) || 
                           Array.from(document.querySelectorAll('a, button, [role="button"]'))
                               .find(e => e.innerText?.includes('Obtener beneficio'));
                if (el) {
                    console.log('🖱️ Click en:', el);
                    el.click();
                }
            }, target.dataGetPerk || 'gamgee.2024');

            console.log('✅ Click ejecutado. Esperando redirección...');
            await page.waitForTimeout(5000);
            
            await page.screenshot({ path: '/tmp/gemini_perks_post_click.png', fullPage: true });
            console.log('📸 Screenshot post-click guardado');
            console.log('📍 URL actual:', page.url());

            if (page.url().includes('accounts.google.com')) {
                console.log('🔐 Redirigido a login de Google. Modo manual activado.');
                console.log('👉 INGRESA TUS CREDENCIALES EN LA VENTANA DEL NAVEGADOR');
                console.log('⚠️ El script esperará hasta que completes el login...');
                
                // Esperar a que el usuario resuelva el login manualmente
                await page.waitForTimeout(120000); // 2 minutos para login manual
                
                console.log('📍 URL después de login:', page.url());
                await page.screenshot({ path: '/tmp/gemini_perks_after_login.png', fullPage: true });
            }
        } else {
            console.log('❌ No se encontró el objetivo. Revisa /tmp/gemini_perks_dump.html');
            console.log('🔥 Perks relevantes:');
            perks.filter(p => 
                p.text.toLowerCase().includes('beneficio') || 
                p.text.toLowerCase().includes('gemini') || 
                p.text.toLowerCase().includes('google one') ||
                p.text.toLowerCase().includes('ai') ||
                p.text.toLowerCase().includes('pro')
            ).forEach(p => console.log('  →', JSON.stringify(p)));
        }

        console.log('\n✅ Script completado. Revisa los screenshots en /tmp/');
        console.log('📁 Archivos generados:');
        console.log('  /tmp/gemini_perks_dump.html');
        console.log('  /tmp/gemini_perks_init.png');
        console.log('  /tmp/gemini_perks_post_click.png');

    } catch (error) {
        console.error('❌ Error:', error.message);
        console.error(error.stack);
    } finally {
        if (browser) {
            console.log('\n🔚 Cerrando navegador en 30 segundos...');
            await new Promise(r => setTimeout(r, 30000));
            await browser.close();
        }
    }
}

// ============================================================
// MODO DE EJECUCIÓN
// ============================================================
const args = process.argv.slice(2);
const mode = args[0] || 'help';

switch (mode) {
    case '--console':
        console.log('\n' + CONSOLE_SCRIPT);
        console.log('\n📋 Copia el script de arriba y pégalo en la consola F12 de Chrome.');
        break;
    
    case '--auto':
    case '--run':
        runNodeScript().catch(console.error);
        break;
    
    case '--guide':
        console.log(`
╔══════════════════════════════════════════════════════════════╗
║     📖 GUÍA COMPLETA — Canje Gemini Advanced (Manual)      ║
╚══════════════════════════════════════════════════════════════╝

🟢 PASO 1: CONFIGURAR PROXY (OBLIGATORIO)
   Necesitas una IP de Estados Unidos. Opciones:
   
   A) TOR (Gratis, más lento):
      $ ./scripts/tor_on.sh
      → Puerto SOCKS5: 127.0.0.1:9050
      → Verifica: curl --socks5 127.0.0.1:9050 ipinfo.io/ip
   
   B) VPN a USA (Recomendado):
      Activa tu VPN con salida en USA
      → Verifica: curl ipinfo.io/ip (debe mostrar IP gringa)
   
   C) Proxy residencial (Pago, mejor):
      Ej: BrightData, Oxylabs, IPRoyal
      → Configura en el sistema/extensiones del navegador

🟢 PASO 2: ABRIR NAVEGADOR CON SPOOFING
   A) Opción Fácil — Consola F12:
      1. Navega a:
         https://www.google.com/chromebook/perks/?hl=en&gl=us&pli=1
      2. Abre DevTools (F12 → Console)
      3. Ejecuta:
         node scripts/gemini_chromebook_spoof.cjs --console
      4. Copia el script output y pégalo en la consola
   
   B) Opción Automática (Node.js):
      $ node scripts/gemini_chromebook_spoof.cjs --run
      → Se abrirá Chromium con spoofing completo
      → Sigue las instrucciones en terminal

🟢 PASO 3: LOCALIZAR EL PERK
   Busca en la página:
   • "Google AI Pro" (tarjeta con logo de Google AI)
   • "Obtener beneficio" (botón verde/azul)
   • "Gemini Advanced" (texto promocional)
   • El botón tiene data-get-perk="gamgee.2024"

🟢 PASO 4: CLICK Y LOGIN
   1. Haz click en "Obtener beneficio"
   2. Si pide login, ingresa con cuenta Google (@gmail.com)
   3. Si aparece reCAPTCHA, resuélvelo manualmente (el script no puede)

🟢 PASO 5: COMPLETAR REGISTRO
   1. Google pedirá: método de pago (BIN requerido)
   2. Ingresa una tarjeta virtual con BIN 489504
   3. Confirma el período de prueba gratuita
   4. ¡Listo! Gemini Advanced activado por 12 meses

⚠️ REQUISITOS:
   ✓ IP de Estados Unidos (proxy/VPN)
   ✓ User-Agent Chromebook (el script lo inyecta)
   ✓ Cuenta Google (gratis)
   ✓ Tarjeta con BIN 489504 (virtual, sin fondos)
   ✓ Resolver CAPTCHA manualmente

📞 Si algo falla, verifica el proxy primero.
`);
        break;

    default:
        console.log(`
Uso: node scripts/gemini_chromebook_spoof.cjs [opción]

Opciones:
  --console    Genera script para pegar en F12 del navegador
  --run        Ejecuta automatización con Playwright (modo visible)
  --guide      Muestra guía completa paso a paso
  --help       Este mensaje

Ejemplo:
  node scripts/gemini_chromebook_spoof.cjs --guide
  node scripts/gemini_chromebook_spoof.cjs --run
`);
        break;
}
