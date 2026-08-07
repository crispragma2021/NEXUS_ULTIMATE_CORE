/**
 * NEXUS Stealth Engine v3: Módulo de Evasión de Detección de Bots y Fingerprinting
 * 
 * Capacidades:
 * - Fingerprint aleatorio (UA, viewport, webgl, timezone, locale)
 * - Ruido biométrico: movimiento Perlin, tecleo realista, scroll con inercia
 * - ByPass de reCAPTCHA v3 mediante simulación de comportamiento humano
 */

class FingerprintGenerator {
  constructor() {
    this.userAgents = [
      'Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
      'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
      'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0',
      'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0',
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0',
      'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0',
      'Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0',
    ];

    this.viewports = [
      { width: 1920, height: 1080 },
      { width: 1366, height: 768 },
      { width: 1536, height: 864 },
      { width: 1280, height: 720 },
      { width: 1440, height: 900 },
    ];

    this.webglVendors = [
      { vendor: 'Google Inc.', renderer: 'ANGLE (AMD, AMD Radeon Graphics Direct3D11 vs_5_0, D3D11)' },
      { vendor: 'Google Inc.', renderer: 'ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0, D3D11)' },
      { vendor: 'Intel Inc.', renderer: 'ANGLE (Intel, Intel(R) Iris(TM) Xe Graphics Direct3D11 vs_5_0, D3D11)' },
      { vendor: 'Google Inc.', renderer: 'ANGLE (Apple, Apple M1 Pro)' },
    ];

    this.timezones = [
      'America/Asuncion', 'America/Buenos_Aires', 'America/Sao_Paulo',
      'America/New_York', 'Europe/London', 'Asia/Tokyo',
    ];

    this.locales = [
      'es-PY', 'es-ES', 'en-US', 'pt-BR', 'ja-JP',
    ];

    this.platforms = [
      'Linux x86_64', 'Win32', 'MacIntel',
    ];
  }

  random(arr) {
    return arr[Math.floor(Math.random() * arr.length)];
  }

  generate() {
    return {
      userAgent: this.random(this.userAgents),
      viewport: this.random(this.viewports),
      timezoneId: this.random(this.timezones),
      locale: this.random(this.locales),
      platform: this.random(this.platforms),
      webglVendor: this.random(this.webglVendors),
    };
  }
}

/**
 * Genera valores con distribución normal (Gaussiana)
 * Usa transformación Box-Muller
 */
function gaussianRandom(mean = 0, stdDev = 1) {
  let u = 0, v = 0;
  while (u === 0) u = Math.random();
  while (v === 0) v = Math.random();
  const z = Math.sqrt(-2.0 * Math.log(u)) * Math.cos(2.0 * Math.PI * v);
  return z * stdDev + mean;
}

/**
 * Genera una trayectoria de mouse con ruido Perlin simplificado (bezier + jitter)
 */
function generatePerlinCurve(fromX, fromY, toX, toY, steps = 30) {
  const points = [];
  const controlPoints = [];
  
  // Generar 2 puntos de control aleatorios para la curva bezier cúbica
  const midX = (fromX + toX) / 2;
  const midY = (fromY + toY) / 2;
  const offsetMagnitude = Math.sqrt((toX - fromX) ** 2 + (toY - fromY) ** 2) * 0.3;
  
  const cp1x = midX + (Math.random() - 0.5) * offsetMagnitude;
  const cp1y = midY + (Math.random() - 0.5) * offsetMagnitude;
  const cp2x = midX + (Math.random() - 0.5) * offsetMagnitude * 0.7;
  const cp2y = midY + (Math.random() - 0.5) * offsetMagnitude * 0.7;
  
  controlPoints.push({ x: cp1x, y: cp1y }, { x: cp2x, y: cp2y });

  // Interpolar puntos a lo largo de la curva bezier cúbica con jitter Perlin
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    // Bezier cúbico: B(t) = (1-t)³P0 + 3(1-t)²tP1 + 3(1-t)t²P2 + t³P3
    const u = 1 - t;
    const tt = t * t;
    const uu = u * u;
    const uuu = uu * u;
    const ttt = tt * t;
    
    let x = uuu * fromX + 3 * uu * t * cp1x + 3 * u * tt * cp2x + ttt * toX;
    let y = uuu * fromY + 3 * uu * t * cp1y + 3 * u * tt * cp2y + ttt * toY;
    
    // Añadir jitter Perlin no-lineal (más jitter en el medio, menos en extremos)
    const jitterFactor = Math.sin(t * Math.PI) * 2.5; // Seno: 0 en t=0, max en t=0.5, 0 en t=1
    const jitterAngle = Math.random() * Math.PI * 2;
    x += Math.cos(jitterAngle) * jitterFactor;
    y += Math.sin(jitterAngle) * jitterFactor;
    
    points.push({ x: Math.round(x), y: Math.round(y) });
  }

  return { points, controlPoints };
}

/**
 * Calcula el delay entre teclas basado en distribución normal (Box-Muller)
 * Media: 80ms, StdDev: 25ms, Clamped: [30, 200]
 */
function keyboardDelay() {
  const raw = gaussianRandom(80, 25);
  return Math.max(30, Math.min(200, Math.round(raw)));
}

class StealthEngine {
  constructor() {
    this.fingerprintGenerator = new FingerprintGenerator();
  }

  // ============================================================
  // FINGERPRINT
  // ============================================================

  getLaunchOptions() {
    const fp = this.fingerprintGenerator.generate();
    return {
      headless: true,
      args: [
        '--disable-blink-features=AutomationControlled',
        '--no-sandbox',
        '--disable-infobars',
        `--window-size=${fp.viewport.width},${fp.viewport.height}`,
      ],
      viewport: fp.viewport,
      userAgent: fp.userAgent,
      locale: fp.locale,
      timezoneId: fp.timezoneId,
    };
  }

  getInitScript() {
    const randomHardwareConcurrency = this.fingerprintGenerator.random([2, 4, 8, 12, 16]);
    const locales = [...this.fingerprintGenerator.locales, 'en-US', 'en-GB']
      .sort(() => 0.5 - Math.random()).slice(0, 3);
    const randomLocales = JSON.stringify(locales);
    const randomPlatform = this.fingerprintGenerator.random(this.fingerprintGenerator.platforms);
    const randomWebglVendor = this.fingerprintGenerator.random(this.fingerprintGenerator.webglVendors);

    return `
      (async () => {
        Object.defineProperty(navigator, 'webdriver', { get: () => false });
        Object.defineProperty(navigator, 'plugins', {
          get: () => [{
            description: 'Portable Document Format',
            filename: 'internal-pdf-viewer',
            name: 'Chrome PDF Plugin',
            length: 1,
            0: { type: 'application/pdf', suffixes: 'pdf', description: 'Portable Document Format' },
          }],
        });
        Object.defineProperty(navigator, 'mimeTypes', {
          get: () => [{
            description: 'Portable Document Format',
            suffixes: 'pdf',
            type: 'application/pdf',
            enabledPlugin: this.plugins[0],
          }],
        });
        Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => ${randomHardwareConcurrency} });
        Object.defineProperty(navigator, 'languages', { get: () => ${randomLocales} });
        Object.defineProperty(navigator, 'platform', { get: () => '${randomPlatform}' });
        if (!window.chrome) {
          window.chrome = { runtime: {}, loadTimes: () => ({}), csi: () => ({}) };
        }
        const originalQuery = window.navigator.permissions.query;
        window.navigator.permissions.query = (parameters) => (
          parameters.name === 'notifications'
            ? Promise.resolve({ state: Notification.permission })
            : originalQuery(parameters)
        );
        const getParameter = WebGLRenderingContext.prototype.getParameter;
        WebGLRenderingContext.prototype.getParameter = function(parameter) {
          if (parameter === 37445) return '${randomWebglVendor.vendor}';
          if (parameter === 37446) return '${randomWebglVendor.renderer}';
          return getParameter(parameter);
        };
        const toDataURL = HTMLCanvasElement.prototype.toDataURL;
        HTMLCanvasElement.prototype.toDataURL = function(type, encoderOptions) {
          const context = this.getContext('2d');
          if (context) {
            context.font = '1px Arial';
            const r = Math.floor(Math.random() * 255);
            const g = Math.floor(Math.random() * 255);
            const b = Math.floor(Math.random() * 255);
            context.fillStyle = "rgba(" + r + ", " + g + ", " + b + ", 0.01)";
            context.fillText('Nexus', 0, 0);
          }
          return toDataURL.apply(this, arguments);
        };
      })();
    `;
  }

  // ============================================================
  // RUIDO BIOMÉTRICO: Movimiento de Mouse con curva Perlin
  // ============================================================

  /**
   * Mueve el mouse simulando una trayectoria natural (curva bezier + jitter Perlin)
   * @param {import('playwright').Page} page
   * @param {number} toX - Coordenada X destino
   * @param {number} toY - Coordenada Y destino
   * @param {number} durationMs - Duración total del movimiento (default: 300-600ms)
   */
  async mouseMoveBiometric(page, toX, toY, durationMs) {
    // Obtener posición actual del mouse
    const currentPos = await page.evaluate(() => ({ x: window.__nexus_mouse_x || 0, y: window.__nexus_mouse_y || 0 }));
    
    const duration = durationMs || Math.floor(Math.random() * 300) + 300; // 300-600ms
    const steps = Math.max(10, Math.floor(duration / 15)); // ~15ms por paso
    
    const { points } = generatePerlinCurve(currentPos.x, currentPos.y, toX, toY, steps);
    
    for (let i = 0; i < points.length; i++) {
      const point = points[i];
      await page.mouse.move(point.x, point.y);
      
      // Delay variable entre movimientos (distribución normal)
      const stepDelay = Math.max(5, Math.round(gaussianRandom(12, 4)));
      await page.waitForTimeout(stepDelay);
    }
    
    // Almacenar última posición conocida
    await page.evaluate((x, y) => { window.__nexus_mouse_x = x; window.__nexus_mouse_y = y; }, toX, toY);
  }

  /**
   * Hace clic con movimiento biométrico previo
   * @param {import('playwright').Page} page
   * @param {string} selector
   */
  async clickBiometric(page, selector) {
    const locator = page.locator(selector).first();
    const box = await locator.boundingBox();
    if (!box) throw new Error(`Elemento no visible: ${selector}`);
    
    // Calcular punto central del elemento con pequeño jitter aleatorio
    const centerX = box.x + box.width / 2 + (Math.random() - 0.5) * 4;
    const centerY = box.y + box.height / 2 + (Math.random() - 0.5) * 4;
    
    // 1. Mover el mouse con trayectoria natural
    await this.mouseMoveBiometric(page, centerX, centerY);
    
    // 2. Pausa de "decisión humana" (150-400ms)
    await page.waitForTimeout(Math.floor(Math.random() * 250) + 150);
    
    // 3. Hover suave + clic
    await locator.hover({ timeout: 5000, force: true });
    await page.waitForTimeout(Math.floor(Math.random() * 80) + 20);
    await locator.click({ timeout: 5000, force: true });
    
    // 4. Post-click delay
    await page.waitForTimeout(Math.floor(Math.random() * 100) + 50);
  }

  // ============================================================
  // RUIDO BIOMÉTRICO: Tecleo No-Determinista
  // ============================================================

  /**
   * Escribe texto simulando digitación humana real con distribución normal de delays
   * @param {import('playwright').Page} page
   * @param {string} selector
   * @param {string} text
   */
  async typeBiometric(page, selector, text) {
    const locator = page.locator(selector).first();
    
    // Click biométrico primero para enfocar
    await this.clickBiometric(page, selector);
    await page.waitForTimeout(Math.floor(Math.random() * 150) + 50);
    
    // Limpiar campo
    await locator.fill('');
    await page.waitForTimeout(Math.floor(Math.random() * 100) + 50);
    
    // Escribir carácter por carácter con delays realistas
    let previousErrors = 0;
    for (let i = 0; i < text.length; i++) {
      const char = text[i];
      
      // 2% de probabilidad de error tipográfico (carácter incorrecto + backspace)
      if (Math.random() < 0.02 && i > 2 && previousErrors < 2) {
        previousErrors++;
        const wrongChar = String.fromCharCode(char.charCodeAt(0) + (Math.random() > 0.5 ? 1 : -1));
        await page.keyboard.type(wrongChar);
        await page.waitForTimeout(keyboardDelay());
        await page.keyboard.press('Backspace');
        await page.waitForTimeout(keyboardDelay() * 1.5);
      }
      
      // Escribir carácter con delay gaussiano
      await page.keyboard.type(char);
      await page.waitForTimeout(keyboardDelay());
    }
  }

  // ============================================================
  // RUIDO BIOMÉTRICO: Scroll con Inercia
  // ============================================================

  /**
   * Realiza scroll suave simulando movimiento de rueda de mouse o trackpad
   * @param {import('playwright').Page} page
   * @param {number} distance - Distancia total en píxels (negativo = arriba)
   * @param {number} durationMs - Duración total (default: 500-1000ms)
   */
  async scrollBiometric(page, distance, durationMs) {
    const duration = durationMs || Math.floor(Math.random() * 500) + 500;
    const steps = Math.max(8, Math.floor(duration / 30)); // ~30ms por paso
    const direction = distance > 0 ? 1 : -1;
    const absDistance = Math.abs(distance);
    
    // Curva de aceleración: inicio lento, acelera, desacelera al final
    for (let i = 0; i < steps; i++) {
      const t = i / steps;
      // Fórmula de ease-in-out: t < 0.5 ? 2*t^2 : -1+(4-2*t)*t
      const easedT = t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
      const prevEasedT = i > 0 ? (i - 1) / steps < 0.5 
        ? 2 * ((i - 1) / steps) ** 2 
        : -1 + (4 - 2 * ((i - 1) / steps)) * ((i - 1) / steps) : 0;
      
      const stepDistance = (easedT - prevEasedT) * absDistance * direction;
      
      await page.mouse.wheel(0, stepDistance);
      
      // Delay entre scrolls con distribución normal
      const stepDelay = Math.max(10, Math.round(gaussianRandom(30, 10)));
      await page.waitForTimeout(stepDelay);
    }
  }
}

module.exports = { StealthEngine };
