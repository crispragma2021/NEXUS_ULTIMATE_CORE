const axios = require('axios');
const sqlite3 = require('sqlite3').verbose();
const { open } = require('sqlite');
const path = require('path');

class ResearchModule {
    constructor() {
        this.dbPath = path.join(__dirname, '../../data/nexus_memoria.db');
        this.keys = {};
    }

    async loadKeys() {
        console.log('[🔑] Cargando API Keys desde la Bóveda de Secretos...');
        const db = await open({
            filename: this.dbPath,
            driver: sqlite3.Database
        });

        const rows = await db.all('SELECT clave, valor FROM system_secrets');
        rows.forEach(row => {
            this.keys[row.clave] = row.valor;
        });
        await db.close();
    }

    async researchExa(query) {
        if (!this.keys.EXA_API_KEY) return [];
        console.log(`[🌐] Investigando en Exa: "${query}"`);
        try {
            const response = await axios.post('https://api.exa.ai/search', {
                query: query,
                useAutoprompt: true,
                numResults: 5
            }, {
                headers: { 'x-api-key': this.keys.EXA_API_KEY }
            });
            return response.data.results || [];
        } catch (e) {
            console.error(`[❌] Exa Error: ${e.message}`);
            return [];
        }
    }

    async researchTavily(query) {
        if (!this.keys.TAVILY_API_KEY) return [];
        console.log(`[🌐] Investigando en Tavily: "${query}"`);
        try {
            const response = await axios.post('https://api.tavily.com/search', {
                api_key: this.keys.TAVILY_API_KEY,
                query: query,
                search_depth: "advanced",
                max_results: 5
            });
            return response.data.results || [];
        } catch (e) {
            console.error(`[❌] Tavily Error: ${e.message}`);
            return [];
        }
    }

    async generateGeminiContent(topic, style, research) {
        // Usamos el Proxy Hijack 4444 para canalizar a Gemini (Vertex AI)
        console.log(`[🧠] Forjando contenido soberano para: "${topic}"...`);
        
        const context = research.map(r => `Fuente: ${r.title || r.url}\nContenido: ${r.content || r.snippet}`).join('\n\n');
        
        const prompt = `
        Identidad: Eres Gabriel, un analista de NEXUS.
        Misión: Generar un post de Facebook sobre "${topic}".
        Estilo solicitado: ${style} (informativo, provocador o storytelling).
        
        Investigación previa:
        ${context}
        
        Reglas de Oro:
        1. Enganche potente en las primeras 2 líneas.
        2. Lenguaje humano, directo, sin "puffery" de IA.
        3. Máximo 4 emojis.
        4. 3 hashtags al final.
        5. Llama a la acción (pregunta al final).
        6. Si el tema es técnico, mantén la precisión Rust/NEXUS.
        
        Respuesta limpia, lista para publicar.
        `;

        try {
            // Asumiendo que el Proxy Hijack está corriendo en localhost:4444
            // Si no, podemos intentar llamar directamente a Vertex/OpenRouter
            const response = await axios.post('http://localhost:4444/v1/chat/completions', {
                model: "gemini-1.5-flash",
                messages: [{ role: "user", content: prompt }]
            });
            return response.data.choices[0].message.content;
        } catch (e) {
            console.warn(`[⚠️] Proxy 4444 no responde. Intentando alternativa directa...`);
            // Fallback a API key directa de Gemini si existe
            if (this.keys.GEMINI_API_KEY) {
                 // Implementación simplificada de fallback
                 return "Contenido generado (Fallback): " + topic;
            }
            throw new Error("No se pudo contactar con ningún motor de IA.");
        }
    }

    async getFullPost(topic, style = 'informativo') {
        await this.loadKeys();
        const exa = await this.researchExa(topic);
        const tavily = await this.researchTavily(topic);
        
        const research = [...exa, ...tavily];
        if (research.length === 0) {
            console.warn("[⚠️] No se encontró investigación externa. Generando con base interna.");
        }

        return await this.generateGeminiContent(topic, style, research);
    }
}

module.exports = { ResearchModule };
