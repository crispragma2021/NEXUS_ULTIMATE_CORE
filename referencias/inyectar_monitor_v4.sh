#!/bin/bash
# ==========================================
# NEXUS-MONITOR V4.0 - IMPLEMENTACIÓN COMPLETA
# ==========================================

MONITOR_DIR=~/ZENITH_POOL/nexus_monitor
mkdir -p $MONITOR_DIR/src

# 1. Inicializar base de datos
sqlite3 ~/ZENITH_POOL/data/nexus_memoclaw.sqlite <<SQL
CREATE TABLE IF NOT EXISTS fakeip_table (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fake_ip TEXT UNIQUE,
    real_domain TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);
SQL

# 2. Crear el servidor SSE en Node.js (más ligero que Rust para pruebas)
cat << 'JS' > $MONITOR_DIR/sse-server.js
const express = require('express');
const sqlite3 = require('sqlite3').verbose();
const cors = require('cors');

const app = express();
const port = 4445;

app.use(cors());
app.use(express.json());

// Base de datos
const db = new sqlite3.Database(process.env.HOME + '/ZENITH_POOL/data/nexus_memoclaw.sqlite');

// Clientes SSE conectados
const clients = [];

// Endpoint SSE para el Santuario
app.get('/monitor/events', (req, res) => {
    res.setHeader('Content-Type', 'text/event-stream');
    res.setHeader('Cache-Control', 'no-cache');
    res.setHeader('Connection', 'keep-alive');
    
    const clientId = Date.now();
    const newClient = { id: clientId, res };
    clients.push(newClient);
    
    // Enviar historial reciente
    db.all("SELECT fake_ip, real_domain, timestamp FROM fakeip_table ORDER BY id DESC LIMIT 20", (err, rows) => {
        rows.forEach(row => {
            res.write(`data: ${JSON.stringify(row)}\n\n`);
        });
    });
    
    req.on('close', () => {
        const index = clients.findIndex(c => c.id === clientId);
        if (index !== -1) clients.splice(index, 1);
    });
});

// Endpoint para capturar interceptaciones (lo llamará el proxy hijack)
app.post('/capture', (req, res) => {
    const { fakeip, domain } = req.body;
    
    db.run("INSERT OR REPLACE INTO fakeip_table (fake_ip, real_domain) VALUES (?, ?)", 
        [fakeip, domain], 
        (err) => {
            if (!err) {
                // Notificar a todos los clientes SSE
                const eventData = JSON.stringify({ fakeip, domain, timestamp: new Date() });
                clients.forEach(client => client.res.write(`data: ${eventData}\n\n`));
                res.json({ status: 'captured' });
            } else {
                res.status(500).json({ error: err.message });
            }
        });
});

// Endpoint de salud
app.get('/health', (req, res) => res.json({ status: 'monitor online' }));

app.listen(port, () => {
    console.log(`🔍 NEXUS MONITOR escuchando en http://localhost:${port}`);
    console.log(`   → SSE: http://localhost:${port}/monitor/events`);
    console.log(`   → POST: http://localhost:${port}/capture`);
});
JS

# 3. package.json para el monitor
cat << 'JSON' > $MONITOR_DIR/package.json
{
  "name": "nexus-monitor",
  "version": "1.0.0",
  "main": "sse-server.js",
  "scripts": {
    "start": "node sse-server.js"
  },
  "dependencies": {
    "express": "^4.18.2",
    "sqlite3": "^5.1.6",
    "cors": "^2.8.5"
  }
}
JSON

# 4. Instalar dependencias
cd $MONITOR_DIR && npm install

# 5. Actualizar el Santuario para apuntar al monitor REAL
cat << 'HTML' > ~/ZENITH_POOL/nexus_santuario/src/index.html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>🏛️ NEXUS SANTUARIO - MONITOR</title>
    <style>
        body { font-family: monospace; background: #05080f; color: #00ffcc; margin: 0; padding: 20px; }
        .grid { display: grid; grid-template-columns: 2fr 1fr; gap: 20px; }
        .card { background: #0d1117; border: 1px solid #00ffcc33; padding: 15px; border-radius: 8px; }
        .interceptacion { color: #ffcc00; border-bottom: 1px solid #333; padding: 8px; font-size: 0.9em; }
        .fakeip { color: #ff3366; font-weight: bold; }
        #live-monitor { height: 500px; overflow-y: auto; background: #000; padding: 10px; border-radius: 4px; }
        .status { font-size: 0.8em; padding: 4px 12px; border-radius: 20px; display: inline-block; }
        .status.online { background: #00ffcc22; color: #00ffcc; }
        .status.offline { background: #ff444422; color: #ff4444; }
    </style>
</head>
<body>
    <h1>🏛️ NEXUS SANTUARIO <span id="status" class="status offline">⬤ MONITOR OFFLINE</span></h1>
    <div class="grid">
        <div class="card">
            <h2>🔄 Tráfico Interceptado (Real-Time)</h2>
            <div id="live-monitor"> Esperando interceptaciones...</div>
        </div>
        <div class="card">
            <h2>🗺️ Guía de Arquitectura</h2>
            <pre style="font-size:0.7em;">
flujo_real:
  [Antigravity/Gemini/Cursor]
        │ (syscall connect)
        ↓
  [PROXY HIJACK :4444]
        │ (intercepta)
        ├─→ ZENITH POOL (13 llaves)
        └─→ /capture → MONITOR :4445
                          │ (SSE)
                          ↓
                    SANTUARIO UI
            </pre>
            <p style="font-size:0.8em;">💡 El monitor recibe eventos CADA VEZ que el proxy hijack desvía una conexión.</p>
        </div>
    </div>

    <script>
        let eventSource = null;
        let reconnectAttempts = 0;
        
        function conectarMonitor() {
            if (eventSource) eventSource.close();
            
            eventSource = new EventSource('http://localhost:4445/monitor/events');
            
            eventSource.onopen = () => {
                document.getElementById('status').innerHTML = '⬤ MONITOR ONLINE';
                document.getElementById('status').className = 'status online';
                reconnectAttempts = 0;
            };
            
            eventSource.onmessage = (event) => {
                const data = JSON.parse(event.data);
                const container = document.getElementById('live-monitor');
                const div = document.createElement('div');
                div.className = 'interceptacion';
                div.innerHTML = `[${new Date().toLocaleTimeString()}] <span class="fakeip">${data.fake_ip || data.fakeip}</span> → ${data.real_domain || data.domain}`;
                container.prepend(div);
                
                // Limitar historial visible
                while (container.children.length > 100) {
                    container.removeChild(container.lastChild);
                }
            };
            
            eventSource.onerror = () => {
                document.getElementById('status').innerHTML = '⬤ MONITOR OFFLINE';
                document.getElementById('status').className = 'status offline';
                eventSource.close();
                
                reconnectAttempts++;
                const delay = Math.min(5000, reconnectAttempts * 1000);
                setTimeout(conectarMonitor, delay);
            };
        }
        
        conectarMonitor();
    </script>
</body>
</html>
HTML

echo ""
echo "✅ MONITOR V4.0 IMPLEMENTADO"
echo ""
echo "📋 Para que funcione:"
echo ""
echo "1. Inicia el monitor SSE:"
echo "   cd ~/ZENITH_POOL/nexus_monitor && npm start"
echo ""
echo "2. Modifica tu binario Rust para que llame a POST /capture"
echo "   cuando intercepte una conexión:"
echo "   curl -X POST http://localhost:4445/capture \\"
echo "        -H 'Content-Type: application/json' \\"
echo "        -d '{\"fakeip\":\"198.18.1.5\",\"domain\":\"cloudcode-pa.googleapis.com\"}'"
echo ""
echo "3. Reinicia el Santuario:"
echo "   cd ~/ZENITH_POOL/nexus_santuario && npm start"
echo ""
