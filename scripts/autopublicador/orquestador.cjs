const sqlite3 = require('sqlite3').verbose();
const { open } = require('sqlite');
const path = require('path');
const { ResearchModule } = require('./research_module.cjs');
const { FBSessionManager } = require('./fb_session_manager.cjs');
const { PostPublisher } = require('./post_publisher.cjs');

class AutopublicadorOrquestador {
    constructor() {
        this.dbPath = path.join(__dirname, '../../data/nexus_memoria.db');
        this.research = new ResearchModule();
        this.session = new FBSessionManager();
    }

    async init() {
        console.log('[🌀] Inicializando Orquestador de Publicación...');
        this.db = await open({
            filename: this.dbPath,
            driver: sqlite3.Database
        });

        await this.db.exec(`
            CREATE TABLE IF NOT EXISTS fb_autopublish_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                topic TEXT NOT NULL,
                style TEXT DEFAULT 'informativo',
                content TEXT,
                status TEXT DEFAULT 'pending',
                scheduled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                published_at DATETIME,
                screenshot_path TEXT,
                error TEXT
            )
        `);
    }

    async enqueue(topic, style = 'informativo', scheduled_at = null) {
        console.log(`[➕] Encolando post: "${topic}" (${style})`);
        const query = scheduled_at 
            ? 'INSERT INTO fb_autopublish_queue (topic, style, scheduled_at) VALUES (?, ?, ?)'
            : 'INSERT INTO fb_autopublish_queue (topic, style) VALUES (?, ?)';
        
        const params = scheduled_at ? [topic, style, scheduled_at] : [topic, style];
        const result = await this.db.run(query, params);
        return result.lastID;
    }

    async processNext() {
        const post = await this.db.get(`
            SELECT * FROM fb_autopublish_queue 
            WHERE status = 'pending' 
            AND scheduled_at <= CURRENT_TIMESTAMP 
            ORDER BY scheduled_at ASC LIMIT 1
        `);

        if (!post) return null;

        console.log(`[🚀] Procesando Post ID ${post.id}: "${post.topic}"`);
        
        await this.db.run('UPDATE fb_autopublish_queue SET status = "processing" WHERE id = ?', [post.id]);

        try {
            // 1. Generar contenido
            const content = await this.research.getFullPost(post.topic, post.style);
            
            // 2. Lanzar navegador stealth
            const { context, engine } = await this.session.launchStealthBrowser({ headless: true });
            const page = await context.newPage();

            // 3. Verificar login
            const isLogged = await this.session.verifyFBLogin(page);
            if (!isLogged) {
                throw new Error("Sesión de Facebook expirada o bloqueada.");
            }

            // 4. Publicar
            const publisher = new PostPublisher(page, engine);
            const result = await publisher.publish(content);

            if (result.success) {
                await this.db.run(`
                    UPDATE fb_autopublish_queue 
                    SET status = "completed", 
                        content = ?, 
                        published_at = CURRENT_TIMESTAMP, 
                        screenshot_path = ? 
                    WHERE id = ?
                `, [content, result.screenshot, post.id]);
                console.log(`[✅] Post ${post.id} publicado con éxito.`);
            } else {
                throw new Error(result.error || "Fallo desconocido en publicación.");
            }

            await context.close();
            return result;

        } catch (e) {
            console.error(`[❌] Error procesando post ${post.id}: ${e.message}`);
            await this.db.run(`
                UPDATE fb_autopublish_queue 
                SET status = "failed", 
                    error = ? 
                WHERE id = ?
            `, [e.message, post.id]);
            return { success: false, error: e.message };
        }
    }

    async startScheduler(intervalMs = 300000) { // Cada 5 min por defecto
        console.log(`[⏰] Scheduler activado. Latido cada ${intervalMs/1000}s.`);
        
        const run = async () => {
            try {
                const res = await this.processNext();
                if (res) {
                    // Si procesamos uno, intentamos el siguiente después de un breve cooldown humano
                    setTimeout(run, 60000); 
                } else {
                    setTimeout(run, intervalMs);
                }
            } catch (err) {
                console.error("[❌] Error en el loop del scheduler:", err);
                setTimeout(run, intervalMs);
            }
        };

        run();
    }
}

// Para ejecución directa desde CLI
if (require.main === module) {
    (async () => {
        const orch = new AutopublicadorOrquestador();
        await orch.init();
        const action = process.argv[2];
        const topic = process.argv[3];
        
        if (action === 'add' && topic) {
            const id = await orch.enqueue(topic, process.argv[4] || 'informativo');
            console.log(`Post encolado con ID: ${id}`);
        } else if (action === 'run') {
            await orch.processNext();
        } else if (action === 'start') {
            orch.startScheduler();
        } else {
            console.log('Uso: node orquestador.cjs [add "tema" [estilo] | run | start]');
        }
    })();
}

module.exports = { AutopublicadorOrquestador };
