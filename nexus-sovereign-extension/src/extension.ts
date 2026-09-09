// ============================================================================
// 🔱 NEXUS SOVEREIGN EXTENSION - MAIN ENTRY
// ============================================================================
// Extensión autónoma de VS Code. Sin modos, sin dependencia de Roo Code.
// Como Antigravity, pero con Agentic Loop + Tool Calls nativas + OpenRouter.
//
// Capacidades completas (Paridad Antigravity IDE):
//   1. Agentic Loop autónomo       ✅
//   2. Terminal interno (xterm+WS) ✅
//   3. Tool Calling nativo          ✅
//   4. Memoria semántica (SQLite)   ✅
//   5. Web Search inteligente       ✅
//   6. Context Detection automático ✅ (NUEVO)
//   7. Diff Preview visual          ✅ (NUEVO)
//   8. Session Persistence local    ✅ (NUEVO)
//   9. Multi-Agent dinámico         ✅ (NUEVO)
//   10. Action History local        ✅ (NUEVO)
//
// Arquitectura:
//   extension.ts → services.ts → HudPanel + AgenticLoop + ToolExecutor
//                → SessionStore + ContextDetector + DiffPreview
//
// ============================================================================

import * as vscode from 'vscode';
import { AgentControlPanel } from './panels/hudPanel';
// NOTE: TerminalPanel se importa de forma perezosa (lazy) dentro del comando
// 'nexus.openTerminal'. xterm/@xterm/addon-fit usan APIs de browser (self/window)
// que NO existen en el extension host de Node.js; cargarlo estáticamente rompe
// la activación completa de la extensión.
import { DiffChange } from './panels/DiffPreview';
import { initializeServices, getSessionStore, getContextDetector, getDiffPreview } from './services';

let hudPanel: AgentControlPanel | undefined;
let nexusStatusBar: vscode.StatusBarItem | undefined;
// Referencia al módulo del panel de terminal (lazy-load; se asigna al abrir).
let terminalPanel: typeof import('./panels/terminalPanel').TerminalPanel | undefined;

export function activate(context: vscode.ExtensionContext) {
  console.log('🔱 [NEXUS] Activando extensión soberana v2.0 (paridad Antigravity)...');

  // ── Inicializar servicios singleton ──────────────────────────
  initializeServices(context);

  const sessionStore = getSessionStore();
  const contextDetector = getContextDetector();
  const diffPreview = getDiffPreview();

  // Restaurar estado de sesión previo
  const savedState = sessionStore.getState();
  console.log(`🔱 [NEXUS] Sesión restaurada: ${savedState.messages.length} mensajes, ${savedState.actions.length} acciones`);

  // ── Status Bar ──────────────────────────────────────────────
  nexusStatusBar = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100
  );
  nexusStatusBar.text = '$(hubot) NEXUS';
  nexusStatusBar.command = 'nexus.openHud';
  nexusStatusBar.tooltip = '🔱 NEXUS Sovereign - Haz clic para abrir';
  nexusStatusBar.backgroundColor = new vscode.ThemeColor(
    'statusBarItem.warningBackground'
  );
  nexusStatusBar.show();
  context.subscriptions.push(nexusStatusBar);

  // ── HUD Panel ───────────────────────────────────────────────
  hudPanel = new AgentControlPanel(context);

  // ── Comandos ────────────────────────────────────────────────
  context.subscriptions.push(
    vscode.commands.registerCommand('nexus.openHud', () => {
      hudPanel?.show();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('nexus.status', async () => {
      const nexusConnected = await checkNexusCoreHealth();
      sessionStore.setNexusConnected(nexusConnected);

      if (nexusConnected) {
        vscode.window.showInformationMessage(
          '🔱 NEXUS Sovereign: 🟢 Core Online'
        );
      } else {
        vscode.window.showWarningMessage(
          '🔱 NEXUS Sovereign: 🟡 Core Offline (OpenRouter autónomo activo)'
        );
      }
    })
  );

  // ── Comando: Terminal interno (lazy load) ──────────────────
  // Carga perezosa para no romper la activación: xterm usa self/window,
  // que no existen en el extension host de Node.js. El polyfill de 'self'
  // se inyecta justo antes del require.
  context.subscriptions.push(
    vscode.commands.registerCommand('nexus.openTerminal', async () => {
      if (!terminalPanel) {
        // Polyfill: en el extension host, globalThis.self no existe.
        (globalThis as any).self = globalThis;
        // Import dinámico del panel (se evalúa solo aquí, no al activar).
        const { TerminalPanel } = await import('./panels/terminalPanel');
        terminalPanel = TerminalPanel;
      }
      terminalPanel.createOrShow(context.extensionUri);
    })
  );

  // ── Comando: Diff Preview ───────────────────────────────────
  context.subscriptions.push(
    vscode.commands.registerCommand('nexus.diffPreview', async () => {
      const editor = vscode.window.activeTextEditor;
      if (editor) {
        const doc = editor.document;
        const original = doc.getText();
        const clipboard = await vscode.env.clipboard.readText();
        if (clipboard && clipboard !== original) {
          const changes: DiffChange[] = [{
            file: vscode.workspace.asRelativePath(doc.uri),
            original,
            modified: clipboard,
            type: 'modify',
          }];
          diffPreview.show(changes);
        } else {
          vscode.window.showInformationMessage('📋 Copia algo al portapapeles primero para hacer diff');
        }
      } else {
        vscode.window.showInformationMessage('📄 Abre un archivo para hacer diff');
      }
    })
  );

  // ── Comando: Action History ─────────────────────────────────
  context.subscriptions.push(
    vscode.commands.registerCommand('nexus.actionHistory', async () => {
      const actions = sessionStore.getActions(20);
      if (actions.length === 0) {
        vscode.window.showInformationMessage('📋 No hay acciones registradas en esta sesión');
        return;
      }
      const items = actions.map(a => ({
        label: `${a.status === 'success' ? '✅' : a.status === 'error' ? '❌' : '⏳'} ${a.type}`,
        description: a.description.slice(0, 80),
        detail: new Date(a.timestamp).toLocaleString(),
      }));
      const picked = await vscode.window.showQuickPick(items, {
        placeHolder: '📋 Historial de acciones (20 más recientes)',
      });
      if (picked) {
        vscode.window.showInformationMessage(`🔍 ${picked.description}`);
      }
    })
  );

  // ── Comando: Project Context ───────────────────────────────
  context.subscriptions.push(
    vscode.commands.registerCommand('nexus.projectContext', async () => {
      const ctx = await contextDetector.detect(true); // force refresh
      const summary = contextDetector.getContextSummary(ctx);
      const doc = await vscode.workspace.openTextDocument({
        content: summary,
        language: 'markdown',
      });
      vscode.window.showTextDocument(doc);
    })
  );

  // ── Comando: Reset Session ──────────────────────────────────
  context.subscriptions.push(
    vscode.commands.registerCommand('nexus.resetSession', async () => {
      const confirm = await vscode.window.showWarningMessage(
        '⚠️ ¿Resetear toda la sesión? Se perderá el historial de chat y acciones.',
        { modal: true },
        'Sí, resetear'
      );
      if (confirm) {
        sessionStore.resetSession();
        vscode.window.showInformationMessage('🗑️ Sesión reseteada');
      }
    })
  );

  // ── Configurar atajos de teclado recomendados ─────────────
  context.subscriptions.push(
    vscode.commands.registerCommand('nexus.setupKeybindings', async () => {
      const keybindings = [
        { key: 'ctrl+shift+n', command: 'nexus.openHud', when: 'editorFocus' },
        { key: 'ctrl+shift+t', command: 'nexus.openTerminal', when: 'editorFocus' },
        { key: 'ctrl+shift+d', command: 'nexus.diffPreview', when: 'editorFocus' },
      ];
      const message = 'Atajos recomendados:\n' +
        keybindings.map(k => `  ${k.key} → ${k.command}`).join('\n') +
        '\n\nPuedes configurarlos en keyboard shortcuts (Ctrl+K Ctrl+S)';
      vscode.window.showInformationMessage(message);
    })
  );

  // ── Health check y API Key al inicio ─────────────────────────
  setTimeout(async () => {
    const connected = await checkNexusCoreHealth();
    const apiKeyConfigured = getOpenRouterApiKey() !== null;
    sessionStore.setNexusConnected(connected);
    updateStatusBar(connected, apiKeyConfigured);

    // Detectar contexto del proyecto automáticamente
    try {
      const projectCtx = await contextDetector.detect();
      console.log(`🔱 [NEXUS] Contexto detectado: ${projectCtx.name} (${projectCtx.languages.join(', ')})`);
      if (projectCtx.gitBranch) {
        console.log(`🔱 [NEXUS] Git branch: ${projectCtx.gitBranch}`);
      }
      sessionStore.addAction({
        type: 'context_detection',
        description: `Proyecto: ${projectCtx.name} | Lenguajes: ${projectCtx.languages.join(', ') || 'ninguno'}`,
        status: 'success',
      });
    } catch (e) {
      console.log('🔱 [NEXUS] No se pudo detectar contexto del proyecto:', e);
    }

    console.log(`🔱 [NEXUS] Health check: ${connected ? '🟢 Online' : '🔴 Offline'}`);
    if (!apiKeyConfigured) {
      vscode.window.showWarningMessage(
        '❌ NEXUS Sovereign: API Key de OpenRouter no configurada. ' +
        'Configura "nexus.openRouterApiKey" en las opciones de la extensión.'
      );
    }
  }, 2000);

  console.log('✅ [NEXUS] Extensión lista - Modo Autónomo (paridad Antigravity)');
}

export function deactivate() {
  console.log('🔱 [NEXUS] Desactivando extensión...');

  // Persistir estado final
  try {
    const sessionStore = getSessionStore();
    sessionStore.addAction({
      type: 'session_end',
      description: 'Extensión desactivada',
      status: 'success',
    });
  } catch (e) { /* no problem */ }

  if (hudPanel) {
    hudPanel = undefined;
  }
  if (nexusStatusBar) {
    nexusStatusBar.dispose();
    nexusStatusBar = undefined;
  }
  if (terminalPanel?.currentPanel) {
    terminalPanel.currentPanel.dispose();
  }
  try {
    getDiffPreview().dispose();
  } catch (e) { /* no problem */ }
}

export async function checkNexusCoreHealth(): Promise<boolean> {
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 3000);
    const resp = await fetch('http://localhost:43210/api/health', {
      signal: controller.signal
    });
    clearTimeout(timeout);
    return resp.ok;
  } catch {
    return false;
  }
}

/**
 * Obtiene la API key de OpenRouter desde la configuración de VS Code o variable de entorno.
 */
export function getOpenRouterApiKey(): string | null {
  const config = vscode.workspace.getConfiguration('nexus');
  return config.get<string>('openRouterApiKey') || process.env.OPENROUTER_API_KEY || null;
}

export function updateStatusBar(connected: boolean, apiKeyConfigured: boolean): void {
  if (nexusStatusBar) {
    let statusText = '$(hubot) NEXUS';
    let statusTooltip = '🔱 NEXUS Sovereign';
    let backgroundColor: vscode.ThemeColor | undefined = undefined;

    if (connected) {
      statusText += ' 🟢';
      statusTooltip += ' - Core Online';
    } else {
      statusText += ' 🔴';
      statusTooltip += ' - Core Offline (Modo Autónomo)';
      backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
    }

    if (!apiKeyConfigured) {
      statusText += ' ⚠️';
      statusTooltip += ' - (API Key OpenRouter Faltante)';
      if (!connected) {
        backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
      }
    }

    nexusStatusBar.text = statusText;
    nexusStatusBar.tooltip = statusTooltip;
    nexusStatusBar.backgroundColor = backgroundColor;
  }
}
