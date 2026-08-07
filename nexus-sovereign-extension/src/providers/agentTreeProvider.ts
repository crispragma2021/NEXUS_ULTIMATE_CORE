// ============================================================================
// 🔱 NEXUS — AgentTreeProvider: Árbol de agentes NEXUS
// ============================================================================
// ABSORBIDO de Antigravity (agentTreeProvider.ts) y MEJORADO:
//   - Mismos patrones TreeDataProvider + TreeItem
//   - Escaneo del directorio .agent/agents/
//   - Iconos y colores por estado (idle/active/error)
//   - Tooltip expandido
//   - Comando launchAgent al hacer clic
// ============================================================================

import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

// ---------------------------------------------------------------------------
// AgentItem — Cada agente en el árbol
// ---------------------------------------------------------------------------
export class AgentItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        private status: 'idle' | 'active' | 'error',
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(label, collapsibleState);

        this.tooltip = `🤖 Agente: ${label}\nEstado: ${status}\nClic para lanzar`;
        this.description = status;

        // Comando al hacer clic
        this.command = {
            command: 'nexus.launchAgent',
            title: 'Launch Agent',
            arguments: [this],
        };

        // Icono según estado
        switch (status) {
            case 'active':
                this.iconPath = new vscode.ThemeIcon(
                    'play-circle',
                    new vscode.ThemeColor('charts.green')
                );
                this.resourceUri = vscode.Uri.parse(`command:agent?status=active`);
                break;
            case 'error':
                this.iconPath = new vscode.ThemeIcon(
                    'error',
                    new vscode.ThemeColor('charts.red')
                );
                this.resourceUri = vscode.Uri.parse(`command:agent?status=error`);
                break;
            case 'idle':
            default:
                this.iconPath = new vscode.ThemeIcon('record');
                break;
        }
    }

    /** Cambiar estado dinámicamente */
    setStatus(newStatus: 'idle' | 'active' | 'error'): void {
        this.status = newStatus;
        this.description = newStatus;
        this.tooltip = `🤖 Agente: ${this.label}\nEstado: ${newStatus}`;
    }
}

// ---------------------------------------------------------------------------
// AgentTreeProvider — Proveedor de datos para el árbol de agentes
// ---------------------------------------------------------------------------
export class AgentTreeProvider implements vscode.TreeDataProvider<AgentItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<AgentItem | undefined | null | void> =
        new vscode.EventEmitter<AgentItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<AgentItem | undefined | null | void> =
        this._onDidChangeTreeData.event;

    /** Mapa de estados de agentes conocido (persistente en memoria) */
    private agentStates: Map<string, 'idle' | 'active' | 'error'> = new Map();

    constructor(private context: vscode.ExtensionContext) {}

    /** Refrescar el árbol (ree-scanea el directorio) */
    refresh(): void {
        this._onDidChangeTreeData.fire(undefined);
    }

    getTreeItem(element: AgentItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: AgentItem): Thenable<AgentItem[]> {
        if (!element) {
            return this.getAgents();
        }
        return Promise.resolve([]);
    }

    /** Escanear el directorio .agent/agents/ del workspace */
    private getAgents(): Promise<AgentItem[]> {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) {
            return Promise.resolve([]);
        }

        const agentDir = path.join(
            workspaceFolders[0].uri.fsPath,
            '.agent',
            'agents'
        );

        return new Promise(resolve => {
            if (!fs.existsSync(agentDir)) {
                resolve([]);
                return;
            }

            try {
                const files = fs.readdirSync(agentDir);
                const agents = files
                    .filter(file => {
                        const fullPath = path.join(agentDir, file);
                        return fs.statSync(fullPath).isDirectory();
                    })
                    .map(agentName => {
                        // Preservar estado conocido si existe
                        const knownStatus = this.agentStates.get(agentName) || 'idle';
                        return new AgentItem(
                            agentName,
                            knownStatus,
                            vscode.TreeItemCollapsibleState.None
                        );
                    });

                resolve(agents);
            } catch (e) {
                console.error('❌ [NEXUS] Error leyendo agentes:', e);
                resolve([]);
            }
        });
    }

    /** Actualizar estado de un agente */
    updateAgentState(agentName: string, status: 'idle' | 'active' | 'error'): void {
        this.agentStates.set(agentName, status);
        this.refresh();
    }
}
