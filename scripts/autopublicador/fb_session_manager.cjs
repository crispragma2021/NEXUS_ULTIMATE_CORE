const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');
const { StealthEngine } = require('../nexus_stealth_engine.cjs');

const BASE_DIR = '/home/soberano/NEXUS_ULTIMATE_CORE';

class FBSessionManager {
    constructor() {
        this.profileDir = path.join(BASE_DIR, 'data/gabriel_profile');
        this.sessionStateFile = path.join(BASE_DIR, 'data/secrets/fb_gabriel_state.json');
        this.stealth = new StealthEngine();
        this.lastFingerprint = null;
    }

    async launchStealthBrowser(options = {}) {
        console.log('[🧬] Iniciando motor de navegación stealth para Gabriel...');
        
        const fp = this.stealth.fingerprintGenerator.generate();
        this.lastFingerprint = fp;

        const launchOptions = this.stealth.getLaunchOptions();
        
        // Configuración absoluta para Facebook
        const context = await chromium.launchPersistentContext(this.profileDir, {
            headless: options.headless !== undefined ? options.headless : false,
            userAgent: fp.userAgent,
            viewport: fp.viewport,
            locale: fp.locale,
            timezoneId: fp.timezoneId,
            args: [
                ...launchOptions.args,
                '--disable-blink-features=AutomationControlled',
                '--disable-features=IsolateOrigins,site-per-process',
            ],
            bypassCSP: true,
            ignoreHTTPSErrors: true
        });

        // Inyectar camuflaje profundo
        await context.addInitScript(this.stealth.getInitScript());
        
        // Si existe un estado de sesión guardado, podrías cargarlo, 
        // pero launchPersistentContext ya maneja la mayoría en profileDir.

        return { context, engine: this.stealth, fingerprint: fp };
    }

    async saveSession(context) {
        console.log('[💾] Guardando estado soberano de la sesión...');
        const state = await context.storageState();
        fs.mkdirSync(path.dirname(this.sessionStateFile), { recursive: true });
        fs.writeFileSync(this.sessionStateFile, JSON.stringify(state, null, 2));
    }

    async verifyFBLogin(page) {
        console.log('[🔍] Verificando integridad de la sesión en Facebook...');
        try {
            await page.goto('https://www.facebook.com/', { waitUntil: 'networkidle', timeout: 30000 });
            
            // Si vemos el campo de búsqueda o el composer, estamos dentro
            const loggedIn = await page.evaluate(() => {
                const indicators = [
                    '[aria-label="¿Qué estás pensando?"]',
                    '[role="search"]',
                    '[aria-label="Facebook"]',
                    'a[href="/me/"]'
                ];
                return indicators.some(sel => !!document.querySelector(sel));
            });

            if (loggedIn) {
                console.log('[✅] Sesión de Gabriel activa y validada.');
                return true;
            } else {
                console.log('[⚠️] Sesión no detectada. Se requiere intervención o re-login.');
                return false;
            }
        } catch (e) {
            console.error(`[❌] Error en validación: ${e.message}`);
            return false;
        }
    }
}

module.exports = { FBSessionManager };
