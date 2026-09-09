// ============================================================================
// 🔱 NEXUS TLS TERMINATOR (Port 8443 -> 4444)
// ============================================================================
// Script ligero en Node.js nativo para actuar como puente TLS MitM local.
// Escucha en HTTPS en 127.0.0.1:8443, realiza el handshake con el cliente,
// descifra el tráfico y lo canaliza en HTTP plano al proxy local (puerto 4444).
// ============================================================================

const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');

const SECRETS_DIR = '/home/soberano/NEXUS_ULTIMATE_CORE/secrets';
const KEY_PATH = path.join(SECRETS_DIR, 'nexus-server.key');
const CERT_PATH = path.join(SECRETS_DIR, 'nexus-server.pem');

// Verificar existencia de certificados
if (!fs.existsSync(KEY_PATH) || !fs.existsSync(CERT_PATH)) {
    console.error('❌ [TLS TERMINATOR] Error: Certificados no encontrados en secrets/.');
    console.error('   Por favor, ejecuta primero: ./scripts/generar_certificados.sh');
    process.exit(1);
}

const sslOptions = {
    key: fs.readFileSync(KEY_PATH),
    cert: fs.readFileSync(CERT_PATH),
    secureProtocol: 'TLSv1_2_server_method', // Deshabilitar TLS 1.0/1.1
    ciphers: 'HIGH:!aNULL:!eNULL:!EXPORT:!DES:!RC4:!MD5:!PSK:!SRP' // Ciphers seguros
};

const TARGET_HOST = '127.0.0.1';
const TARGET_PORT = 4444;
const LISTEN_PORT = 8443;
const LISTEN_HOST = '127.0.0.1';

// Crear el servidor HTTPS
const server = https.createServer(sslOptions, (clientReq, clientRes) => {
    const start = Date.now();
    
    // Imprimir ruta interceptada
    console.log(`🔱 [TLS TERMINATOR] Interceptado: [${clientReq.method}] ${clientReq.url}`);

    // Replicar las cabeceras del cliente, asegurando el host correcto
    const headers = { ...clientReq.headers };
    headers.host = `${TARGET_HOST}:${TARGET_PORT}`;

    // Configurar la petición proxy hacia el proxy_hijack (HTTP puerto 4444)
    const proxyReq = http.request({
        host: TARGET_HOST,
        port: TARGET_PORT,
        path: clientReq.url,
        method: clientReq.method,
        headers: headers
    }, (proxyRes) => {
        // Reenviar cabeceras de respuesta y código de estado
        clientRes.writeHead(proxyRes.statusCode, proxyRes.headers);
        
        // Canalizar el cuerpo de la respuesta
        proxyRes.pipe(clientRes, { end: true });
        
        proxyRes.on('end', () => {
            const duration = Date.now() - start;
            console.log(`✅ [TLS TERMINATOR] Resuelto [${clientReq.method}] ${clientReq.url} -> Status ${proxyRes.statusCode} (${duration}ms)`);
        });
    });

    proxyReq.on('error', (err) => {
        console.error(`❌ [TLS TERMINATOR] Error de proxy hacia 4444 en ${clientReq.url}:`, err.message);
        clientRes.writeHead(502, { 'Content-Type': 'application/json' });
        clientRes.end(JSON.stringify({ 
            error: 'NEXUS Gateway Timeout', 
            details: 'El proxy_hijack en el puerto 4444 no responde. ¿Está levantado el servicio?' 
        }));
    });

    // Canalizar el cuerpo de la petición entrante al proxy
    clientReq.pipe(proxyReq, { end: true });
});

server.on('error', (err) => {
    console.error('❌ [TLS TERMINATOR] Error en el servidor HTTPS:', err);
});

server.listen(LISTEN_PORT, LISTEN_HOST, () => {
    console.log(`🔱 [TLS TERMINATOR] Escudo TLS Activo en https://${LISTEN_HOST}:${LISTEN_PORT}`);
    console.log(`🔱 [TLS TERMINATOR] Redirigiendo peticiones descifradas a http://127.0.0.1:${TARGET_PORT}`);
});
