#!/usr/bin/env node
/**
 * OMEGA-SEARCH MCP Server v2.2: Hardened Edition
 * Blindado contra crashes asíncronos y cierres inesperados.
 */

const readline = require('readline');
const { performScientificSearch } = require('./omega_search.cjs');

// Redirigir errores no capturados a stderr para no romper el protocolo JSON-RPC de stdout
process.on('uncaughtException', (err) => {
  console.error(`[CRITICAL] Uncaught Exception: ${err.message}`);
});

process.on('unhandledRejection', (reason, promise) => {
  console.error(`[CRITICAL] Unhandled Rejection: ${reason}`);
});

function send(obj) { 
  try {
    process.stdout.write(JSON.stringify(obj) + '\n'); 
  } catch (e) {
    console.error(`[ERROR] Fallo al enviar respuesta JSON: ${e.message}`);
  }
}

function error(id, msg) { send({ jsonrpc: '2.0', id, error: { code: -32000, message: String(msg) } }); }
function ok(id, result) { send({ jsonrpc: '2.0', id, result }); }

const TOOLS = {
  omega_deep_search: {
    description: 'Realiza una búsqueda científica profunda en multihilo sobre foros de desarrollo (GitHub, StackOverflow, Rust forums). Extrae código listo para compilar y discusiones técnicas relevantes.',
    inputSchema: {
      type: 'object',
      properties: {
        query: { type: 'string', description: 'Término o error exacto a investigar' },
        limit: { type: 'number', description: 'Número máximo de páginas profundas a analizar (default: 5)' }
      },
      required: ['query']
    }
  }
};

const rl = readline.createInterface({ input: process.stdin });

rl.on('line', async (line) => {
  if (!line.trim()) return;
  let msg;
  try { 
    msg = JSON.parse(line); 
  } catch (e) { 
    console.error(`[ERROR] JSON Inválido recibido: ${line}`);
    return; 
  }

  const { id, method, params } = msg;

  try {
    if (method === 'initialize') {
      ok(id, {
        protocolVersion: '2024-11-05',
        capabilities: { tools: {} },
        serverInfo: { name: 'nexus-omega-search-mcp', version: '1.2.0' }
      });
    } else if (method === 'tools/list') {
      ok(id, {
        tools: Object.entries(TOOLS).map(([name, def]) => ({
          name, description: def.description, inputSchema: def.inputSchema
        }))
      });
    } else if (method === 'tools/call') {
      if (params.name === 'omega_deep_search') {
        const query = params.arguments.query;
        const limit = params.arguments.limit || 5;
        
        console.error(`[INFO] Iniciando búsqueda profunda: "${query}" (limit: ${limit})`);
        
        // Ejecutar búsqueda con protección total contra crashes
        let searchResult;
        try {
          searchResult = await performScientificSearch(query, limit);
        } catch (searchErr) {
          console.error(`[ERROR] El motor de búsqueda falló: ${searchErr.message}`);
          return error(id, `Fallo crítico en el motor de búsqueda: ${searchErr.message}`);
        }

        if (!searchResult || !Array.isArray(searchResult)) {
          return ok(id, { content: [{ type: 'text', text: 'No se obtuvieron resultados válidos del motor.' }] });
        }
        
        // Formatear el resultado JSON de forma muy legible para el orquestador
        const formatted = searchResult.map((res, i) => {
          if (!res) return `=== [#${i + 1}] Resultado vacío ===\n`;
          if (res.error) {
            return `=== [#${i + 1}] FUENTE: ${res.source || 'Desconocida'} | URL: ${res.url || 'N/A'} ===\n❌ Error: ${res.error}\n`;
          }
          
          const scoreLine = typeof res.score === 'number' ? ` | RELEVANCIA: ${res.score.toFixed(2)}` : '';
          const meta = (res.data && res.data.meta && res.data.meta.ogTitle)
            ? ` | TEMA: ${res.data.meta.ogTitle}`
            : '';
            
          const codeBlocks = (res.data && res.data.codeBlocks && res.data.codeBlocks.length > 0)
            ? res.data.codeBlocks.map((c, idx) => `[Código #${idx + 1}]\n${c}`).join('\n\n')
            : '(Sin bloques de código en este enlace)';
            
          const discussions = (res.data && res.data.discussions && res.data.discussions.length > 0)
            ? res.data.discussions.map((d, idx) => `- ${d}`).join('\n')
            : '(Sin discusiones en este enlace)';
            
          return `=== [#${i + 1}] FUENTE: ${res.source || 'Web'}${scoreLine}${meta} | TÍTULO: ${res.title || 'Sin Título'} | URL: ${res.url || 'N/A'} ===\n\n[DISCUSIÓN TÉCNICA]\n${discussions}\n\n[BLOQUES DE CÓDIGO EXTRAÍDOS]\n${codeBlocks}\n`;
        }).join('\n=======================================================\n');

        ok(id, { content: [{ type: 'text', text: formatted || 'Búsqueda completada sin contenido legible.' }] });
      } else {
        error(id, `Tool desconocida: ${params.name}`);
      }
    } else if (method === 'notifications/initialized') {
      // no-op
    } else {
      error(id, `Método desconocido: ${method}`);
    }
  } catch (e) {
    console.error(`[ERROR] Fallo en el handler de mensajes: ${e.message}`);
    error(id, `Error interno del servidor MCP: ${e.message}`);
  }
});
