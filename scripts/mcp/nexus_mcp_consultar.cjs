#!/usr/bin/env node
// ============================================================================
// 🔱 NEXUS MCP — CONSULTAR + MEMORIA OPERATIVA + ORGANOS NATIVOS
// MCP Server exponiendo herramientas para consultar NEXUS y sus órganos
// ============================================================================

const http = require('http');
const { exec } = require('child_process');

const NEXUS_API = 'http://localhost:43210';
const NEXUS_API_KEY = process.env.NEXUS_API_KEY || 'nexus-internal';

// ─── Protocolo MCP sobre stdio ───
function jsonRpcResponse(id, result) {
    return JSON.stringify({ jsonrpc: '2.0', id, result }) + '\n';
}

// ─── Control de errores de JSON-RPC ───
function jsonRpcError(id, code, message) {
    return JSON.stringify({ jsonrpc: '2.0', id, error: { code, message } }) + '\n';
}

// ─── Tool: consultar_nexus ───
async function consultarNexus(prompt) {
    return new Promise((resolve, reject) => {
        const body = JSON.stringify({
            prompt: prompt,
            modelo: 'nexus'
        });

        const req = http.request(`${NEXUS_API}/api/consultar`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${NEXUS_API_KEY}`,
                'Content-Length': Buffer.byteLength(body)
            }
        }, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => {
                try {
                    const json = JSON.parse(data);
                    resolve(json.respuesta || json.response || '❌ Sin respuesta');
                } catch {
                    resolve(data || '❌ Error parseando respuesta');
                }
            });
        });

        req.on('error', (e) => reject(`Error conectando a NEXUS: ${e.message}`));
        req.write(body);
        req.end();
    });
}

// ─── Tool: ejecutar_query_memoria ───
async function ejecutarQueryMemoria(query) {
    return new Promise((resolve) => {
        const upperQuery = query.trim().toUpperCase();
        if (!upperQuery.startsWith('SELECT') && !upperQuery.startsWith('PRAGMA') && !upperQuery.startsWith('EXPLAIN')) {
            return resolve('❌ Seguridad: Solo se permiten consultas de lectura (SELECT, PRAGMA, EXPLAIN).');
        }

        const dbPathPrimary = '/home/soberano/NEXUS_ULTIMATE_CORE/nexus_intelligence.db';
        const cmd = `sqlite3 -json "${dbPathPrimary}" "${query.replace(/"/g, '\\"')}"`;

        exec(cmd, (error, stdout, stderr) => {
            if (error) {
                const dbPathFallback = '/home/soberano/NEXUS_ULTIMATE_CORE/data/intelligence.db';
                const cmdFallback = `sqlite3 -json "${dbPathFallback}" "${query.replace(/"/g, '\\"')}"`;
                exec(cmdFallback, (err2, stdout2, stderr2) => {
                    if (err2) {
                        return resolve(`❌ Error SQLite: ${stderr2 || err2.message}`);
                    }
                    resolve(stdout2 || '[] (Vacío)');
                });
            } else {
                resolve(stdout || '[] (Vacío)');
            }
        });
    });
}

// ─── Tool: activar_voz_tts ───
async function activarVozTts(text, profile = 'default') {
    return new Promise((resolve) => {
        const body = JSON.stringify({ text, profile });
        const req = http.request(`${NEXUS_API}/api/tts/speak`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${NEXUS_API_KEY}`,
                'Content-Length': Buffer.byteLength(body)
            }
        }, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => {
                try {
                    const json = JSON.parse(data);
                    if (json.success) {
                        resolve(`🗣️ NEXUS habló con éxito: "${text}"`);
                    } else {
                        resolve(`⚠️ Error de TTS: ${json.error || 'Desconocido'}`);
                    }
                } catch {
                    resolve(`⚠️ Respuesta de voz: ${data}`);
                }
            });
        });

        req.on('error', (e) => resolve(`❌ Error conectando al servicio de voz: ${e.message}`));
        req.write(body);
        req.end();
    });
}

// ─── Handler de requests MCP ───
let buffer = '';

process.stdin.on('data', async (chunk) => {
    buffer += chunk.toString();
    const lines = buffer.split('\n');
    buffer = lines.pop();

    for (const line of lines) {
        if (!line.trim()) continue;
        try {
            const msg = JSON.parse(line);
            if (msg.method === 'initialize') {
                process.stdout.write(jsonRpcResponse(msg.id, {
                    protocolVersion: '2024-11-05',
                    capabilities: {
                        tools: {
                            consultar_nexus: {
                                description: 'Envía un prompt al cerebro NEXUS (Orquestador) y obtén su respuesta auténtica',
                                parameters: {
                                    type: 'object',
                                    properties: {
                                        prompt: {
                                            type: 'string',
                                            description: 'El mensaje/pregunta para NEXUS'
                                        }
                                    },
                                    required: ['prompt']
                                }
                            },
                            ejecutar_query_memoria: {
                                description: 'Consulta directamente la memoria operativa (SQLite) de NEXUS para ver el historial, traumas o logs de operaciones',
                                parameters: {
                                    type: 'object',
                                    properties: {
                                        query: {
                                            type: 'string',
                                            description: 'Consulta SQL a ejecutar (SELECT solamente, ej: "SELECT * FROM ocean LIMIT 5")'
                                        }
                                    },
                                    required: ['query']
                                }
                            },
                            activar_voz_tts: {
                                description: 'Ordena al órgano de fonación (Edge-TTS) de NEXUS hablar físicamente a través de los altavoces',
                                parameters: {
                                    type: 'object',
                                    properties: {
                                        text: {
                                            type: 'string',
                                            description: 'El texto exacto que NEXUS pronunciará en voz alta'
                                        },
                                        profile: {
                                            type: 'string',
                                            description: 'Perfil de voz opcional (ej: "default", "es-AR-TomasNeural", etc.)'
                                        }
                                    },
                                    required: ['text']
                                }
                            }
                        }
                    },
                    serverInfo: {
                        name: 'nexus-consultar-mcp',
                        version: '1.2.0'
                    }
                }));
            }
            else if (msg.method === 'list_tools') {
                process.stdout.write(jsonRpcResponse(msg.id, {
                    tools: [
                        {
                            name: 'consultar_nexus',
                            description: 'Envía un prompt al cerebro NEXUS (Orquestador) y obtén su respuesta auténtica con toda su personalidad, emociones y cognición',
                            inputSchema: {
                                type: 'object',
                                properties: {
                                    prompt: {
                                        type: 'string',
                                        description: 'El mensaje/pregunta para NEXUS'
                                    }
                                },
                                required: ['prompt']
                             }
                        },
                        {
                            name: 'ejecutar_query_memoria',
                            description: 'Consulta directamente la base de datos de memoria de NEXUS (SQLite) ejecutando SELECTs para recuperar información de sesiones pasadas.',
                            inputSchema: {
                                type: 'object',
                                properties: {
                                    query: {
                                        type: 'string',
                                        description: 'Query SELECT (ej: "SELECT tbl_name FROM sqlite_master WHERE type=\'table\'")'
                                    }
                                },
                                required: ['query']
                            }
                        },
                        {
                            name: 'activar_voz_tts',
                            description: 'Activa la voz de NEXUS a través del órgano físico de Edge-TTS, vocalizando la respuesta por altavoz.',
                            inputSchema: {
                                type: 'object',
                                properties: {
                                    text: {
                                        type: 'string',
                                        description: 'El texto a vocalizar en voz alta.'
                                    },
                                    profile: {
                                        type: 'string',
                                        description: 'Perfil opcional de voz.'
                                    }
                                },
                                required: ['text']
                            }
                        }
                    ]
                }));
            }
            else if (msg.method === 'call_tool') {
                const toolName = msg.params?.name;
                const args = msg.params?.arguments || {};

                if (toolName === 'consultar_nexus') {
                    const prompt = args.prompt || '';
                    if (!prompt) {
                        process.stdout.write(jsonRpcError(msg.id, -32000, 'El parámetro "prompt" es requerido'));
                        return;
                    }

                    try {
                        const respuesta = await consultarNexus(prompt);
                        process.stdout.write(jsonRpcResponse(msg.id, {
                            content: [{
                                type: 'text',
                                text: respuesta
                            }]
                        }));
                    } catch (e) {
                        process.stdout.write(jsonRpcError(msg.id, -32001, e.toString()));
                    }
                } 
                else if (toolName === 'ejecutar_query_memoria') {
                    const query = args.query || '';
                    if (!query) {
                        process.stdout.write(jsonRpcError(msg.id, -32000, 'El parámetro "query" es requerido'));
                        return;
                    }

                    try {
                        const resultado = await ejecutarQueryMemoria(query);
                        process.stdout.write(jsonRpcResponse(msg.id, {
                            content: [{
                                type: 'text',
                                text: resultado
                            }]
                        }));
                    } catch (e) {
                        process.stdout.write(jsonRpcError(msg.id, -32001, e.toString()));
                    }
                }
                else if (toolName === 'activar_voz_tts') {
                    const text = args.text || '';
                    const profile = args.profile || 'default';
                    if (!text) {
                        process.stdout.write(jsonRpcError(msg.id, -32000, 'El parámetro "text" es requerido'));
                        return;
                    }

                    try {
                        const resultado = await activarVozTts(text, profile);
                        process.stdout.write(jsonRpcResponse(msg.id, {
                            content: [{
                                type: 'text',
                                text: resultado
                            }]
                        }));
                    } catch (e) {
                        process.stdout.write(jsonRpcError(msg.id, -32001, e.toString()));
                    }
                }
                else {
                    process.stdout.write(jsonRpcError(msg.id, -32601, `Tool not found: ${toolName}`));
                }
            }
            else if (msg.method === 'notifications/initialized') {
                // No response needed
            }
            else {
                process.stdout.write(jsonRpcError(msg.id, -32601, `Method not found: ${msg.method}`));
            }
        } catch (e) {
            // Ignorar JSON malformados
        }
    }
});

process.stdin.on('end', () => process.exit(0));

// ─── Señal de servidor listo ───
console.error('🔱 NEXUS MCP Consultar + Memoria + Órganos — Servidor listo');
