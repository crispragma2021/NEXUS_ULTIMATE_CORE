#!/usr/bin/env node
/**
 * NEXUS System Control MCP — Auto-inmunidad de red y control de procesos OMEGA
 *
 * Expone herramientas para liberar puertos bloqueados, monitorear demonios
 * y auto-diagnosticar fallos en los puertos del núcleo.
 */

const readline = require('readline');
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function send(obj) { process.stdout.write(JSON.stringify(obj) + '\n'); }
function error(id, msg) { send({ jsonrpc: '2.0', id, error: { code: -32000, message: msg } }); }
function ok(id, result) { send({ jsonrpc: '2.0', id, result }); }

const TOOLS = {
  sys_free_port: {
    description: 'Busca qué procesos están bloqueando un puerto de red específico y los elimina con SIGKILL (-9) para liberar el puerto.',
    inputSchema: {
      type: 'object',
      properties: {
        port: { type: 'number', description: 'Puerto a liberar (ej: 42210, 42220, 5173)' }
      },
      required: ['port']
    }
  },
  sys_daemon_control: {
    description: 'Controla y monitorea el estado de los demonios de backend de Rust (nexus_gateway, nexus-shell).',
    inputSchema: {
      type: 'object',
      properties: {
        service: { type: 'string', enum: ['gateway', 'shell'], description: 'Servicio a controlar' },
        action: { type: 'string', enum: ['status', 'start', 'stop', 'restart'], description: 'Acción a ejecutar' }
      },
      required: ['service', 'action']
    }
  },
  sys_check_health: {
    description: 'Comprueba el estado de salud de todos los puertos y endpoints del ecosistema NEXUS.',
    inputSchema: { type: 'object', properties: {} }
  },
  sys_resource_usage: {
    description: 'Reporte biométrico detallado del uso de CPU, RAM y carga del sistema (Soberano).',
    inputSchema: { type: 'object', properties: {} }
  }
};

async function handleTool(name, args) {
  switch (name) {
    case 'sys_free_port': {
      const port = args.port;
      try {
        // Encontrar PIDs usando lsof
        const pids = execSync(`lsof -t -i :${port}`).toString().trim().split('\n').filter(Boolean);
        if (pids.length === 0) {
          return { content: [{ type: 'text', text: `El puerto :${port} ya está libre.` }] };
        }
        
        pids.forEach(pid => {
          execSync(`kill -9 ${pid}`);
        });
        
        return { content: [{ type: 'text', text: `✅ Puerto :${port} liberado. Procesos eliminados: ${pids.join(', ')}` }] };
      } catch (e) {
        // lsof devuelve código de salida 1 si no encuentra coincidencias, lo cual es normal
        return { content: [{ type: 'text', text: `El puerto :${port} está libre o no se detectaron procesos bloqueantes.` }] };
      }
    }

    case 'sys_daemon_control': {
      const { service, action } = args;
      const cmdBase = service === 'gateway' ? 'nexus_gateway' : 'nexus-shell';
      
      if (action === 'status') {
        try {
          const ps = execSync(`pgrep -af ${cmdBase}`).toString().trim();
          return { content: [{ type: 'text', text: `🟢 ACTIVO:\n${ps}` }] };
        } catch {
          return { content: [{ type: 'text', text: `🔴 INACTIVO: El servicio ${service} no se está ejecutando.` }] };
        }
      }
      
      if (action === 'stop') {
        try {
          execSync(`pkill -9 -f ${cmdBase}`);
          return { content: [{ type: 'text', text: `✅ Servicio ${service} detenido.` }] };
        } catch {
          return { content: [{ type: 'text', text: `El servicio ${service} ya estaba detenido.` }] };
        }
      }

      if (action === 'start' || action === 'restart') {
        try {
          execSync(`pkill -9 -f ${cmdBase}`);
        } catch (e) {}

        const scriptPath = service === 'gateway' 
          ? './scripts/ignicion_os_interno.sh'
          : './scripts/nexus_start.sh';
          
        try {
          // Lanzar de forma desacoplada
          execSync(`nohup ${scriptPath} > /tmp/nexus_${service}.log 2>&1 &`);
          return { content: [{ type: 'text', text: `🚀 Servicio ${service} inicializado en background.` }] };
        } catch (e) {
          return { content: [{ type: 'text', text: `❌ Error al iniciar ${service}: ${e.message}` }] };
        }
      }
    }

    case 'sys_check_health': {
      const ports = [
        { port: 1420, name: 'Santuario UI (Chat)' },
        { port: 5173, name: 'HUD Chat' },
        { port: 42220, name: 'Portal de Trading' },
        { port: 42210, name: 'Core API Rust backend' }
      ];

      const report = ports.map(p => {
        try {
          execSync(`nc -z -w 1 localhost ${p.port}`);
          return `  [:${p.port}] 🟢 OK      - ${p.name}`;
        } catch {
          return `  [:${p.port}] 🔴 CAÍDO  - ${p.name}`;
        }
      }).join('\n');

      return { content: [{ type: 'text', text: `📊 REPORT DE SALUD DE RED:\n${report}` }] };
    }

    case 'sys_resource_usage': {
      try {
        const uptime = execSync('uptime -p').toString().trim();
        const load = execSync("cat /proc/loadavg | awk '{print $1, $2, $3}'").toString().trim();
        const mem = execSync("free -m | grep Mem | awk '{print $3\"MB / \"$2\"MB (\"int($3/$2*100)\"%)\"}'").toString().trim();
        const cpu = execSync("grep 'cpu ' /proc/stat | awk '{usage=($2+$4)*100/($2+$4+$5)} END {print int(usage)\"%\"}'").toString().trim();
        const topProcs = execSync("ps -eo pcpu,pmem,comm --sort=-pcpu | head -6 | tail -5").toString().trim();
        
        const report = [
          `⚡ UPTIME: ${uptime}`,
          `🧠 MEMORIA: ${mem}`,
          `🔥 CPU LOAD: ${cpu} (Avg: ${load})`,
          `🔝 PROCESOS TOP (CPU):\n${topProcs}`
        ].join('\n');

        return { content: [{ type: 'text', text: `🧬 BIOMETRÍA DEL SISTEMA:\n${report}` }] };
      } catch (e) {
        return error(id, `Fallo al obtener métricas biométricas: ${e.message}`);
      }
    }

    default:
      throw new Error(`Acción desconocida: ${name}`);
  }
}

// Loop de lectura estándar JSON-RPC
const rl = readline.createInterface({ input: process.stdin });
rl.on('line', async (line) => {
  if (!line.trim()) return;
  let msg;
  try { msg = JSON.parse(line); } catch { return; }
  
  const { id, method, params } = msg;

  try {
    if (method === 'initialize') {
      ok(id, {
        protocolVersion: '2024-11-05',
        capabilities: { tools: {} },
        serverInfo: { name: 'nexus-sys-control', version: '1.0.0' }
      });
    } else if (method === 'tools/list') {
      ok(id, {
        tools: Object.entries(TOOLS).map(([name, def]) => ({
          name, description: def.description, inputSchema: def.inputSchema
        }))
      });
    } else if (method === 'tools/call') {
      const result = await handleTool(params.name, params.arguments || {});
      ok(id, result);
    } else if (method === 'notifications/initialized') {
      // no-op
    } else {
      error(id, `Método desconocido: ${method}`);
    }
  } catch (e) {
    error(id, e.message);
  }
});
