// ============================================================================
// 🔱 NEXUS — WebSocket Bridge v2: Conexión en tiempo real con NEXUS Core
// ============================================================================
// Protocolo NEXUS Core (nexus-shell/src/api.rs:340-490):
//
//   Server → Client:
//     {"type":"output","data":"<terminal output>"}
//     {"type":"mode","mode":"raw|normal"}
//
//   Client → Server:
//     {"type":"command","command":"<comando>"}
//     {"type":"keypress","key":"<tecla>"}
//     <texto-plano> (fallback → se ejecuta como comando)
//
// Mejoras v2:
//   - Health check HTTP previo antes de conectar WebSocket
//   - File logging a /tmp/nexus_ws_error.log para diagnóstico remoto
//   - Extracción de closeCode/reason de onclose (donde WS expone detalles reales)
//   - Reconexión silenciosa: no muestra error visual en intentos automáticos
//   - Captura detallada del error en archivo para que NEXUS lo lea directamente
// ============================================================================

import { NEXUS_API_BASE } from '../constants';
import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

// ---------------------------------------------------------------------------
// Constantes de logging
// ---------------------------------------------------------------------------
const WS_LOG_FILE = '/tmp/nexus_ws_error.log';

function logToFile(level: string, message: string, details?: any): void {
    try {
        const timestamp = new Date().toISOString();
        const detailStr = details ? ` | ${typeof details === 'object' ? JSON.stringify(details, Object.getOwnPropertyNames(details), 2) : String(details)}` : '';
        const line = `[${timestamp}] [${level}] ${message}${detailStr}\n`;
        fs.appendFileSync(WS_LOG_FILE, line);
    } catch {
        // Silencio total si falla escritura a archivo
    }
}

// ---------------------------------------------------------------------------
// Tipos del protocolo
// ---------------------------------------------------------------------------
export interface TerminalOutput {
    type: 'output';
    data: string;
}

export interface TerminalModeChange {
    type: 'mode';
    mode: 'raw' | 'normal';
}

export type ServerMessage = TerminalOutput | TerminalModeChange;

export interface WebSocketState {
    connected: boolean;
    mode: 'raw' | 'normal';
    lastOutput: string;
    reconnectAttempt: number;
}

// ---------------------------------------------------------------------------
// Callbacks
// ---------------------------------------------------------------------------
export type OutputCallback = (data: string) => void;
export type ModeChangeCallback = (mode: 'raw' | 'normal') => void;
export type ConnectionCallback = (connected: boolean) => void;

// ---------------------------------------------------------------------------
// NexusWebSocket — Singleton de conexión WS
// ---------------------------------------------------------------------------
class NexusWebSocket {
    private ws: WebSocket | null = null;
    private url: string;
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    private reconnectAttempts: number = 0;
    private maxReconnectAttempts: number = 10;
    private reconnectDelay: number = 1000; // 1s inicial, backoff ×2
    private isDisposed: boolean = false;

    // Buffering de comandos mientras desconectado
    private pendingCommands: string[] = [];

    // Estado
    private _state: WebSocketState = {
        connected: false,
        mode: 'normal',
        lastOutput: '',
        reconnectAttempt: 0,
    };

    // Callbacks
    private onOutputCallbacks: OutputCallback[] = [];
    private onModeChangeCallbacks: ModeChangeCallback[] = [];
    private onConnectionCallbacks: ConnectionCallback[] = [];

    constructor() {
        this.url = this.buildWsUrl();
        logToFile('INFO', 'NexusWebSocket v2 inicializado', { url: this.url });
    }

    /** Construir URL WS desde la base HTTP */
    private buildWsUrl(): string {
        const base = NEXUS_API_BASE.replace(/^http/, 'ws');
        return `${base}/api/terminal/ws`;
    }

    /** Obtener estado actual */
    get state(): Readonly<WebSocketState> {
        return { ...this._state };
    }

    /** ¿Está conectado? */
    get isConnected(): boolean {
        return this._state.connected;
    }

    // ------------------------------------------------------------------
    // Health Check HTTP previo (evita errores WS si core no está listo)
    // ------------------------------------------------------------------
    private async healthCheck(): Promise<boolean> {
        try {
            const httpBase = NEXUS_API_BASE;
            const controller = new AbortController();
            const timeout = setTimeout(() => controller.abort(), 2000);
            
            const res = await fetch(`${httpBase}/health`, {
                method: 'GET',
                signal: controller.signal,
            });
            
            clearTimeout(timeout);
            const ok = res.ok || res.status === 404; // 404 significa que el servidor responde
            logToFile('DEBUG', `Health check: ${ok ? 'PASS' : 'FAIL'}`, { status: res.status });
            return ok;
        } catch (err: any) {
            logToFile('WARN', `Health check failed: ${err?.message || 'unknown'}`);
            return false;
        }
    }

    // ------------------------------------------------------------------
    // Conexión (con health check previo)
    // ------------------------------------------------------------------
    async connect(): Promise<void> {
        if (this.isDisposed) return;
        if (this.ws?.readyState === WebSocket.OPEN) return;

        // ── Health check previo ──────────────────────────────────────
        const healthy = await this.healthCheck();
        if (!healthy) {
            logToFile('WARN', `Health check falló, agendando reconexión (intento #${this.reconnectAttempts + 1})`);
            this._state.connected = false;
            this.emitConnection(false);
            this.scheduleReconnect();
            return;
        }

        try {
            console.log(`🔌 [NEXUS WS] Conectando a ${this.url}...`);
            logToFile('INFO', `Conectando a ${this.url}`);
            this.ws = new WebSocket(this.url);

            this.ws.onopen = () => {
                console.log('✅ [NEXUS WS] Conectado');
                logToFile('INFO', 'WebSocket conectado exitosamente');
                this._state.connected = true;
                this._state.reconnectAttempt = 0;
                this.reconnectAttempts = 0;
                this.reconnectDelay = 1000;
                this.emitConnection(true);

                // Enviar comandos pendientes
                this.flushPendingCommands();
            };

            this.ws.onmessage = (event: MessageEvent) => {
                try {
                    const message: ServerMessage = JSON.parse(event.data);
                    this.handleMessage(message);
                } catch {
                    console.warn('⚠️ [NEXUS WS] Mensaje no-JSON recibido:', event.data);
                }
            };

            this.ws.onclose = (event: CloseEvent) => {
                const closeCode = event.code || 1006;
                const closeReason = event.reason || '(sin razón)';
                const wasClean = event.wasClean || false;
                
                console.log(
                    `🔌 [NEXUS WS] Desconectado (code=${closeCode}, reason="${closeReason}", clean=${wasClean})`
                );
                logToFile('WARN', `WebSocket cerrado`, { code: closeCode, reason: closeReason, wasClean });
                
                this._state.connected = false;
                this.ws = null;
                this.emitConnection(false);
                
                // Solo mostrar error visual si no fue cierre limpio y no es reconexión temprana
                if (!wasClean && this.reconnectAttempts > 0) {
                    const errorMsg = `❌ NEXUS WS: Desconexión inesperada (código ${closeCode})`;
                    vscode.window.showWarningMessage(errorMsg);
                }
                
                this.scheduleReconnect();
            };

            this.ws.onerror = (error: Event) => {
                // El objeto Event de WebSocket es INTENCIONALMENTE VACÍO por seguridad.
                // Los detalles reales están en onclose (code + reason).
                // Logging detallado para diagnóstico:
                const errorInfo = {
                    type: error.type,
                    time: new Date().toISOString(),
                    url: this.url,
                    note: 'WebSocket onerror no expone detalles por diseño de seguridad. Ver onclose para code/reason reales.'
                };
                console.error('❌ [NEXUS WS] Error de conexión (detalles en onclose):', errorInfo);
                logToFile('ERROR', 'Error de WebSocket (onerror)', errorInfo);
                
                // NO mostrar ventana emergente aquí - onclose se dispara después
                // La reconexión se maneja en onclose
            };
        } catch (err: any) {
            const catchErr = {
                message: err?.message || String(err),
                stack: err?.stack || '(no stack)',
                time: new Date().toISOString()
            };
            console.error('❌ [NEXUS WS] Error al crear WebSocket:', catchErr);
            logToFile('ERROR', 'Error al crear WebSocket', catchErr);
            this.scheduleReconnect();
        }
    }

    /** Desconectar voluntariamente */
    disconnect(): void {
        this.isDisposed = true;
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        if (this.ws) {
            this.ws.onclose = null; // evitar reconexión automática
            this.ws.close(1000, 'Cliente desconectado');
            this.ws = null;
        }
        this._state.connected = false;
        this.emitConnection(false);
        logToFile('INFO', 'WebSocket desconectado voluntariamente');
    }

    /** Reconectar (para reinicio manual) */
    reconnect(): void {
        this.reconnectAttempts = 0;
        this.reconnectDelay = 1000;
        if (this.ws) {
            this.ws.onclose = null;
            this.ws.close(1000, 'Reconexión manual');
        }
        this._state.connected = false;
        this.connect();
    }

    // ------------------------------------------------------------------
    // Envío de datos
    // ------------------------------------------------------------------

    /** Enviar comando de terminal */
    sendCommand(command: string): void {
        if (this.ws?.readyState === WebSocket.OPEN) {
            const msg = JSON.stringify({ type: 'command', command });
            this.ws.send(msg);
        } else {
            // Buffer para cuando reconecte
            this.pendingCommands.push(command);
            console.log('📥 [NEXUS WS] Comando en cola (desconectado):', command);
        }
    }

    /** Enviar keypress individual */
    sendKeypress(key: string): void {
        if (this.ws?.readyState === WebSocket.OPEN) {
            const msg = JSON.stringify({ type: 'keypress', key });
            this.ws.send(msg);
        }
    }

    // ------------------------------------------------------------------
    // Callbacks
    // ------------------------------------------------------------------

    onOutput(callback: OutputCallback): () => void {
        this.onOutputCallbacks.push(callback);
        return () => {
            const idx = this.onOutputCallbacks.indexOf(callback);
            if (idx !== -1) this.onOutputCallbacks.splice(idx, 1);
        };
    }

    onModeChange(callback: ModeChangeCallback): () => void {
        this.onModeChangeCallbacks.push(callback);
        return () => {
            const idx = this.onModeChangeCallbacks.indexOf(callback);
            if (idx !== -1) this.onModeChangeCallbacks.splice(idx, 1);
        };
    }

    onConnectionChange(callback: ConnectionCallback): () => void {
        this.onConnectionCallbacks.push(callback);
        return () => {
            const idx = this.onConnectionCallbacks.indexOf(callback);
            if (idx !== -1) this.onConnectionCallbacks.splice(idx, 1);
        };
    }

    // ------------------------------------------------------------------
    // Manejo interno de mensajes
    // ------------------------------------------------------------------

    private handleMessage(message: ServerMessage): void {
        switch (message.type) {
            case 'output':
                this._state.lastOutput = message.data;
                this.emitOutput(message.data);
                break;
            case 'mode':
                this._state.mode = message.mode;
                this.emitModeChange(message.mode);
                break;
        }
    }

    // ------------------------------------------------------------------
    // Reconexión con backoff exponencial
    // ------------------------------------------------------------------

    private scheduleReconnect(): void {
        if (this.isDisposed) return;

        this.reconnectAttempts++;
        this._state.reconnectAttempt = this.reconnectAttempts;

        if (this.reconnectAttempts > this.maxReconnectAttempts) {
            console.error('❌ [NEXUS WS] Máximos intentos de reconexión alcanzados');
            logToFile('ERROR', `Máximos intentos de reconexión (${this.maxReconnectAttempts}) alcanzados`);
            vscode.window.showErrorMessage(
                '❌ NEXUS WS: No se pudo reconectar después de varios intentos. Verifica que nexus-core esté corriendo.'
            );
            return;
        }

        const delay = this.reconnectDelay;
        // Backoff exponencial con techo de 30s
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000);

        console.log(`🔄 [NEXUS WS] Reconectando en ${delay}ms (intento ${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
        
        this.reconnectTimer = setTimeout(() => {
            this.connect();
        }, delay);
    }

    /** Enviar comandos encolados durante desconexión */
    private flushPendingCommands(): void {
        if (this.pendingCommands.length === 0) return;
        
        const commands = [...this.pendingCommands];
        this.pendingCommands = [];
        
        for (const cmd of commands) {
            this.sendCommand(cmd);
        }
        console.log(`📤 [NEXUS WS] ${commands.length} comando(s) enviado(s) desde la cola`);
    }

    // ------------------------------------------------------------------
    // Emisión de eventos a callbacks
    // ------------------------------------------------------------------

    private emitOutput(data: string): void {
        for (const cb of this.onOutputCallbacks) {
            try { cb(data); } catch { /* ignorar errores en callbacks */ }
        }
    }

    private emitModeChange(mode: 'raw' | 'normal'): void {
        for (const cb of this.onModeChangeCallbacks) {
            try { cb(mode); } catch { /* ignorar errores en callbacks */ }
        }
    }

    private emitConnection(connected: boolean): void {
        for (const cb of this.onConnectionCallbacks) {
            try { cb(connected); } catch { /* ignorar errores en callbacks */ }
        }
    }
}

// Exportar singleton
export const nexusWebSocket = new NexusWebSocket();
export default NexusWebSocket;
