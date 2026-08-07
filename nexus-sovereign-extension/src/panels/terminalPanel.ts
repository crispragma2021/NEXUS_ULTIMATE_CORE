import * as vscode from 'vscode';
import { TerminalClient } from '../terminal/TerminalClient'; // Importar el cliente de la terminal

export class TerminalPanel {
    public static currentPanel: TerminalPanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private _disposables: vscode.Disposable[] = [];
    private _terminalClient: TerminalClient;

    public static createOrShow(extensionUri: vscode.Uri) {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        // Si ya tenemos un panel, simplemente muéstralo.
        if (TerminalPanel.currentPanel) {
            TerminalPanel.currentPanel._panel.reveal(column);
            return;
        }

        // Sino, crea un nuevo panel.
        const panel = vscode.window.createWebviewPanel(
            'nexusTerminal',
            'NEXUS Terminal',
            column || vscode.ViewColumn.One,
            {
                enableScripts: true,
                localResourceRoots: [
                    vscode.Uri.joinPath(extensionUri, 'media') // Si necesitas recursos locales
                ]
            }
        );

        TerminalPanel.currentPanel = new TerminalPanel(panel, extensionUri);
    }

    private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
        this._panel = panel;
        this._terminalClient = new TerminalClient(this._panel); // Pasar el panel al cliente

        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
    }

    public show() {
        this._panel.reveal(vscode.ViewColumn.Beside);
    }

    public dispose() {
        TerminalPanel.currentPanel = undefined;

        this._panel.dispose();
        this._terminalClient.dispose(); // Asegurarse de limpiar el cliente de la terminal

        while (this._disposables.length) {
            const x = this._disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }
}
