// ============================================================================
// 🔱 NEXUS — DiffPreview: Visor de cambios antes de aplicar
// ============================================================================
// Genera un diff visual en el HUD o panel lateral
// ============================================================================

import * as vscode from 'vscode';

export interface DiffChange {
    file: string;
    original: string;
    modified: string;
    type: 'create' | 'modify' | 'delete';
}

export class DiffPreview {
    private panel: vscode.WebviewPanel | undefined;

    show(changes: DiffChange[]): void {
        const title = `🔱 NEXUS Diff — ${changes.length} cambio(s)`;

        if (this.panel) {
            this.panel.reveal(vscode.ViewColumn.Beside);
        } else {
            this.panel = vscode.window.createWebviewPanel(
                'nexusDiffPreview',
                title,
                vscode.ViewColumn.Beside,
                { enableScripts: true, retainContextWhenHidden: true }
            );
            this.panel.onDidDispose(() => { this.panel = undefined; });
        }
        this.panel.title = title;
        this.panel.webview.html = this.renderDiff(changes);
    }

    private escapeHtml(text: string): string {
        return text
            .replace(/&/g, '&')
            .replace(/</g, '<')
            .replace(/>/g, '>')
            .replace(/"/g, '"');
    }

    private renderDiff(changes: DiffChange[]): string {
        const changeCards = changes.map(c => {
            const icon = c.type === 'create' ? '🟢' : c.type === 'delete' ? '🔴' : '🟡';
            const label = c.type === 'create' ? 'CREADO' : c.type === 'delete' ? 'ELIMINADO' : 'MODIFICADO';
            
            // Generar diff línea por línea
            const origLines = c.original.split('\n');
            const modLines = c.modified.split('\n');
            const maxLines = Math.max(origLines.length, modLines.length);
            let diffHtml = '';
            
            for (let i = 0; i < maxLines; i++) {
                const origLine = origLines[i] || '';
                const modLine = modLines[i] || '';
                if (origLine === modLine) {
                    diffHtml += `<div class="line unchanged"><span class="ln">${i + 1}</span>${this.escapeHtml(origLine)}</div>`;
                } else {
                    if (origLine) {
                        diffHtml += `<div class="line removed"><span class="ln">${i + 1}</span><span class="prefix">-</span>${this.escapeHtml(origLine)}</div>`;
                    }
                    if (modLine) {
                        diffHtml += `<div class="line added"><span class="ln">${i + 1}</span><span class="prefix">+</span>${this.escapeHtml(modLine)}</div>`;
                    }
                }
            }

            return `
            <div class="change-card">
                <div class="change-header ${c.type}">
                    <span class="change-icon">${icon}</span>
                    <span class="change-file">${this.escapeHtml(c.file)}</span>
                    <span class="change-badge">${label}</span>
                </div>
                <div class="diff-content">${diffHtml}</div>
            </div>`;
        }).join('');

        return `<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
        font-family: 'Cascadia Code', 'Fira Code', monospace;
        background: #0a0e1a;
        color: #c0caf5;
        padding: 16px;
    }
    .change-card {
        margin-bottom: 16px;
        border: 1px solid #2a2d5e;
        border-radius: 6px;
        overflow: hidden;
    }
    .change-header {
        padding: 10px 14px;
        display: flex;
        align-items: center;
        gap: 10px;
        border-bottom: 1px solid #2a2d5e;
    }
    .change-header.create { background: rgba(0,255,136,0.08); }
    .change-header.modify { background: rgba(255,204,0,0.08); }
    .change-header.delete { background: rgba(255,85,85,0.08); }
    .change-file { flex: 1; font-size: 13px; }
    .change-badge {
        font-size: 10px;
        padding: 2px 8px;
        border-radius: 3px;
        background: #2a2d5e;
    }
    .diff-content { padding: 8px 0; font-size: 12px; }
    .line {
        padding: 1px 14px;
        display: flex;
        gap: 8px;
        min-height: 20px;
        align-items: center;
    }
    .line.unchanged { background: transparent; }
    .line.removed { background: rgba(255,85,85,0.1); color: #ff5555; }
    .line.added { background: rgba(0,255,136,0.1); color: #00ff88; }
    .ln {
        color: #565f89;
        min-width: 32px;
        text-align: right;
        user-select: none;
        font-size: 11px;
    }
    .prefix {
        min-width: 14px;
        font-weight: bold;
    }
    .actions {
        padding: 12px 14px;
        display: flex;
        gap: 8px;
        border-top: 1px solid #2a2d5e;
    }
    .actions button {
        padding: 6px 16px;
        border: 1px solid #3a3d7e;
        border-radius: 4px;
        background: #1a1b3a;
        color: #c0caf5;
        cursor: pointer;
        font-family: inherit;
        font-size: 12px;
        transition: all 0.2s;
    }
    .actions button:hover { background: #2a2d5e; border-color: #7aa2f7; }
    .actions button.apply { background: #00ff88; color: #0a0e1a; border-color: #00ff88; }
    .actions button.apply:hover { background: #33ffaa; }
</style>
</head>
<body>
    <h2 style="margin-bottom: 16px; font-size: 14px; color: #00ff88;">🔱 NEXUS Diff Preview</h2>
    ${changeCards}
    <div class="actions">
        <button class="apply" onclick="applyAll()">✅ Aplicar todo</button>
        <button onclick="cancelAll()">❌ Cancelar</button>
    </div>
    <script>
        const vscode = acquireVsCodeApi();
        function applyAll() { vscode.postMessage({ command: 'diffApplyAll' }); }
        function cancelAll() { vscode.postMessage({ command: 'diffCancelAll' }); }
    </script>
</body>
</html>`;
    }

    dispose(): void {
        this.panel?.dispose();
    }
}
