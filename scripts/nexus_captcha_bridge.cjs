/**
 * 🧬 NEXUS CAPTCHA BRIDGE v1 — BrowserBridge + Capsolver Client
 * 
 * BrowserBridge: detecta CAPTCHA en DOM, inyecta tokens, coordina resolución
 * Integra con StealthEngine v3 para evasión biométrica + Capsolver API
 */

const https = require('https');

// ============================================================================
// CAPSOLVER API CLIENT
// ============================================================================

class CapsolverClient {
  constructor(apiKey) {
    this.apiKey = apiKey;
    this.baseUrl = 'api.capsolver.com';
    this.pollInterval = 2000;
    this.maxRetries = 60; // 2 minutos max
  }

  /**
   * Consulta saldo disponible
   */
  async getBalance() {
    const data = await this._request('/getBalance', { clientKey: this.apiKey });
    if (data.errorId !== 0) throw new Error(`Capsolver error: ${data.errorDescription || data.errorCode}`);
    return data.balance;
  }

  /**
   * Crea una tarea de resolución CAPTCHA
   */
  async createTask(captchaType, taskParams) {
    const task = {
      type: captchaType,
      websiteURL: taskParams.url,
      websiteKey: taskParams.siteKey,
    };

    if (taskParams.pageAction) task.pageAction = taskParams.pageAction;
    if (taskParams.minScore !== undefined) task.minScore = taskParams.minScore;
    if (taskParams.isInvisible) task.isInvisible = true;

    const data = await this._request('/createTask', {
      clientKey: this.apiKey,
      task,
    });

    if (data.errorId !== 0) {
      throw new Error(`Capsolver createTask: [${data.errorCode}] ${data.errorDescription}`);
    }

    return data.taskId;
  }

  /**
   * Hace polling hasta obtener el token resuelto
   */
  async getTaskResult(taskId) {
    const start = Date.now();

    for (let attempt = 0; attempt < this.maxRetries; attempt++) {
      const data = await this._request('/getTaskResult', {
        clientKey: this.apiKey,
        taskId,
      });

      if (data.errorId !== 0) {
        throw new Error(`Capsolver getTaskResult: [${data.errorCode}] ${data.errorDescription}`);
      }

      if (data.status === 'ready') {
        const solution = data.solution || {};
        const token = solution.gRecaptchaResponse || solution.token || solution.text || null;
        return {
          token,
          solution,
          elapsed: Date.now() - start,
        };
      }

      // Aún procesando
      await this._sleep(this.pollInterval);
    }

    throw new Error(`Capsolver timeout (${this.maxRetries * this.pollInterval}ms) para taskId: ${taskId}`);
  }

  /**
   * Método completo: crear tarea + polling
   */
  async solve(captchaType, taskParams) {
    const taskId = await this.createTask(captchaType, taskParams);
    return this.getTaskResult(taskId);
  }

  // --- privados ---

  _request(path, body) {
    return new Promise((resolve, reject) => {
      const payload = JSON.stringify(body);
      const options = {
        hostname: this.baseUrl,
        path,
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(payload),
        },
        timeout: 30000,
      };

      const req = https.request(options, (res) => {
        let data = '';
        res.on('data', (chunk) => data += chunk);
        res.on('end', () => {
          try {
            resolve(JSON.parse(data));
          } catch (e) {
            reject(new Error(`Capsolver parse error: ${data.slice(0, 200)}`));
          }
        });
      });

      req.on('error', reject);
      req.on('timeout', () => { req.destroy(); reject(new Error('Capsolver timeout')); });
      req.write(payload);
      req.end();
    });
  }

  _sleep(ms) {
    return new Promise(r => setTimeout(r, ms));
  }
}

// ============================================================================
// CAPTCHA TYPES
// ============================================================================

const CAPTCHA_TYPES = {
  RECAPTCHA_V2: 'ReCaptchaV2TaskProxyless',
  RECAPTCHA_V3: 'ReCaptchaV3TaskProxyless',
  HCAPTCHA: 'HCaptchaTaskProxyless',
  TURNSTILE: 'AntiTurnstileTaskProxyless',
  IMAGE: 'ImageToTextTask',
};

// ============================================================================
// SELECTORS PARA DETECCIÓN DE CAPTCHA EN DOM
// ============================================================================

const CAPTCHA_SELECTORS = {
  [CAPTCHA_TYPES.RECAPTCHA_V2]: [
    'iframe[src*="google.com/recaptcha/api2/anchor"]',
    'iframe[src*="recaptcha/api2"]',
    'div.g-recaptcha',
    'div[class*="g-recaptcha"]',
  ],
  [CAPTCHA_TYPES.RECAPTCHA_V3]: [
    'iframe[src*="google.com/recaptcha/api.js"]',
    'script[src*="recaptcha/api.js"]',
    'textarea#g-recaptcha-response',
  ],
  [CAPTCHA_TYPES.HCAPTCHA]: [
    'iframe[src*="hcaptcha.com"]',
    'div.h-captcha',
    'div[class*="h-captcha"]',
  ],
  [CAPTCHA_TYPES.TURNSTILE]: [
    'iframe[src*="challenges.cloudflare.com"]',
    'div.cf-turnstile',
    'div[class*="cf-turnstile"]',
  ],
};

// ============================================================================
// BROWSER BRIDGE
// ============================================================================

/**
 * ============================================================================
 * 🧬 ARQUITECTURA: CaptchaBridge se inyecta DENTRO del navegador MCP
 * ============================================================================
 *
 * ┌─────────────────────────────────────────────────────────────────┐
 * │  NEXUS BROWSER MCP (Playwright)                                │
 * │  ├── StealthEngine v3  ← ruido biométrico                      │
 * │  ├── CaptchaBridge    ← ¡AQUÍ! detecta + resuelve + inyecta    │
 * │  │   ├── detectCaptcha()  → escanea el DOM en vivo             │
 * │  │   ├── resolveCaptcha() → Capsolver API (pagado)             │
 * │  │   ├── injectToken()    → inyecta token en la página         │
 * │  │   └── [FUTURO] localResolve() → Tesseract/EasyOCR/LLaVA     │
 * │  └── proxyMesh (Tor)    ← rotación de IP                       │
 * └─────────────────────────────────────────────────────────────────┘
 *
 * 🔜 PRÓXIMOS MÓDULOS (separados, no dentro del navegador):
 * ┌─────────────────────────────────────┐
 * │  core/src/captcha/local_resolver.rs │  ← Rust, subprocess
 * │  ├── Tesseract OCR (subprocess)     │     llama a Python
 * │  ├── EasyOCR (bridge Python)        │     por separado
 * │  └── LLaVA (Ollama API local)       │
 * └─────────────────────────────────────┘
 *         ↕ IPC (stdin/stdout o HTTP)
 * ┌─────────────────────────────────────┐
 * │  CaptchaBridge (dentro del MCP)     │
 * └─────────────────────────────────────┘
 *
 * CONCLUSIÓN:
 * - CaptchaBridge JS → VIVE DENTRO del navegador MCP
 * - Local resolvers (Tesseract/EasyOCR/LLaVA) → MÓDULOS SEPARADOS llamados por el bridge
 * ============================================================================
 */

class CaptchaBridge {
  constructor(capsolverApiKey) {
    this.capsolver = new CapsolverClient(capsolverApiKey);
  }

  /**
   * Detecta qué tipo de CAPTCHA hay en la página actual via DOM + JS
   * @param {import('playwright').Page} page
   * @returns {Promise<{captcha: string|null, siteKey: string|null, action: string|null, message: string}>}
   */
  async detectCaptcha(page) {
    const result = await page.evaluate((selectors) => {
      // Mapear selectores a tipos de captcha
      const typeMap = {};
      for (const [type, sels] of Object.entries(selectors)) {
        for (const sel of sels) {
          typeMap[sel] = type;
        }
      }

      // Buscar todos los iframes/elementos de captcha
      for (const [sel, type] of Object.entries(typeMap)) {
        const el = document.querySelector(sel);
        if (el) {
          // Extraer site-key
          let siteKey = null;
          if (el.tagName === 'IFRAME') {
            const src = el.src || '';
            const match = src.match(/[?&]k=([^&]+)/);
            if (match) siteKey = match[1];
          } else if (el.dataset.sitekey) {
            siteKey = el.dataset.sitekey;
          } else if (el.getAttribute('data-sitekey')) {
            siteKey = el.getAttribute('data-sitekey');
          }

          return { captcha: type, siteKey, found: true };
        }
      }

      // Detectar Cloudflare challenge por HTML específico
      if (document.getElementById('challenge-form') || 
          document.querySelector('.cf-browser-verification') ||
          document.body.textContent.includes('Checking your browser')) {
        return { captcha: 'CloudflareChallenge', siteKey: null, found: true };
      }

      return { captcha: null, siteKey: null, found: false };
    }, CAPTCHA_SELECTORS);

    if (result.found) {
      return {
        captcha: result.captcha,
        siteKey: result.siteKey,
        action: null,
        message: `✅ CAPTCHA detectado: ${result.captcha}${result.siteKey ? ` (key: ${result.siteKey.slice(0, 10)}...)` : ''}`,
      };
    }

    return {
      captcha: null,
      siteKey: null,
      action: null,
      message: '✅ No se detectó CAPTCHA en la página',
    };
  }

  /**
   * Inyecta token de resolución en la página
   * @param {import('playwright').Page} page
   * @param {string} token
   * @param {string} captchaType
   */
  async injectToken(page, token, captchaType) {
    const success = await page.evaluate(({ token, captchaType }) => {
      try {
        switch (captchaType) {
          case 'ReCaptchaV2TaskProxyless':
          case 'ReCaptchaV3TaskProxyless': {
            // Inyectar en textarea
            const textarea = document.getElementById('g-recaptcha-response');
            if (textarea) {
              textarea.value = token;
              textarea.style.display = 'block';
              // Disparar evento onChange
              textarea.dispatchEvent(new Event('input', { bubbles: true }));
              textarea.dispatchEvent(new Event('change', { bubbles: true }));
            }
            // Callback de grecaptcha si existe
            if (window.___grecaptcha_cfg) {
              try {
                const clientIds = Object.keys(window.___grecaptcha_cfg.clients || {});
                for (const id of clientIds) {
                  const client = window.___grecaptcha_cfg.clients[id];
                  if (client && typeof client.callback === 'function') {
                    client.callback(token);
                  }
                }
              } catch (e) { /* silencio */ }
            }
            return true;
          }
          case 'HCaptchaTaskProxyless': {
            if (window.hcaptcha) {
              window.hcaptcha.setData(token);
              return true;
            }
            // Fallback: buscar callback
            const hcaptchaIframe = document.querySelector('iframe[src*="hcaptcha.com"]');
            if (hcaptchaIframe) {
              hcaptchaIframe.contentWindow?.postMessage({ token }, '*');
              return true;
            }
            return false;
          }
          case 'AntiTurnstileTaskProxyless': {
            // Turnstile - inyectar token via postMessage
            const turnstileIframes = document.querySelectorAll('iframe[src*="challenges.cloudflare.com"]');
            turnstileIframes.forEach(iframe => {
              iframe.contentWindow?.postMessage({ 
                source: 'turnstile', 
                token,
                widgetId: iframe.getAttribute('data-widget-id')
              }, '*');
            });
            return turnstileIframes.length > 0;
          }
          default:
            return false;
        }
      } catch (e) {
        return false;
      }
    }, { token, captchaType });

    return success;
  }

  /**
   * Resuelve CAPTCHA de imagen (ImageToTextTask) con resolver LOCAL
   * (tesseract + Ollama visión) vía el contenedor CUA — sin API key.
   * Solo para CAPTCHAs de imagen/texto; reCAPTCHA/hCaptcha van a Capsolver.
   * @param {string} imagePathOrB64 - ruta a imagen o base64
   * @returns {Promise<{success: boolean, answer: string|null, method: string, message: string}>}
   */
  async resolveImageLocal(imagePathOrB64, captchaType) {
    const start = Date.now();
    try {
      // Ejecutar el resolver local dentro del contenedor CUA (Nexus Hands world)
      const exec = require('child_process').execFileSync;
      const args = [
        'exec', 'nexus-cua-gui', 'python3', '/opt/nexus_captcha_local.py',
        'solve', imagePathOrB64,
        '--tipo', captchaType === 'semantico' ? 'semantico' : 'texto_ocr',
      ];
      const out = exec('docker', args, { encoding: 'utf-8', timeout: 90000 });
      const result = JSON.parse(out);

      if (result.ok && result.respuesta) {
        return {
          success: true,
          answer: result.respuesta,
          method: `local:${result.metodo}`,
          elapsed: Date.now() - start,
          message: `✅ CAPTCHA de imagen resuelto localmente (${result.metodo})`,
        };
      }
      return {
        success: false,
        answer: null,
        method: `local:${result.metodo || 'unknown'}`,
        elapsed: Date.now() - start,
        message: `Resolver local no devolvió respuesta: ${result.error || 'vacío'}`,
      };
    } catch (e) {
      return {
        success: false,
        answer: null,
        method: 'local',
        elapsed: Date.now() - start,
        message: `Resolver local error: ${e.message}`,
      };
    }
  }

  /**
   * Resuelve CAPTCHA detectado automáticamente
   * Orden: LOCAL (imagen/texto) → Capsolver (reCAPTCHA/hCaptcha/Turnstile)
   * @param {import('playwright').Page} page
   * @param {string} url - URL actual (página del captcha)
   * @returns {Promise<{success: boolean, token: string|null, method: string, elapsed: number}>}
   */
  async resolveCaptcha(page, url) {
    const detection = await this.detectCaptcha(page);
    
    if (!detection.captcha) {
      return { success: false, token: null, method: 'none', elapsed: 0, message: 'No CAPTCHA detected' };
    }

    // Si es Cloudflare challenge, no podemos resolver con Capsolver directamente
    if (detection.captcha === 'CloudflareChallenge') {
      return {
        success: false,
        token: null,
        method: 'none',
        elapsed: 0,
        message: 'Cloudflare Challenge detectado — requiere evasión biométrica + rotación IP'
      };
    }

    const start = Date.now();

    // ===== TIER LOCAL: CAPTCHA de imagen/texto se resuelve SIN API key =====
    if (detection.captcha === CAPTCHA_TYPES.IMAGE) {
      // Capturar la imagen del CAPTCHA desde la página (elemento img o canvas)
      const imgData = await this._extractCaptchaImage(page, detection);
      if (imgData) {
        const local = await this.resolveImageLocal(imgData, 'texto_ocr');
        if (local.success) {
          // Inyectar la respuesta de texto en el input del captcha
          await this._fillImageAnswer(page, local.answer);
          return {
            success: true,
            token: local.answer,
            method: local.method,
            elapsed: local.elapsed,
            message: local.message,
          };
        }
      }
      // fallback → Capsolver ImageToText
      return this._resolveImageWithCapsolver(detection, start);
    }

    if (!detection.siteKey) {
      return {
        success: false,
        token: null,
        method: 'none',
        elapsed: 0,
        message: `CAPTCHA ${detection.captcha} detectado pero sin siteKey`
      };
    }

    try {
      const taskParams = {
        url,
        siteKey: detection.siteKey,
      };

      // Para reCAPTCHA v3, enviar action
      if (detection.captcha === 'ReCaptchaV3TaskProxyless') {
        taskParams.pageAction = detection.action || 'verify';
        taskParams.minScore = 0.5;
      }

      const result = await this.capsolver.solve(detection.captcha, taskParams);

      if (result.token) {
        // Inyectar token en la página
        await this.injectToken(page, result.token, detection.captcha);
        
        return {
          success: true,
          token: result.token,
          method: 'capsolver',
          elapsed: Date.now() - start,
          message: `✅ CAPTCHA resuelto (${Date.now() - start}ms)`,
        };
      }

      return {
        success: false,
        token: null,
        method: 'capsolver',
        elapsed: Date.now() - start,
        message: 'Capsolver no devolvió token',
      };
    } catch (e) {
      return {
        success: false,
        token: null,
        method: 'capsolver',
        elapsed: Date.now() - start,
        message: `Capsolver error: ${e.message}`,
      };
    }
  }

  /**
   * Extrae la imagen de un CAPTCHA de imagen desde la página
   * @private
   */
  async _extractCaptchaImage(page, detection) {
    try {
      return await page.evaluate(() => {
        // Buscar un <img> cerca del elemento captcha o en el DOM
        const imgSelectors = [
          'img[src*="captcha"]',
          'img[src*="recaptcha"]',
          '.captcha img',
          '#captcha img',
          'img[alt*="captcha" i]',
          'img[src*="image"][src*="challenge"]',
        ];
        for (const sel of imgSelectors) {
          const img = document.querySelector(sel);
          if (img && img.src) return img.src;
        }
        // Intentar canvas a dataURL
        const canvas = document.querySelector('canvas');
        if (canvas) {
          try { return canvas.toDataURL('image/png'); } catch (e) { /* silencio */ }
        }
        return null;
      });
    } catch (e) {
      return null;
    }
  }

  /**
   * Rellena el input de respuesta de un CAPTCHA de imagen con la respuesta
   * @private
   */
  async _fillImageAnswer(page, answer) {
    try {
      await page.evaluate((answer) => {
        const sels = [
          'input[name*="captcha" i]',
          'input[id*="captcha" i]',
          '#captcha-input',
          'input[placeholder*="captcha" i]',
        ];
        for (const sel of sels) {
          const el = document.querySelector(sel);
          if (el) {
            el.focus();
            el.value = answer;
            el.dispatchEvent(new Event('input', { bubbles: true }));
            el.dispatchEvent(new Event('change', { bubbles: true }));
            return true;
          }
        }
        return false;
      }, answer);
    } catch (e) { /* silencio */ }
  }

  /**
   * Resuelve CAPTCHA de imagen con Capsolver (fallback cuando local falla)
   * @private
   */
  async _resolveImageWithCapsolver(detection, start) {
    try {
      if (!detection.siteKey) {
        return { success: false, token: null, method: 'capsolver-image', elapsed: Date.now() - start, message: 'Imagen CAPTCHA sin siteKey' };
      }
      const result = await this.capsolver.solve(CAPTCHA_TYPES.IMAGE, {
        url: detection.siteKey,
        siteKey: detection.siteKey,
      });
      if (result.token) {
        return {
          success: true,
          token: result.token,
          method: 'capsolver-image',
          elapsed: Date.now() - start,
          message: `✅ CAPTCHA de imagen resuelto vía Capsolver (${Date.now() - start}ms)`,
        };
      }
      return { success: false, token: null, method: 'capsolver-image', elapsed: Date.now() - start, message: 'Capsolver imagen no devolvió token' };
    } catch (e) {
      return { success: false, token: null, method: 'capsolver-image', elapsed: Date.now() - start, message: `Capsolver imagen error: ${e.message}` };
    }
  }

  /**
   * Verifica saldo de Capsolver
   */
  async checkBalance() {
    try {
      const balance = await this.capsolver.getBalance();
      return { available: true, balance, message: `💰 Saldo Capsolver: $${balance.toFixed(4)}` };
    } catch (e) {
      return { available: false, balance: 0, message: `❌ Error saldo: ${e.message}` };
    }
  }
}

// ============================================================================
// EXPORT
// ============================================================================

module.exports = {
  CaptchaBridge,
  CapsolverClient,
  CAPTCHA_TYPES,
  CAPTCHA_SELECTORS,
};
