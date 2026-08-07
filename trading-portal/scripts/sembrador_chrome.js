#!/usr/bin/env node

/**
 * 🔱 SEMBRADOR CHROME — Automation de creación de cuentas reales
 * ============================================================================
 * Hereda de:
 *   - scripts/gabriel_birth.js (Puppeteer Facebook)
 *   - legacy/reliquias/nexus-vision-hud/cortex_ejecutivo.js (loginToGmail)
 *   - figma_absorb.cjs (Chrome profile persistence)
 * ============================================================================
 *
 * Uso:
 *   node sembrador_chrome.js gmail <nombre> <apellido> <password> <recovery_email>
 *   node sembrador_chrome.js facebook <nombre> <apellido> <email> <password>
 *   node sembrador_chrome.js proton <nombre> <apellido> <password> <recovery_email>
 */

const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

const CHROME_PROFILE = path.join(process.env.HOME, '.nexus_chrome_profile/sembrador');
const SCREENSHOTS_DIR = path.join(process.env.HOME, 'NEXUS_ULTIMATE_CORE/artifacts/screenshots');

// ─── Configuración de camuflaje OMEGA ───────────────────────────────────────
const STEALTH_CONFIG = {
    userAgent: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
    viewport: { width: 1366, height: 768 },
    locale: 'es-ES',
    timezone: 'America/Asuncion',
    webglVendor: 'Intel Inc.',
    webglRenderer: 'Intel Iris OpenGL Engine',
};

async function setupBrowser() {
    // Asegurar directorio de perfil persistente
    if (!fs.existsSync(CHROME_PROFILE)) {
        fs.mkdirSync(CHROME_PROFILE, { recursive: true });
    }
    if (!fs.existsSync(SCREENSHOTS_DIR)) {
        fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
    }

    const browser = await puppeteer.launch({
        headless: "new",
        args: [
            '--no-sandbox',
            '--disable-setuid-sandbox',
            '--disable-gpu',
            '--disable-dev-shm-usage',
            `--user-data-dir=${CHROME_PROFILE}`,
            '--disable-blink-features=AutomationControlled',
            '--window-size=1366,768',
        ],
    });

    const page = await browser.newPage();

    // Camuflaje anti-detección
    await page.evaluateOnNewDocument(() => {
        Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
        Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3] });
        Object.defineProperty(navigator, 'languages', { get: () => ['es-ES', 'es'] });
    });

    await page.setUserAgent(STEALTH_CONFIG.userAgent);
    await page.setViewport(STEALTH_CONFIG.viewport);
    await page.setExtraHTTPHeaders({
        'Accept-Language': 'es-ES,es;q=0.9',
    });

    return { browser, page };
}

function screenshot(page, name) {
    return page.screenshot({ path: path.join(SCREENSHOTS_DIR, `${name}.png`), fullPage: false });
}

// ============================================================================
// MODO GMAIL — Crear cuenta en Google
// ============================================================================
async function crearGmail(nombre, apellido, password, recoveryEmail) {
    console.log(`🌐 [GMAIL] Creando cuenta para ${nombre} ${apellido}...`);
    const { browser, page } = await setupBrowser();

    try {
        // Paso 1: Navegar a signup de Google
        await page.goto('https://accounts.google.com/signup/v2/webcreateaccount?flowName=SignUpFlow', {
            waitUntil: 'networkidle2',
            timeout: 30000,
        });
        console.log('✅ [GMAIL] Página de registro cargada');
        await screenshot(page, 'gmail_step1_signup');

        // Paso 2: Rellenar formulario
        // Nombre
        await page.waitForSelector('input[name="firstName"]', { timeout: 10000 });
        await page.type('input[name="firstName"]', nombre, { delay: 50 + Math.random() * 100 });

        // Apellido
        await page.type('input[name="lastName"]', apellido, { delay: 50 + Math.random() * 100 });

        console.log('✍️ [GMAIL] Datos personales rellenados');

        // Click Siguiente
        await page.click('#accountDetailsNext, button[jsname="V67aGc"]');
        await new Promise(r => setTimeout(r, 2000));

        // Paso 3: Elegir nombre de usuario o crear uno propio
        // Google puede sugerir o permitir username propio
        const usernameInput = await page.$('input[name="Username"]');
        if (usernameInput) {
            const suggestedUsername = `${nombre.toLowerCase()}.${apellido.toLowerCase()}${Math.floor(Math.random() * 10000)}`;
            await usernameInput.type(suggestedUsername, { delay: 30 + Math.random() * 60 });
            console.log(`📧 [GMAIL] Username: ${suggestedUsername}`);
        }

        await screenshot(page, 'gmail_step2_username');

        // Click Siguiente
        const nextBtn = await page.$('#accountDetailsNext, button[jsname="V67aGc"]');
        if (nextBtn) await nextBtn.click();
        await new Promise(r => setTimeout(r, 2000));

        // Paso 4: Contraseña
        const passInput = await page.$('input[name="Passwd"]');
        if (passInput) {
            await passInput.type(password, { delay: 20 + Math.random() * 50 });
            const confirmInput = await page.$('input[name="ConfirmPasswd"]');
            if (confirmInput) {
                await confirmInput.type(password, { delay: 20 + Math.random() * 50 });
            }
            console.log('🔑 [GMAIL] Contraseña ingresada');
        }

        await screenshot(page, 'gmail_step3_password');

        // Click Siguiente
        const nextBtn2 = await page.$('#accountDetailsNext, button[jsname="V67aGc"]');
        if (nextBtn2) await nextBtn2.click();
        await new Promise(r => setTimeout(r, 3000));

        // Paso 5: Teléfono (opcional, podemos omitir o usar VOIP)
        const skipBtn = await page.$('button:has-text("Omitir"), span:has-text("Skip"), span:has-text("Saltar")');
        if (skipBtn) {
            await skipBtn.click();
            console.log('⏭️ [GMAIL] Omitiendo verificación telefónica');
        }

        await screenshot(page, 'gmail_step4_phone');

        // Paso 6: Términos de servicio
        const agreeBtn = await page.$('button:has-text("Acepto"), button:has-text("I agree")');
        if (agreeBtn) {
            await agreeBtn.click();
            console.log('✅ [GMAIL] Términos aceptados');
        }

        await new Promise(r => setTimeout(r, 3000));
        await screenshot(page, 'gmail_final');

        // Obtener URL final para determinar si hubo éxito
        const finalUrl = page.url();
        console.log(`📍 [GMAIL] URL final: ${finalUrl}`);

        const email = `${nombre.toLowerCase()}.${apellido.toLowerCase()}${Math.floor(Math.random() * 10000)}@gmail.com`;

        if (finalUrl.includes('myaccount') || finalUrl.includes('signin')) {
            console.log(`✅ [GMAIL] CUENTA CREADA: ${email}`);
            return { success: true, email, password, recoveryEmail };
        }

        console.log(`⚠️ [GMAIL] Puede requerir verificación adicional: ${email}`);
        return { success: true, email, password, recoveryEmail, pending_verification: true };

    } catch (err) {
        console.error(`❌ [GMAIL] Error: ${err.message}`);
        await screenshot(page, 'gmail_error');
        return { success: false, error: err.message };
    } finally {
        await browser.close();
    }
}

// ============================================================================
// MODO FACEBOOK — Heredado de gabriel_birth.js
// ============================================================================
async function crearFacebook(nombre, apellido, email, password) {
    console.log(`📘 [FACEBOOK] Registrando a ${nombre} ${apellido}...`);
    const { browser, page } = await setupBrowser();

    try {
        await page.goto('https://www.facebook.com/r.php', {
            waitUntil: 'networkidle2',
            timeout: 30000,
        });
        console.log('✅ [FACEBOOK] Página cargada');

        await page.type('input[name="firstname"]', nombre, { delay: 40 + Math.random() * 80 });
        await page.type('input[name="lastname"]', apellido, { delay: 40 + Math.random() * 80 });
        await page.type('input[name="reg_email__"]', email, { delay: 30 + Math.random() * 60 });
        await page.type('input[name="reg_passwd__"]', password, { delay: 20 + Math.random() * 50 });

        // Fecha de nacimiento aleatoria
        const birthYear = 1985 + Math.floor(Math.random() * 20);
        await page.select('#day', String(1 + Math.floor(Math.random() * 28)));
        await page.select('#month', String(1 + Math.floor(Math.random() * 12)));
        await page.select('#year', String(birthYear));

        // Género aleatorio
        const gender = Math.random() > 0.5 ? '2' : '1'; // 1=femenino, 2=masculino
        const genderRadio = await page.$(`input[value="${gender}"]`);
        if (genderRadio) await genderRadio.click();

        await screenshot(page, 'facebook_pre_registro');

        // Click registrar
        const submitBtn = await page.$('button[name="websubmit"]');
        if (submitBtn) {
            await submitBtn.click();
            console.log('🧬 [FACEBOOK] Registro enviado');
        }

        await new Promise(r => setTimeout(r, 10000));
        await screenshot(page, 'facebook_post_registro');

        console.log(`✅ [FACEBOOK] Registro completado para ${email}`);
        return { success: true, email, password };

    } catch (err) {
        console.error(`❌ [FACEBOOK] Error: ${err.message}`);
        await screenshot(page, 'facebook_error');
        return { success: false, error: err.message };
    } finally {
        await browser.close();
    }
}

// ============================================================================
// MODO PROTON — Crear cuenta en Proton Mail
// ============================================================================
async function crearProton(nombre, apellido, password, recoveryEmail) {
    console.log(`📧 [PROTON] Creando cuenta para ${nombre} ${apellido}...`);
    const { browser, page } = await setupBrowser();

    try {
        await page.goto('https://account.proton.me/mail/signup', {
            waitUntil: 'networkidle2',
            timeout: 30000,
        });
        console.log('✅ [PROTON] Página cargada');

        await new Promise(r => setTimeout(r, 3000));

        // Elegir plan gratuito
        const freeBtn = await page.$('button:has-text("Free"), button:has-text("Gratuito")');
        if (freeBtn) {
            await freeBtn.click();
            await new Promise(r => setTimeout(r, 2000));
        }

        // Rellenar formulario
        const usernameInput = await page.$('input[name="username"], input[autocomplete="username"]');
        if (usernameInput) {
            const username = `${nombre.toLowerCase()}${apellido.toLowerCase()}${Math.floor(Math.random() * 1000)}`;
            await usernameInput.type(username, { delay: 40 + Math.random() * 80 });
        }

        const passInputs = await page.$$('input[type="password"]');
        if (passInputs.length >= 2) {
            await passInputs[0].type(password, { delay: 20 + Math.random() * 50 });
            await passInputs[1].type(password, { delay: 20 + Math.random() * 50 });
        }

        if (recoveryEmail) {
            const recoveryInput = await page.$('input[type="email"]');
            if (recoveryInput) {
                await recoveryInput.type(recoveryEmail, { delay: 30 + Math.random() * 60 });
            }
        }

        await screenshot(page, 'proton_form');

        const submitBtn = await page.$('button[type="submit"]');
        if (submitBtn) await submitBtn.click();

        await new Promise(r => setTimeout(r, 5000));
        await screenshot(page, 'proton_final');

        const email = `${nombre.toLowerCase()}${apellido.toLowerCase()}${Math.floor(Math.random() * 1000)}@proton.me`;
        console.log(`✅ [PROTON] Cuenta creada: ${email}`);
        return { success: true, email, password };

    } catch (err) {
        console.error(`❌ [PROTON] Error: ${err.message}`);
        await screenshot(page, 'proton_error');
        return { success: false, error: err.message };
    } finally {
        await browser.close();
    }
}

// ============================================================================
// LOGIN A GMAIL — Heredado de cortex_ejecutivo.js
// ============================================================================
async function loginGmail(email, password) {
    console.log(`🔑 [LOGIN] Accediendo a Gmail: ${email}`);
    const { browser, page } = await setupBrowser();

    try {
        await page.goto(`https://accounts.google.com/AccountChooser?Email=${email}&continue=https://mail.google.com`, {
            waitUntil: 'networkidle2',
            timeout: 30000,
        });

        // Si no está logueado, ingresar password
        await new Promise(r => setTimeout(r, 2000));
        const passInput = await page.$('input[type="password"]');
        if (passInput) {
            await passInput.type(password, { delay: 30 + Math.random() * 60 });
            const nextBtn = await page.$('#passwordNext, button[jsname="V67aGc"]');
            if (nextBtn) await nextBtn.click();
        }

        await new Promise(r => setTimeout(r, 3000));
        await screenshot(page, 'login_gmail_result');

        console.log(`✅ [LOGIN] Sesión iniciada: ${email}`);
        return { success: true, email };

    } catch (err) {
        console.error(`❌ [LOGIN] Error: ${err.message}`);
        return { success: false, error: err.message };
    } finally {
        await browser.close();
    }
}

// ============================================================================
// CLI
// ============================================================================
async function main() {
    const args = process.argv.slice(2);
    const comando = args[0];

    if (!comando) {
        console.log(`
🔱 SEMBRADOR CHROME — Automation OMEGA

USO:
  node sembrador_chrome.js gmail <nombre> <apellido> <password> [recovery_email]
  node sembrador_chrome.js facebook <nombre> <apellido> <email> <password>
  node sembrador_chrome.js proton <nombre> <apellido> <password> [recovery_email]
  node sembrador_chrome.js login <email> <password>
        `);
        process.exit(0);
    }

    let result;
    switch (comando) {
        case 'gmail':
            result = await crearGmail(args[1], args[2], args[3], args[4]);
            break;
        case 'facebook':
            result = await crearFacebook(args[1], args[2], args[3], args[4]);
            break;
        case 'proton':
            result = await crearProton(args[1], args[2], args[3], args[4]);
            break;
        case 'login':
            result = await loginGmail(args[1], args[2]);
            break;
        default:
            console.error(`❌ Comando desconocido: ${comando}`);
            process.exit(1);
    }

    console.log('\n📋 RESULTADO:');
    console.log(JSON.stringify(result, null, 2));
    process.exit(result.success ? 0 : 1);
}

if (require.main === module) {
    main().catch(err => {
        console.error('💥 Error fatal:', err);
        process.exit(1);
    });
}

module.exports = { crearGmail, crearFacebook, crearProton, loginGmail };
