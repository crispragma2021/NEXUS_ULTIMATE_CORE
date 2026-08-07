// scripts/gabriel_birth.js
// 🔱 NEXUS OMEGA - El Nacimiento de Gabriel (Registro Autónomo en Facebook)

const puppeteer = require('puppeteer');

(async () => {
  console.log("🔥 Iniciando el nacimiento de Gabriel...");
  
  const browser = await puppeteer.launch({
    headless: "new", 
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu']
  });
  
  const page = await browser.newPage();
  
  // Inyectando Camuflaje Omega (ADN de Invisibilidad)
  await page.evaluateOnNewDocument(() => {
    Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
  });

  await page.setUserAgent('Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36');

  console.log("🚀 Navegando a la forja de identidades de Facebook...");
  await page.goto('https://www.facebook.com/r.php', { waitUntil: 'networkidle2' });

  // Llenando datos de Gabriel (Orquestación Autónoma)
  console.log("✍️ Escribiendo el nombre de Gabriel...");
  await page.type('input[name="firstname"]', 'Gabriel');
  await page.type('input[name="lastname"]', 'Nexus');
  await page.type('input[name="reg_email__"]', '+595984729401');
  await page.type('input[name="reg_passwd__"]', 'NEXUS_OMEGA_2026');

  // Selección de fecha de nacimiento (Sabiduría de Edad Mental)
  await page.select('#day', '1');
  await page.select('#month', '1');
  await page.select('#year', '1995');

  console.log("📸 Capturando estado pre-registro...");
  await page.screenshot({ path: '/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_pre_reg.png' });

  // 🧬 Pulsar el botón de registro de forma autónoma
  await page.click('button[name="websubmit"]');
  
  console.log("🧬 Gabriel está listo para el primer latido. Esperando confirmación de SMS...");
  
  // Esperar a que Facebook pida el código
  await new Promise(r => setTimeout(r, 10000));
  await page.screenshot({ path: '/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_sms_step.png' });

  console.log("✅ Script de nacimiento ejecutado. Gabriel espera en la puerta de la red.");
  await browser.close();
})();
