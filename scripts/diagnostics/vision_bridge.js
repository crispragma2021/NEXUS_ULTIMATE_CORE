import fs from 'fs';
import path from 'path';

// Parsear manualmente el archivo .env sin dependencias externas
let OPENROUTER_API_KEY = null;
try {
  const envPath = path.join(process.cwd(), '.env');
  if (fs.existsSync(envPath)) {
    const envContent = fs.readFileSync(envPath, 'utf8');
    const match = envContent.match(/^OPENROUTER_API_KEY\s*=\s*([^\s#]+)/m);
    if (match) {
      OPENROUTER_API_KEY = match[1];
    }
  }
} catch (e) {
  console.error('⚠️ Error leyendo .env manualmente:', e.message);
}

if (!OPENROUTER_API_KEY) {
  console.error('❌ Error: OPENROUTER_API_KEY no encontrada en el archivo .env');
  process.exit(1);
}

// Ruta por defecto del screenshot
const defaultScreenshot = '/tmp/nexus_vite_screenshot.png';
const imagePath = process.argv[2] || defaultScreenshot;

if (!fs.existsSync(imagePath)) {
  console.error(`❌ Error: El archivo de imagen no existe en: ${imagePath}`);
  process.exit(1);
}

console.log(`👁️ Analizando imagen via OpenRouter: ${imagePath}...`);

// Convertir la imagen a base64
const imageBase64 = fs.readFileSync(imagePath).toString('base64');
const dataUrl = `data:image/png;base64,${imageBase64}`;

const url = 'https://openrouter.ai/api/v1/chat/completions';

const payload = {
  model: 'google/gemini-2.5-flash',
  messages: [
    {
      role: 'user',
      content: [
        {
          type: 'text',
          text: "Analiza detalladamente esta captura de pantalla de la interfaz de usuario. Busca y reporta de forma concisa:\n" +
                "1. Errores visuales claros (elementos rotos, textos superpuestos, cajas mal alineadas).\n" +
                "2. Estado del dashboard (si se muestran gráficos, tablas de trading, alertas o menús).\n" +
                "3. Cualquier asimetría o problema de diseño/estética que deba corregirse.\n" +
                "Responde en español con viñetas cortas y pragmáticas."
        },
        {
          type: 'image_url',
          image_url: {
            url: dataUrl
          }
        }
      ]
    }
  ]
};

async function analyze() {
  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${OPENROUTER_API_KEY}`,
        'HTTP-Referer': 'https://github.com/nexus-ultimate-core',
        'X-Title': 'NEXUS Vision Bridge'
      },
      body: JSON.stringify(payload)
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`OpenRouter respondió con estado ${response.status}: ${errorText}`);
    }

    const data = await response.json();
    const description = data.choices?.[0]?.message?.content;

    if (!description) {
      throw new Error('La respuesta de la API no contiene texto en choices[0].message.content');
    }

    console.log('\n================ REPORTES DE LA VISTA (OJOS DE GEMINI A TRAVÉS DE OPENROUTER) ================');
    console.log(description);
    console.log('==============================================================================================\n');
  } catch (error) {
    console.error('❌ Error llamando a la API de visión:', error.message);
    process.exit(1);
  }
}

analyze();
