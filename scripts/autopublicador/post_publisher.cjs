const path = require('path');
const fs = require('fs');

class PostPublisher {
    constructor(page, stealthEngine) {
        this.page = page;
        this.stealth = stealthEngine;
    }

    async navigateToHome() {
        console.log('[📱] Navegando a Facebook Home...');
        await this.page.goto('https://www.facebook.com/', {
            waitUntil: 'networkidle',
            timeout: 60000,
        });

        // Limpiar modales intrusivos
        await this.dismissModals();
    }

    async dismissModals() {
        await this.page.evaluate(() => {
            const selectors = [
                'div[role="dialog"]',
                'div[id^="login_"]',
                'div[class*="login"]',
                'div[aria-label*="Log In"]',
                'div[aria-label*="Iniciar sesión"]',
                'div[aria-label*="Notifications"]',
                'div[aria-label*="Cerrar"]',
                '[role="banner"] + div div[role="button"]' // A veces el botón X de popups
            ];
            selectors.forEach(s => {
                document.querySelectorAll(s).forEach(el => {
                    try { el.remove(); } catch(e) {}
                });
            });
            document.body.style.overflow = 'visible';
        });
        await this.page.waitForTimeout(1000);
    }

    async clickComposer() {
        console.log('[🖱️] Localizando caja de publicación...');
        
        // Facebook cambia clases constantemente, usamos atributos aria y texto
        const composerSelectors = [
            '[aria-label="¿Qué estás pensando?"]',
            '[aria-label*="pensando"]',
            'div[role="button"] span:has-text("pensando")',
            'span:text("¿Qué estás pensando, Gabriel?")',
            'div[class*="xh8yej3"]' 
        ];

        for (const selector of composerSelectors) {
            try {
                const el = await this.page.$(selector);
                if (el) {
                    await this.stealth.clickBiometric(this.page, selector);
                    await this.page.waitForTimeout(2000);
                    return true;
                }
            } catch (e) {}
        }

        // Fallback: Click en cualquier cosa que parezca el composer
        await this.page.evaluate(() => {
            const spans = Array.from(document.querySelectorAll('span'));
            const target = spans.find(s => s.innerText.includes('pensando') || s.innerText.includes('mind'));
            if (target) {
                target.closest('[role="button"]')?.click();
            }
        });
        await this.page.waitForTimeout(2000);
        return true;
    }

    async typeContent(content) {
        console.log('[⌨️] Escribiendo contenido biométricamente...');
        
        // Selectores para el editor expandido
        const textAreaSelectors = [
            '[aria-label*="pensando"] div[contenteditable="true"]',
            'div[role="textbox"][aria-label*="pensando"]',
            'div[contenteditable="true"][role="textbox"]'
        ];

        let found = false;
        for (const selector of textAreaSelectors) {
            try {
                const el = await this.page.waitForSelector(selector, { timeout: 5000 });
                if (el) {
                    await this.stealth.typeBiometric(this.page, selector, content);
                    found = true;
                    break;
                }
            } catch (e) {}
        }

        if (!found) throw new Error('No se pudo encontrar el editor de texto expandido.');
    }

    async clickPublish() {
        console.log('[🚀] Ejecutando publicación...');
        
        const publishSelectors = [
            'div[aria-label="Publicar"][role="button"]',
            'div[role="button"] span:text("Publicar")',
            'div[aria-label="Post"][role="button"]'
        ];

        for (const selector of publishSelectors) {
            try {
                const el = await this.page.$(selector);
                if (el) {
                    await this.stealth.clickBiometric(this.page, selector);
                    await this.page.waitForTimeout(5000);
                    return true;
                }
            } catch (e) {}
        }
        return false;
    }

    async verifyPublication() {
        // Si el composer ya no está, asumimos éxito inicial
        const composerOpen = await this.page.$('div[role="dialog"] [aria-label*="pensando"]');
        return !composerOpen;
    }

    async publish(postContent) {
        const reportPath = path.join(__dirname, 'reports/screenshots');
        fs.mkdirSync(reportPath, { recursive: true });

        try {
            await this.navigateToHome();
            const opened = await this.clickComposer();
            if (!opened) throw new Error('No se pudo abrir el composer.');
            
            await this.page.waitForTimeout(2000);
            await this.typeContent(postContent);
            await this.page.waitForTimeout(3000);
            
            await this.clickPublish();
            await this.page.waitForTimeout(5000);
            
            const success = await this.verifyPublication();
            
            const ssName = `fb_post_${Date.now()}.png`;
            const ssPath = path.join(reportPath, ssName);
            await this.page.screenshot({ path: ssPath });
            
            return {
                success,
                screenshot: ssPath,
                timestamp: new Date().toISOString()
            };
        } catch (error) {
            console.error(`[❌] Error en publicación: ${error.message}`);
            return {
                success: false,
                error: error.message,
                timestamp: new Date().toISOString()
            };
        }
    }
}

module.exports = { PostPublisher };
