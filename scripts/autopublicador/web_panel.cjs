const express = require('express');
const cors = require('cors');
const path = require('path');
const { AutopublicadorOrquestador } = require('./orquestador.cjs');

const app = express();
const port = 5180;
const orquestador = new AutopublicadorOrquestador();

app.use(cors());
app.use(express.json());

// Servir screenshots de forma estática
app.use('/screenshots', express.static(path.join(__dirname, 'reports/screenshots')));

// API: Encolar post
app.post('/api/queue', async (req, res) => {
    const { topic, style, scheduled_at } = req.body;
    if (!topic) return res.status(400).json({ error: "Topic es requerido" });

    try {
        const id = await orquestador.enqueue(topic, style, scheduled_at);
        res.json({ success: true, id });
    } catch (e) {
        res.status(500).json({ error: e.message });
    }
});

// API: Listar cola
app.get('/api/queue', async (req, res) => {
    try {
        const rows = await orquestador.db.all('SELECT * FROM fb_autopublish_queue ORDER BY scheduled_at DESC LIMIT 50');
        res.json(rows);
    } catch (e) {
        res.status(500).json({ error: e.message });
    }
});

// API: Procesar ahora
app.post('/api/process-now', async (req, res) => {
    try {
        const result = await orquestador.processNext();
        res.json(result || { message: "No hay posts pendientes" });
    } catch (e) {
        res.status(500).json({ error: e.message });
    }
});

// Servir panel HTML (Interface Profesional v2.0)
app.get('/', (req, res) => {
    res.send(`
<!DOCTYPE html>
<html lang="es" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NEXUS - FB Autopublicador v2.0</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/lucide@latest"></script>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;600;700&family=Fira+Code:wght@400;500&display=swap" rel="stylesheet">
    <script>
        tailwind.config = {
            darkMode: 'class',
            theme: {
                extend: {
                    colors: {
                        nexus: {
                            bg: '#020617',
                            card: '#0f172a',
                            emerald: '#10b981',
                            cyan: '#06b6d4',
                            border: '#1e293b',
                            text: '#e2e8f0',
                            muted: '#94a3b8'
                        }
                    },
                    fontFamily: {
                        sans: ['Outfit', 'sans-serif'],
                        mono: ['Fira Code', 'monospace']
                    }
                }
            }
        }
    </script>
    <style>
        body { background-color: #020617; color: #e2e8f0; font-family: 'Outfit', sans-serif; }
        .glass { background: rgba(15, 23, 42, 0.7); backdrop-filter: blur(12px); border: 1px solid rgba(30, 41, 59, 0.5); }
        .emerald-glow { box-shadow: 0 0 15px rgba(16, 185, 129, 0.1); }
        .emerald-border-glow:focus { border-color: #10b981; box-shadow: 0 0 10px rgba(16, 185, 129, 0.2); outline: none; }
        ::-webkit-scrollbar { width: 6px; }
        ::-webkit-scrollbar-track { background: #020617; }
        ::-webkit-scrollbar-thumb { background: #1e293b; border-radius: 10px; }
        ::-webkit-scrollbar-thumb:hover { background: #10b981; }
    </style>
</head>
<body class="flex min-h-screen overflow-hidden">

    <!-- Sidebar -->
    <aside class="w-64 border-r border-nexus-border flex flex-col glass z-10">
        <div class="p-6 border-b border-nexus-border flex items-center gap-3">
            <div class="w-8 h-8 bg-nexus-emerald rounded-lg flex items-center justify-center emerald-glow">
                <i data-lucide="shield-check" class="text-nexus-bg w-5 h-5"></i>
            </div>
            <span class="font-bold text-xl tracking-tight text-white">NEXUS<span class="text-nexus-emerald">.FB</span></span>
        </div>
        
        <nav class="flex-1 p-4 space-y-2 mt-4">
            <a href="#" class="flex items-center gap-3 p-3 rounded-xl bg-nexus-emerald/10 text-nexus-emerald font-semibold border border-nexus-emerald/20 transition-all">
                <i data-lucide="layout-dashboard" class="w-5 h-5"></i> Dashboard
            </a>
            <a href="#" class="flex items-center gap-3 p-3 rounded-xl text-nexus-muted hover:bg-white/5 hover:text-white transition-all">
                <i data-lucide="calendar" class="w-5 h-5"></i> Programador
            </a>
            <a href="#" class="flex items-center gap-3 p-3 rounded-xl text-nexus-muted hover:bg-white/5 hover:text-white transition-all">
                <i data-lucide="history" class="w-5 h-5"></i> Historial
            </a>
            <a href="#" class="flex items-center gap-3 p-3 rounded-xl text-nexus-muted hover:bg-white/5 hover:text-white transition-all">
                <i data-lucide="settings" class="w-5 h-5"></i> Configuración
            </a>
        </nav>

        <div class="p-4 border-t border-nexus-border">
            <div class="bg-nexus-bg/50 p-4 rounded-xl border border-nexus-border">
                <div class="flex items-center justify-between mb-2">
                    <span class="text-xs font-semibold text-nexus-muted uppercase tracking-wider">Sesión Gabriel</span>
                    <span class="flex h-2 w-2 rounded-full bg-nexus-emerald animate-pulse"></span>
                </div>
                <div class="text-sm text-white font-mono">STATUS: ONLINE</div>
                <div class="mt-2 h-1 w-full bg-nexus-border rounded-full overflow-hidden">
                    <div class="h-full bg-nexus-emerald w-4/5"></div>
                </div>
            </div>
        </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col overflow-y-auto bg-[url('https://www.transparenttextures.com/patterns/carbon-fibre.png')]">
        
        <!-- Header -->
        <header class="h-20 border-b border-nexus-border glass flex items-center justify-between px-8 shrink-0">
            <div>
                <h1 class="text-2xl font-bold text-white">Centro de Operaciones Soberano</h1>
                <p class="text-nexus-muted text-sm">Control agéntico de presencia digital</p>
            </div>
            <div class="flex items-center gap-4">
                <div class="flex -space-x-2">
                    <div class="w-10 h-10 rounded-full border-2 border-nexus-bg bg-nexus-emerald flex items-center justify-center text-nexus-bg font-bold">G</div>
                    <div class="w-10 h-10 rounded-full border-2 border-nexus-bg bg-nexus-cyan flex items-center justify-center text-nexus-bg font-bold">N</div>
                </div>
                <button class="bg-nexus-emerald text-nexus-bg font-bold px-4 py-2 rounded-lg hover:bg-nexus-emerald/90 transition-all flex items-center gap-2">
                    <i data-lucide="plus" class="w-4 h-4"></i> Nuevo Despliegue
                </button>
            </div>
        </header>

        <div class="p-8 space-y-8 max-w-7xl mx-auto w-full">
            
            <!-- Metrics Grid -->
            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                <div class="glass p-6 rounded-2xl border-nexus-border relative overflow-hidden group border border-nexus-emerald/20">
                    <div class="absolute -right-4 -bottom-4 text-nexus-emerald/5 group-hover:scale-110 transition-transform">
                        <i data-lucide="send" class="w-24 h-24"></i>
                    </div>
                    <div class="text-nexus-muted text-sm font-semibold uppercase tracking-wider mb-2">Total Posts</div>
                    <div class="text-3xl font-bold text-white font-mono">1,248</div>
                    <div class="text-nexus-emerald text-xs mt-2 flex items-center gap-1">
                        <i data-lucide="trending-up" class="w-3 h-3"></i> +12% esta semana
                    </div>
                </div>
                <div class="glass p-6 rounded-2xl border-nexus-border relative overflow-hidden group">
                    <div class="absolute -right-4 -bottom-4 text-nexus-cyan/5 group-hover:scale-110 transition-transform">
                        <i data-lucide="users" class="w-24 h-24"></i>
                    </div>
                    <div class="text-nexus-muted text-sm font-semibold uppercase tracking-wider mb-2">Alcance Total</div>
                    <div class="text-3xl font-bold text-white font-mono">42.5k</div>
                    <div class="text-nexus-cyan text-xs mt-2 flex items-center gap-1">
                        <i data-lucide="trending-up" class="w-3 h-3"></i> +5.2k hoy
                    </div>
                </div>
                <div class="glass p-6 rounded-2xl border-nexus-border relative overflow-hidden group">
                    <div class="absolute -right-4 -bottom-4 text-nexus-emerald/5 group-hover:scale-110 transition-transform">
                        <i data-lucide="check-circle" class="w-24 h-24"></i>
                    </div>
                    <div class="text-nexus-muted text-sm font-semibold uppercase tracking-wider mb-2">Tasa de Éxito</div>
                    <div class="text-3xl font-bold text-white font-mono">99.2%</div>
                    <div class="text-nexus-emerald text-xs mt-2 flex items-center gap-1">
                        <i data-lucide="shield" class="w-3 h-3"></i> Anti-Detección Activo
                    </div>
                </div>
                <div class="glass p-6 rounded-2xl border-nexus-border relative overflow-hidden group">
                    <div class="absolute -right-4 -bottom-4 text-amber-500/5 group-hover:scale-110 transition-transform">
                        <i data-lucide="clock" class="w-24 h-24"></i>
                    </div>
                    <div class="text-nexus-muted text-sm font-semibold uppercase tracking-wider mb-2">En Cola</div>
                    <div id="metricQueue" class="text-3xl font-bold text-white font-mono">0</div>
                    <div class="text-amber-500 text-xs mt-2 flex items-center gap-1">
                        <i data-lucide="alert-circle" class="w-3 h-3"></i> Próximo en 5 min
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                
                <!-- Composer Card -->
                <div class="lg:col-span-1 space-y-6">
                    <div class="glass p-6 rounded-3xl border-nexus-border shadow-2xl relative overflow-hidden">
                        <div class="absolute top-0 right-0 p-4 opacity-10">
                            <i data-lucide="pen-tool" class="w-12 h-12 text-nexus-emerald"></i>
                        </div>
                        <h2 class="text-lg font-bold text-white mb-6 flex items-center gap-2">
                            <i data-lucide="sparkles" class="text-nexus-emerald w-5 h-5"></i> 
                            Generador de Contenido
                        </h2>
                        
                        <div class="space-y-4">
                            <div>
                                <label class="text-xs font-semibold text-nexus-muted uppercase tracking-widest block mb-2">Tema Estratégico</label>
                                <input type="text" id="topic" placeholder="Ej: El futuro de Rust en Trading" 
                                    class="w-full bg-nexus-bg border border-nexus-border rounded-xl p-4 text-white placeholder:text-nexus-muted/50 emerald-border-glow transition-all font-medium">
                            </div>
                            
                            <div>
                                <label class="text-xs font-semibold text-nexus-muted uppercase tracking-widest block mb-2">Estilo de Narrativa</label>
                                <div class="grid grid-cols-3 gap-2">
                                    <button onclick="setStyle('informativo', this)" class="style-btn active bg-nexus-emerald text-nexus-bg font-bold p-2 rounded-lg text-xs transition-all">Informativo</button>
                                    <button onclick="setStyle('provocador', this)" class="style-btn border border-nexus-border text-nexus-muted p-2 rounded-lg text-xs hover:text-white transition-all">Provocador</button>
                                    <button onclick="setStyle('storytelling', this)" class="style-btn border border-nexus-border text-nexus-muted p-2 rounded-lg text-xs hover:text-white transition-all">Storytelling</button>
                                </div>
                                <input type="hidden" id="style" value="informativo">
                            </div>

                            <div class="pt-4 flex flex-col gap-3">
                                <button onclick="enqueue()" class="w-full bg-nexus-emerald text-nexus-bg font-extrabold py-4 rounded-2xl emerald-glow hover:scale-[1.02] active:scale-95 transition-all flex items-center justify-center gap-3">
                                    <i data-lucide="plus-circle" class="w-5 h-5"></i> ENCOLAR POST
                                </button>
                                <button onclick="processNow()" class="w-full bg-nexus-cyan/10 text-nexus-cyan border border-nexus-cyan/30 font-bold py-3 rounded-2xl hover:bg-nexus-cyan/20 transition-all flex items-center justify-center gap-2">
                                    <i data-lucide="zap" class="w-4 h-4"></i> Ejecutar Inmediato
                                </button>
                            </div>
                        </div>
                    </div>

                    <!-- Session Card -->
                    <div class="glass p-6 rounded-3xl border-nexus-border overflow-hidden">
                        <h3 class="text-sm font-bold text-white mb-4 uppercase tracking-widest">Logs de Inteligencia</h3>
                        <div id="miniLogs" class="space-y-3 font-mono text-[10px]">
                            <div class="text-nexus-emerald">[OK] Sistema Inmune Verificado</div>
                            <div class="text-nexus-cyan">[INFO] Cargando Bóveda de Secretos</div>
                            <div class="text-nexus-muted">[WAIT] Esperando Trigger Cron</div>
                        </div>
                    </div>
                </div>

                <!-- Queue Table Card -->
                <div class="lg:col-span-2">
                    <div class="glass rounded-3xl border-nexus-border overflow-hidden h-full flex flex-col">
                        <div class="p-6 border-b border-nexus-border flex items-center justify-between">
                            <h2 class="text-lg font-bold text-white flex items-center gap-2">
                                <i data-lucide="list-ordered" class="text-nexus-cyan w-5 h-5"></i> 
                                Cola de Publicaciones
                            </h2>
                            <button onclick="loadQueue()" class="p-2 hover:bg-white/5 rounded-full transition-all text-nexus-muted hover:text-nexus-emerald">
                                <i data-lucide="refresh-cw" class="w-5 h-5" id="refreshIcon"></i>
                            </button>
                        </div>
                        
                        <div class="flex-1 overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="text-nexus-muted text-[10px] uppercase tracking-[0.2em] border-b border-nexus-border bg-white/5">
                                        <th class="px-6 py-4 font-semibold">ID</th>
                                        <th class="px-6 py-4 font-semibold">Tema Estratégico</th>
                                        <th class="px-6 py-4 font-semibold">Estado</th>
                                        <th class="px-6 py-4 font-semibold">Programado</th>
                                        <th class="px-6 py-4 font-semibold text-right">Resultado</th>
                                    </tr>
                                </thead>
                                <tbody id="queueTable" class="divide-y divide-nexus-border">
                                    <!-- Rows will be injected here -->
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </main>

    <script>
        lucide.createIcons();

        function setStyle(val, btn) {
            document.getElementById('style').value = val;
            document.querySelectorAll('.style-btn').forEach(b => {
                b.classList.remove('active', 'bg-nexus-emerald', 'text-nexus-bg', 'font-bold');
                b.classList.add('border', 'border-nexus-border', 'text-nexus-muted');
            });
            btn.classList.add('active', 'bg-nexus-emerald', 'text-nexus-bg', 'font-bold');
            btn.classList.remove('border', 'border-nexus-border', 'text-nexus-muted');
        }

        async function loadQueue() {
            const icon = document.getElementById('refreshIcon');
            icon.classList.add('animate-spin');
            
            try {
                const res = await fetch('/api/queue');
                const data = await res.json();
                document.getElementById('metricQueue').innerText = data.filter(r => r.status === 'pending').length;
                
                const tbody = document.getElementById('queueTable');
                tbody.innerHTML = data.map(row => {
                    const statusColors = {
                        completed: 'bg-nexus-emerald/10 text-nexus-emerald border-nexus-emerald/30',
                        pending: 'bg-amber-500/10 text-amber-500 border-amber-500/30',
                        failed: 'bg-red-500/10 text-red-500 border-red-500/30',
                        processing: 'bg-nexus-cyan/10 text-nexus-cyan border-nexus-cyan/30 animate-pulse'
                    };
                    const color = statusColors[row.status] || 'bg-slate-500/10 text-slate-500';

                    return \`
                        <tr class="hover:bg-white/[0.02] transition-colors group">
                            <td class="px-6 py-5 font-mono text-xs text-nexus-muted">#\${row.id.toString().padStart(4, '0')}</td>
                            <td class="px-6 py-5 font-semibold text-white">\${row.topic}</td>
                            <td class="px-6 py-5">
                                <span class="px-3 py-1 rounded-full text-[10px] font-bold border \${color} uppercase tracking-wider">
                                    \${row.status}
                                </span>
                            </td>
                            <td class="px-6 py-5 text-sm text-nexus-muted font-mono">\${new Date(row.scheduled_at).toLocaleTimeString()}</td>
                            <td class="px-6 py-5 text-right">
                                \${row.screenshot_path ? 
                                    \`<a href="/screenshots/\${row.screenshot_path.split('/').pop()}" target="_blank" class="text-nexus-cyan hover:underline inline-flex items-center gap-2 text-xs font-bold">
                                        <i data-lucide="image" class="w-4 h-4"></i> VER CAPTURA
                                    </a>\` : '<span class="text-nexus-muted/30">N/A</span>'}
                            </td>
                        </tr>
                    \`;
                }).join('');
                lucide.createIcons();
            } finally {
                setTimeout(() => icon.classList.remove('animate-spin'), 500);
            }
        }

        async function enqueue() {
            const topic = document.getElementById('topic').value;
            const style = document.getElementById('style').value;
            if(!topic) return alert("Ingresa un tema estratégico");
            
            await fetch('/api/queue', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ topic, style })
            });
            document.getElementById('topic').value = '';
            loadQueue();
        }

        async function processNow() {
            await fetch('/api/process-now', { method: 'POST' });
            loadQueue();
        }

        loadQueue();
        setInterval(loadQueue, 10000);
    </script>
</body>
</html>
    `);
});

async function start() {
    await orquestador.init();
    app.listen(port, () => {
        console.log("NEXUS SOBERANO v2.0 - Panel Online");
    });
}

start();
