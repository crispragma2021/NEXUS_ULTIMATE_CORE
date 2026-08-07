const fs = require('fs');
const { GoogleGenAI } = require('@google/genai');

const imagePath = process.argv[2] || '/tmp/nexus_vite_screenshot.png';

// Credenciales y configuración (heredan del entorno o valores por defecto del proyecto).
const projectId = process.env.GOOGLE_CLOUD_PROJECT || 'project-26e94ab7-4257-4475-ade';
const location = process.env.GOOGLE_CLOUD_LOCATION || 'us-central1';
// Modelo multimodal actual de Vertex AI (gemini-1.0-pro-vision fue removido → 404).
// Configurable por env var VISION_MODEL para futuras migraciones.
const model = process.env.VISION_MODEL || 'gemini-2.5-flash';

async function analyzeImage(path) {
    try {
        if (!fs.existsSync(path)) {
            console.error(`Error: El archivo ${path} no existe.`);
            process.exit(1);
        }

        const imageBuffer = fs.readFileSync(path);
        const base64Image = imageBuffer.toString('base64');

        console.log(`[Vision Bridge] Enviando imagen ${path} para análisis a Vertex AI (modelo ${model})...`);

        // SDK moderno @google/genai — Gemini Enterprise Agent Platform (Vertex AI).
        const ai = new GoogleGenAI({ enterprise: true, project: projectId, location });

        const response = await ai.models.generateContent({
            model,
            contents: [
                {
                    role: 'user',
                    parts: [
                        { text: 'Eres NEXUS (Vision Bridge). Analiza esta captura de un terminal de trading/webapp. Reporta en detalle: (1) elementos de UI visibles y su estado, (2) textos cortados o truncados, (3) elementos superpuestos o desalineados, (4) contraste/legibilidad deficiente, (5) valores numericos mostrados. Sé preciso y conciso, en español.' },
                        { type: 'image', data: base64Image, mime_type: 'image/png' }
                    ],
                },
            ],
        });

        const description = response.text;
        console.log("--- ANÁLISIS MULTIMODAL GEMINI (Vertex AI) ---");
        console.log(description);
        console.log("--- FIN ANÁLISIS ---");

    } catch (error) {
        console.error('[Vision Bridge] Error en la llamada a Vertex AI:', error.message);
        if (error.response && error.response.data) {
            console.error('API Response Data:', JSON.stringify(error.response.data));
        } else if (error.details) {
            console.error('API Details:', error.details);
        } else if (error.body) {
            console.error('API Response Body:', error.body);
        }
        process.exit(1);
    }
}

analyzeImage(imagePath);
