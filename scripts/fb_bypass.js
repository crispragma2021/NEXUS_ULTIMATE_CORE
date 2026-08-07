// scripts/fb_bypass.js
// 🔱 NEXUS OMEGA - Script de Infiltración y Limpieza de DOM para Facebook

const puppeteer = require('puppeteer');

// Pool de User-Agents comunes para Chrome en Windows/Linux
const USER_AGENTS = [
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36',
  'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
  'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
];

// Función para obtener un User-Agent aleatorio
function getRandomUserAgent() {
  return USER_AGENTS[Math.floor(Math.random() * USER_AGENTS.length)];
}

// Función para introducir un retardo aleatorio (jitter)
function randomDelay(min, max) {
  return new Promise(resolve => setTimeout(resolve, Math.random() * (max - min) + min));
}

(async () => {
  const useTor = process.argv.includes('--tor');
  const proxyArgs = useTor ? ['--proxy-server=socks5://127.0.0.1:9050'] : [];

  const browser = await puppeteer.launch({
    headless: "new",
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu', ...proxyArgs]
  });
  const page = await browser.newPage();
  
  // Establecer un User-Agent aleatorio
  await page.setUserAgent(getRandomUserAgent());

  console.log("🚀 Navegando al perfil del Arquitecto...");
  await page.goto('https://www.facebook.com/profile.php?id=100077294943674', { waitUntil: 'networkidle2' });

  await randomDelay(1000, 3000); // Jitter antes de la limpieza del DOM

  console.log("🧹 Ejecutando Bypass de Modales y Limpieza de DOM...");
  await page.evaluate(() => {
    // 1. Eliminar modales de login/registro
    const selectors = [
      'div[role="dialog"]', 
      'div[id^="login_"]', 
      'div[class*="login"]',
      'div[aria-label*="Log In"]',
      'div[aria-label*="Iniciar sesión"]'
    ];
    
    selectors.forEach(s => {
      document.querySelectorAll(s).forEach(el => el.remove());
    });

    // 2. Restaurar el scroll del cuerpo (Facebook lo bloquea cuando sale el modal)
    document.body.style.overflow = 'visible';
    document.documentElement.style.overflow = 'visible';
  });

  // Esperar un momento para que el layout se estabilice (jitter aplicado)
  await randomDelay(1500, 3500);

  console.log("📸 Disparando Visión Omega...");
  await page.screenshot({ path: '/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/facebook_profile_clean.png', fullPage: true });

  console.log("✅ Captura limpia manifestada.");
  await browser.close();
})();
