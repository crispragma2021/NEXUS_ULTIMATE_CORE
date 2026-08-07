#!/usr/bin/env node
/**
 * NEXUS Parallel Agents MCP v2.0 — Soberano Job Scheduler
 * Evolucionado para permitir ejecución persistente en segundo plano sin bloqueo.
 */

const readline = require('readline');
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const TASK_DIR = '/tmp/nexus_parallel_tasks';
if (!fs.existsSync(TASK_DIR)) fs.mkdirSync(TASK_DIR, { recursive: true });

const tasks = new Map(); // id → task metadata

function send(obj) { process.stdout.write(JSON.stringify(obj) + '\n'); }
function error(id, msg) { send({ jsonrpc: '2.0', id, error: { code: -32000, message: String(msg) } }); }
function ok(id, result) { send({ jsonrpc: '2.0', id, result }); }

/**
 * Lanza un proceso en segundo plano con persistencia de logs.
 */
function spawnTask(cmd, cwd, background = false) {
  const id = crypto.randomBytes(4).toString('hex');
  const logFile = path.join(TASK_DIR, `${id}.log`);
  const metaFile = path.join(TASK_DIR, `${id}.json`);

  const proc = spawn('bash', ['-c', cmd], {
    cwd: cwd || '/home/soberano/NEXUS_ULTIMATE_CORE',
    stdio: ['ignore', 'pipe', 'pipe'],
    detached: background
  });

  if (background) proc.unref();

  const task = { 
    id, 
    cmd, 
    status: 'running', 
    startedAt: Date.now(), 
    logFile,
    metaFile,
    exitCode: null,
    output: '' 
  };
  
  tasks.set(id, task);

  const logStream = fs.createWriteStream(logFile);
  proc.stdout.pipe(logStream);
  proc.stderr.pipe(logStream);

  // Buffer local para respuestas rápidas
  proc.stdout.on('data', (d) => task.output += d.toString());
  proc.stderr.on('data', (d) => task.output += d.toString());

  proc.on('close', (code) => {
    task.status = code === 0 ? 'done' : 'failed';
    task.exitCode = code;
    task.endedAt = Date.now();
    fs.writeFileSync(metaFile, JSON.stringify(task, null, 2));
  });

  proc.on('error', (err) => {
    task.status = 'error';
    task.error = err.message;
    fs.writeFileSync(metaFile, JSON.stringify(task, null, 2));
  });

  task.proc = proc;
  return id;
}

async function waitAll(ids, timeoutMs = 30000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const allDone = ids.every(id => tasks.get(id)?.status !== 'running');
    if (allDone) break;
    await new Promise(r => setTimeout(r, 500));
  }
}

const TOOLS = {
  parallel_run: {
    description: 'Ejecuta comandos en paralelo. Si background=true, devuelve el ID de tarea inmediatamente sin esperar.',
    inputSchema: {
      type: 'object',
      properties: {
        commands: { type: 'array', items: { type: 'string' } },
        cwd: { type: 'string' },
        background: { type: 'boolean', description: 'No esperar a que terminen' },
        timeout_ms: { type: 'number' }
      },
      required: ['commands']
    }
  },
  task_status: {
    description: 'Consulta estado y últimos logs de una tarea.',
    inputSchema: {
      type: 'object',
      properties: { 
        task_id: { type: 'string' },
        tail: { type: 'number', description: 'Líneas de log a recuperar' }
      },
      required: ['task_id']
    }
  },
  task_list: {
    description: 'Lista todas las tareas activas y terminadas.',
    inputSchema: { type: 'object', properties: {} }
  },
  task_kill: {
    description: 'Termina un proceso en curso.',
    inputSchema: {
      type: 'object',
      properties: { task_id: { type: 'string' } },
      required: ['task_id']
    }
  }
};

const rl = readline.createInterface({ input: process.stdin });
rl.on('line', async (line) => {
  let msg;
  try { msg = JSON.parse(line); } catch { return; }
  const { id, method, params } = msg;

  try {
    if (method === 'initialize') {
      ok(id, { protocolVersion: '2024-11-05', capabilities: { tools: {} }, serverInfo: { name: 'nexus-parallel-mcp', version: '2.0.0' } });
    } else if (method === 'tools/list') {
      ok(id, { tools: Object.entries(TOOLS).map(([name, def]) => ({ name, ...def })) });
    } else if (method === 'tools/call') {
      const args = params.arguments;
      switch (params.name) {
        case 'parallel_run':
          const ids = args.commands.map(cmd => spawnTask(cmd, args.cwd, !!args.background));
          if (args.background) {
            return ok(id, { content: [{ type: 'text', text: `🚀 Tareas lanzadas en background. IDs: ${ids.join(', ')}\nUsa task_status para monitorear.` }] });
          }
          await waitAll(ids, args.timeout_ms || 30000);
          const results = ids.map(id => {
            const t = tasks.get(id);
            return `[${id}] ${t.status.toUpperCase()}: ${t.output.slice(-1000)}`;
          }).join('\n---\n');
          ok(id, { content: [{ type: 'text', text: results }] });
          break;

        case 'task_status':
          const t = tasks.get(args.task_id);
          if (!t) return error(id, 'Tarea no encontrada');
          let logTail = '';
          if (fs.existsSync(t.logFile)) {
            const lines = fs.readFileSync(t.logFile, 'utf-8').split('\n');
            logTail = lines.slice(-(args.tail || 50)).join('\n');
          }
          ok(id, { content: [{ type: 'text', text: `ID: ${t.id}\nStatus: ${t.status}\nElapsed: ${((Date.now()-t.startedAt)/1000).toFixed(1)}s\nLog:\n${logTail}` }] });
          break;

        case 'task_list':
          const list = [...tasks.values()].map(t => `[${t.status.padEnd(8)}] ${t.id} | ${t.cmd.slice(0, 50)}...`).join('\n') || 'No hay tareas.';
          ok(id, { content: [{ type: 'text', text: list }] });
          break;

        case 'task_kill':
          const tk = tasks.get(args.task_id);
          if (tk && tk.proc) {
            tk.proc.kill('SIGKILL');
            tk.status = 'killed';
            ok(id, { content: [{ type: 'text', text: `Tarea ${args.task_id} eliminada.` }] });
          } else {
            error(id, 'Tarea no encontrada o ya finalizada.');
          }
          break;
      }
    }
  } catch (e) { error(id, e.message); }
});
