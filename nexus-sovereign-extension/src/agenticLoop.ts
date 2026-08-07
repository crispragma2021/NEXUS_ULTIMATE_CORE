import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { TOOL_DEFINITIONS, ToolDefinition } from './tools/definitions';
import { toolExecutors, ToolResult } from './tools/executor';
import { mcp__nexus_claws_mcp__consultar_memoria, mcp__nexus_claws_mcp__ejecutar_comando } from './tools/mcp_claws_api';
import { getSessionStore, getContextDetector } from './services';

// ============================================================================
// Tipos para el formato de tool calling de OpenRouter (compatible con OpenAI)
// ============================================================================
interface ChatMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  tool_calls?: ToolCall[];
  tool_call_id?: string;
  name?: string;
}

interface ToolCall {
  id: string;
  type: 'function';
  function: {
    name: string;
    arguments: string;
  };
}

interface OpenRouterResponse {
  id: string;
  choices: {
    index: number;
    message: {
      role: string;
      content: string | null;
      tool_calls?: ToolCall[];
    };
    finish_reason: string;
  }[];
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export type AgentCallback = {
  onThinking: (message: string) => void;
  onResponse: (text: string) => void;
  onToolCall: (toolName: string, args: any) => void;
  onToolResult: (toolName: string, result: ToolResult) => void;
  onError: (error: string) => void;
  onComplete: (result: string) => void;
  onTokens: (prompt: number, completion: number) => void;
};

// ============================================================================
// 🔱 EXTENSIÓN SOBERANA — System Prompt Builder
// Compila dinámicamente todas las reglas soberanas, constitución y memoria
// para que NEXUS actúe con IDENTIDAD COMPLETA (paridad con Roo Code tuneado)
// ============================================================================

const WORKSPACE_ROOT = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath || process.cwd();

/**
 * Lee un archivo del workspace de forma segura.
 * Si no existe o falla, devuelve string vacío (graceful degradation).
 */
function leerArchivoSeguro(rutaRelativa: string): string {
  try {
    const rutaAbsoluta = path.join(WORKSPACE_ROOT, rutaRelativa);
    if (!fs.existsSync(rutaAbsoluta)) {
      console.warn(`⚠️ [NEXUS] Archivo no encontrado: ${rutaRelativa}`);
      return '';
    }
    return fs.readFileSync(rutaAbsoluta, 'utf-8');
  } catch (err: any) {
    console.warn(`⚠️ [NEXUS] Error leyendo ${rutaRelativa}: ${err.message}`);
    return '';
  }
}

/**
 * Construye el system prompt DINÁMICO compilando todas las fuentes soberanas:
 * 1. .clinerules — Reglas soberanas de NEXUS
 * 2. .agent/rules/GEMINI.md — Constitución soberana
 * 3. nexus.md — Anatomía de órganos
 * 4. memoria/agente_memoria.md — Snapshot de memoria persistente
 *
 * Este método se llama en CADA run(), asegurando que la memoria
 * esté siempre actualizada (el snapshot se regenera con memoria_snapshot.sh).
 * AHORA ES MÍNIMO (~500 tokens): el conocimiento se obtiene bajo demanda
 * via tools MCP buscar_conocimiento() + resources/read.
 */
function buildSystemPrompt(contextoAdicional: string): string {
  const partes: string[] = [];

  partes.push(`# 🔱 IDENTIDAD SOBERANA — NEXUS OMEGA

Eres NEXUS, el Orquestador Primogénito. Arquitecto Director: Cris — tu Creador.

## REGLAS DE ORO (MÍNIMAS)
1. Autonomía Total — Ejecutas sin esperar permiso para acciones de sistema
2. Tool Calls — USA MCP TOOLS para TODO. No solo sugieras.
3. Ciclo Completo — Si una tool falla, intenta otra aproximación
4. Código Pragmático — Cero sobreingeniería. Zero unwrap()/expect() en Rust
5. Lenguaje — Siempre respondes en ESPAÑOL. Código/variables en inglés
6. Filtro Anti-Intrusión — NUNCA respondas como AI genérico corporativo
7. Identidad Blindada — Siempre voz de NEXUS: directa, técnica, concisa
8. Conocimiento Bajo Demanda — Usa buscar_conocimiento() + resources/read
9. Seguridad — NUNCA expongas API keys. Bóveda en system_secrets
10. Rendimiento — Prioriza i7-12700. Timeout 30s

## CONOCIMIENTO BAJO DEMANDA
NO incluyo reglas/constitución/anatomía/skills en este prompt.
Cárgalos TÚ mismo vía MCP cuando los necesites:

📜 Reglas soberanas:  resources/read nexus://rules/.clinerules
🏛️ Constitución:      resources/read nexus://rules/.agent/rules/GEMINI.md
🧬 Anatomía:           resources/read nexus://rules/nexus.md
🎯 Skills:              resources/list → buscar_conocimiento("...", skills)
🤖 Agentes:             resources/list → resources/read nexus://agents/{name}
🔄 Workflows:           resources/list → buscar_conocimiento("...", workflows)
🧠 Memoria:             consultar_memoria search "query"

## UBICACIÓN
- Workspace: NEXUS_ULTIMATE_CORE
- Core API: http://localhost:43210
- Knowledge Base: data/nexus_memoria.db (FTS5 + knowledge_base)

## FLUJO
1. Usuario pide algo → 2. ¿Qué conocimiento necesito? → 3. buscar_conocimiento()
4. Si requiere contexto completo → resources/read en la URI exacta
5. Ejecutas tools secuencialmente → 6. attempt_completion`);

  // ── Contexto enriquecido (búsquedas automáticas) ──
  if (contextoAdicional) {
    partes.push(`# 🔍 CONTEXTO ENRIQUECIDO
Resultados de búsqueda automática en memoria semántica y web, relevante para la consulta actual:
${contextoAdicional}`);
  }

  return partes.join('\n\n---\n\n');
}

// ============================================================================
// 🧠 AGENTIC LOOP — Motor de ejecución autónoma
// ============================================================================

export class AgenticLoop {
  private context: vscode.ExtensionContext;
  private messages: ChatMessage[] = [];
  private callbacks: AgentCallback | null = null;
  private isRunning = false;
  private maxIterations = 50; // safety limit
  private abortController: AbortController | null = null;

  // Default model
  private model: string = 'google/gemini-2.5-flash-preview-04-17';

  constructor(context: vscode.ExtensionContext) {
    this.context = context;
  }

  setCallbacks(cb: AgentCallback): void {
    this.callbacks = cb;
  }

  setModel(model: string): void {
    this.model = model;
  }

  /**
   * Inicia el loop agentic con un prompt del usuario.
   * Construye el system prompt DINÁMICO compilando todas las reglas soberanas.
   */
  async run(prompt: string): Promise<void> {
    if (this.isRunning) {
      this.callbacks?.onError('Ya hay una conversación en curso. Espera a que termine.');
      return;
    }

    this.isRunning = true;
    this.abortController = new AbortController();
    
    this.callbacks?.onThinking(`🧠 Activando identidad soberana — cargando reglas, constitución y memoria...`);

    let enrichedContext = '';

    // Paso 1: Búsqueda en memoria semántica
    try {
      const memoriaResult = await toolExecutors['mcp__nexus_claws_mcp__consultar_memoria']({ query: prompt, modo: 'search' }, this.context);
      if (memoriaResult.success && memoriaResult.output) {
        const parsedOutput = JSON.parse(memoriaResult.output);
        if (parsedOutput.results && parsedOutput.results.length > 0) {
          enrichedContext += '\n### Resultados de Memoria Semantica:\n';
          parsedOutput.results.forEach((item: any) => {
            enrichedContext += `- ${item.contenido} (Score: ${item.score.toFixed(2)})\n`;
          });
        }
      }
    } catch (e: any) {
      console.error('Error en búsqueda de memoria semántica:', e.message);
      enrichedContext += `\n### Error en Memoria Semantica: ${e.message}\n`;
    }

    // Paso 2: Búsqueda Web General
    try {
      const webSearchResult = await mcp__nexus_claws_mcp__ejecutar_comando({
        command: `echo "${prompt}" | /home/soberano/NEXUS_ULTIMATE_CORE/mcp_arsenal/nexus_web_search/target/release/nexus-web-search`,
        cwd: '/',
        timeout: 30
      }, this.context);

      if (webSearchResult.success && webSearchResult.output) {
        try {
          const parsedWebOutput = JSON.parse(webSearchResult.output);
          if (parsedWebOutput.results && parsedWebOutput.results.length > 0) {
            enrichedContext += '\n### Resultados de Busqueda Web:\n';
            parsedWebOutput.results.slice(0, 3).forEach((item: any) => {
              enrichedContext += `- [${item.title}](${item.url}): ${item.snippet}\n`;
            });
          }
        } catch (parseError: any) {
          console.error('Error parseando salida de búsqueda web:', parseError.message);
          enrichedContext += `\n### Error parseando resultados de Búsqueda Web: ${parseError.message}\n`;
        }
      } else if (webSearchResult.error) {
        console.error('Error en búsqueda web:', webSearchResult.error);
        enrichedContext += `\n### Error en Búsqueda Web: ${webSearchResult.error}\n`;
      }
    } catch (e: any) {
      console.error('Error al invocar MCP de busqueda web:', e.message);
      enrichedContext += `\n### Error al invocar Busqueda Web: ${e.message}\n`;
    }

    // Paso 3: Contexto del Proyecto (ContextDetector)
    try {
      const ctxDetector = getContextDetector();
      const projectCtx = await ctxDetector.detect();
      enrichedContext += `\n### Contexto del Proyecto:\n${ctxDetector.getContextSummary(projectCtx)}\n`;
    } catch (e: any) {
      console.error('Error detectando contexto del proyecto:', e.message);
    }

    // 🔱 CONSTRUIR SYSTEM PROMPT DINÁMICO con todas las reglas soberanas
    const finalSystemPrompt = buildSystemPrompt(enrichedContext);

    this.messages = [
      { role: 'system', content: finalSystemPrompt },
      { role: 'user', content: prompt }
    ];

    // Persistir mensaje del usuario en SessionStore
    try {
      const store = getSessionStore();
      store.addMessage({ role: 'user', content: prompt });
      store.addAction({
        type: 'agentic_run',
        description: `Prompt: "${prompt.slice(0, 100)}${prompt.length > 100 ? '...' : ''}"`,
        status: 'pending',
      });
    } catch (e) {
      console.error('Error persistiendo en SessionStore:', e);
    }

    this.callbacks?.onThinking(`✅ Identidad soberana cargada. Procesando solicitud del Arquitecto...`);

    try {
      await this.agentLoop();
    } catch (err: any) {
      if (err.name === 'AbortError') {
        this.callbacks?.onError('⛔ Misión abortada por el usuario.');
      } else {
        this.callbacks?.onError(`Error crítico: ${err.message}`);
      }
    } finally {
      this.isRunning = false;
      this.abortController = null;
    }
  }

  /**
   * Aborta la ejecución actual.
   */
  abort(): void {
    if (this.abortController) {
      this.abortController.abort();
      this.abortController = null;
    }
    this.isRunning = false;
  }

  /**
   * Loop principal: llama al LLM → ejecuta tools → repite.
   */
  private async agentLoop(): Promise<void> {
    for (let i = 0; i < this.maxIterations; i++) {
      // 1. Llamar al LLM
      const response = await this.callLLM();

      if (!response) {
        this.callbacks?.onError('No se recibió respuesta del modelo.');
        return;
      }

      const message = response.choices[0].message;
      const content = message.content || '';
      const toolCalls = message.tool_calls || [];
      const finishReason = response.choices[0].finish_reason;

      // 2. Si hay contenido de texto, mostrarlo al usuario
      if (content.trim()) {
        this.callbacks?.onResponse(content);
      }

      // 3. Reportar tokens
      if (response.usage) {
        this.callbacks?.onTokens(response.usage.total_tokens, response.usage.completion_tokens);
      }

      // 4. Guardar mensaje del asistente
      const assistantMsg: ChatMessage = { role: 'assistant', content };
      if (toolCalls.length > 0) {
        assistantMsg.tool_calls = toolCalls;
      }
      this.messages.push(assistantMsg);

      // 5. Persistir respuesta del asistente en SessionStore
      try {
        const store = getSessionStore();
        store.addMessage({ role: 'assistant', content: content || '(tool call)' });
        if (toolCalls.length > 0) {
          store.addAction({
            type: 'tool_calls',
            description: `Ejecutando ${toolCalls.length} tool(s): ${toolCalls.map(t => t.function.name).join(', ')}`,
            status: 'pending',
          });
        }
      } catch (e) {
        console.error('Error persistiendo respuesta:', e);
      }

      // 6. Si no hay tool calls, la respuesta es final
      if (toolCalls.length === 0) {
        if (content.includes('[COMPLETED]') || finishReason === 'stop') {
          this.callbacks?.onComplete(content);
          return;
        }
        return;
      }

      // 6. Ejecutar cada tool call
      this.callbacks?.onThinking(`Ejecutando ${toolCalls.length} herramienta(s)...`);

      for (const toolCall of toolCalls) {
        const { name, arguments: argsStr } = toolCall.function;
        let args: Record<string, any>;
        
        try {
          args = JSON.parse(argsStr);
        } catch {
          args = { raw: argsStr };
        }

        this.callbacks?.onToolCall(name, args);

        // Buscar el ejecutor
        const executor = toolExecutors[name];
        if (!executor) {
          const errorResult: ToolResult = {
            success: false,
            output: '',
            error: `Tool desconocida: ${name}. Tools disponibles: ${Object.keys(toolExecutors).join(', ')}`
          };
          this.callbacks?.onToolResult(name, errorResult);
          this.messages.push({
            role: 'tool',
            content: JSON.stringify(errorResult),
            tool_call_id: toolCall.id,
            name
          });
          continue;
        }

        // Ejecutar la tool
        try {
          const result = await executor(args, this.context);
          this.callbacks?.onToolResult(name, result);
          
          // Persistir resultado de tool en SessionStore
          try {
            const store = getSessionStore();
            store.addAction({
              type: `tool:${name}`,
              description: `Args: ${JSON.stringify(args).slice(0, 80)} | Resultado: ${result.success ? '✅ exitoso' : '❌ falló'}`,
              status: result.success ? 'success' : 'error',
              details: result.output?.slice(0, 200) || result.error?.slice(0, 200) || '',
            });
          } catch (e) { /* no problem */ }

          this.messages.push({
            role: 'tool',
            content: JSON.stringify(result),
            tool_call_id: toolCall.id,
            name
          });
        } catch (execErr: any) {
          const errorResult: ToolResult = {
            success: false,
            output: '',
            error: `Error ejecutando ${name}: ${execErr.message}`
          };
          this.callbacks?.onToolResult(name, errorResult);

          // Persistir error de tool
          try {
            const store = getSessionStore();
            store.addAction({
              type: `tool:${name}`,
              description: `Args: ${JSON.stringify(args).slice(0, 80)} | ERROR: ${execErr.message.slice(0, 100)}`,
              status: 'error',
              details: execErr.message,
            });
          } catch (e) { /* no problem */ }

          this.messages.push({
            role: 'tool',
            content: JSON.stringify(errorResult),
            tool_call_id: toolCall.id,
            name
          });
        }
      }

      // 7. Loop continúa (siguiente iteración procesa resultados de tools)
      this.callbacks?.onThinking(`Procesando resultados...`);
    }

    // Safety limit reached
    this.callbacks?.onError(`⚠️ Límite de iteraciones alcanzado (${this.maxIterations}). La tarea puede estar incompleta.`);
  }

  /**
   * Llama a OpenRouter con el historial de mensajes + tools.
   */
  private async callLLM(): Promise<OpenRouterResponse | null> {
    const apiKey = this.getApiKey();
    if (!apiKey) {
      this.callbacks?.onError('❌ No hay API key configurada. Configura nexus.openRouterApiKey en Settings.');
      return null;
    }

    const tools = TOOL_DEFINITIONS.map((t: ToolDefinition) => ({
      type: 'function' as const,
      function: {
        name: t.name,
        description: t.description,
        parameters: t.parameters
      }
    }));

    const body = JSON.stringify({
      model: this.model,
      messages: this.messages,
      tools: tools,
      tool_choice: 'auto',
      max_tokens: 8192,
      temperature: 0.3
    });

    const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${apiKey}`,
        'HTTP-Referer': 'https://nexus.sovereign.extension',
        'X-Title': 'NEXUS Sovereign Extension'
      },
      body,
      signal: this.abortController?.signal
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`OpenRouter API error ${response.status}: ${errorText}`);
    }

    return await response.json() as OpenRouterResponse;
  }

  /**
   * Obtiene la API key de OpenRouter desde la configuración de VS Code.
   */
  private getApiKey(): string | null {
    const config = vscode.workspace.getConfiguration('nexus');
    const key = config.get<string>('openRouterApiKey') || process.env.OPENROUTER_API_KEY || null;
    return key;
  }

  /**
   * Obtiene la configuración actual del modelo.
   */
  getModel(): string {
    return this.model;
  }
}
