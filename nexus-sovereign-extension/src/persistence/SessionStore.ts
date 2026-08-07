// ============================================================================
// 🔱 NEXUS — SessionStore: Persistencia local con SQLite (via sql.js)
// ============================================================================
// Almacena: historial de chat, acciones, agentes, estado de sesión
// Resiste recargas de VS Code — todo queda en disco
// ============================================================================

import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

export interface ChatMessage {
    role: 'user' | 'assistant' | 'system';
    content: string;
    timestamp: number;
    id: string;
}

export interface StoredAction {
    id: string;
    type: string;
    description: string;
    timestamp: number;
    status: 'success' | 'error' | 'pending';
    details?: string;
}

export interface SessionState {
    version: number;
    lastActive: number;
    selectedAgent: string | null;
    messages: ChatMessage[];
    actions: StoredAction[];
    nexusConnected: boolean;
}

export class SessionStore {
    private static instance: SessionStore;
    private storagePath: string;
    private state: SessionState;
    private saveTimer: ReturnType<typeof setTimeout> | null = null;
    private readonly SAVE_DELAY = 500; // ms debounce

    private constructor(context: vscode.ExtensionContext) {
        // Almacenar en globalStoragePath para que sobreviva recargas
        this.storagePath = path.join(context.globalStoragePath, 'session.json');
        this.state = this.load();
    }

    static getInstance(context?: vscode.ExtensionContext): SessionStore {
        if (!SessionStore.instance) {
            if (!context) throw new Error('SessionStore requiere ExtensionContext en primera inicialización');
            SessionStore.instance = new SessionStore(context);
        }
        return SessionStore.instance;
    }

    // ── Carga/Salvado ────────────────────────────────────────────

    private load(): SessionState {
        try {
            if (fs.existsSync(this.storagePath)) {
                const raw = fs.readFileSync(this.storagePath, 'utf-8');
                const parsed = JSON.parse(raw);
                // Validar versión
                if (parsed.version === 1) {
                    return parsed as SessionState;
                }
            }
        } catch (e) {
            console.error('❌ [SessionStore] Error cargando sesión:', e);
        }
        return this.defaultState();
    }

    private defaultState(): SessionState {
        return {
            version: 1,
            lastActive: Date.now(),
            selectedAgent: null,
            messages: [],
            actions: [],
            nexusConnected: false,
        };
    }

    private save(): void {
        // Debounce para no escribir en cada operación
        if (this.saveTimer) clearTimeout(this.saveTimer);
        this.saveTimer = setTimeout(() => {
            try {
                this.state.lastActive = Date.now();
                const dir = path.dirname(this.storagePath);
                if (!fs.existsSync(dir)) {
                    fs.mkdirSync(dir, { recursive: true });
                }
                fs.writeFileSync(this.storagePath, JSON.stringify(this.state, null, 2), 'utf-8');
            } catch (e) {
                console.error('❌ [SessionStore] Error guardando sesión:', e);
            }
        }, this.SAVE_DELAY);
    }

    // ── Getters ──────────────────────────────────────────────────

    getState(): Readonly<SessionState> {
        return { ...this.state };
    }

    getMessages(): ChatMessage[] {
        return [...this.state.messages];
    }

    getActions(limit?: number): StoredAction[] {
        const actions = [...this.state.actions].sort((a, b) => b.timestamp - a.timestamp);
        return limit ? actions.slice(0, limit) : actions;
    }

    getSelectedAgent(): string | null {
        return this.state.selectedAgent;
    }

    // ── Mutaciones ───────────────────────────────────────────────

    addMessage(msg: Omit<ChatMessage, 'id' | 'timestamp'>): void {
        const message: ChatMessage = {
            ...msg,
            id: `msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
            timestamp: Date.now(),
        };
        this.state.messages.push(message);
        // Mantener máx 500 mensajes
        if (this.state.messages.length > 500) {
            this.state.messages = this.state.messages.slice(-500);
        }
        this.save();
    }

    clearMessages(): void {
        this.state.messages = [];
        this.save();
    }

    addAction(action: Omit<StoredAction, 'id' | 'timestamp'>): void {
        const stored: StoredAction = {
            ...action,
            id: `act_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
            timestamp: Date.now(),
        };
        this.state.actions.push(stored);
        // Mantener máx 200 acciones
        if (this.state.actions.length > 200) {
            this.state.actions = this.state.actions.slice(-200);
        }
        this.save();
    }

    setSelectedAgent(agent: string | null): void {
        this.state.selectedAgent = agent;
        this.save();
    }

    setNexusConnected(connected: boolean): void {
        this.state.nexusConnected = connected;
        this.save();
    }

    // ── Reset ────────────────────────────────────────────────────

    resetSession(): void {
        this.state = this.defaultState();
        this.save();
    }
}
