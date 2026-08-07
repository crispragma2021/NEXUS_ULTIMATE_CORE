// ============================================================================
// 🔱 NEXUS — ModeTreeProvider: ELIMINADO (sin modos)
// ============================================================================
// La extensión NEXUS Sovereign es autónoma sin modos (como Antigravity).
// Este archivo se mantiene como stub para evitar errores de import.
// ============================================================================

import * as vscode from 'vscode';

export class ModeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly isActive: boolean,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(label, collapsibleState);
    }
}

export class ModeTreeProvider implements vscode.TreeDataProvider<ModeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<ModeItem | undefined | null | void> =
        new vscode.EventEmitter<ModeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<ModeItem | undefined | null | void> =
        this._onDidChangeTreeData.event;

    getTreeItem(element: ModeItem): vscode.TreeItem {
        return element;
    }

    getChildren(_element?: ModeItem): Thenable<ModeItem[]> {
        return Promise.resolve([]);
    }
}
