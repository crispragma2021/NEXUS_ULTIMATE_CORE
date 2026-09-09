import * as vscode from 'vscode';
import { Terminal } from 'xterm'; // asumo que xterm.js estará instalado
import { FitAddon } from '@xterm/addon-fit'; // para el resize
import WebSocket from 'ws'; // Node.js WebSocket client

interface TerminalMessage {
    type: 'input' | 'resize' | 'output' | 'signal' | 'status';
    data?: string;
    cols?: number;
    rows?: number;
    name?: string; // para señales
    code?: number; // para status
    message?: string; // para status
}

export class TerminalClient {
    private websocket: WebSocket | undefined;
    private terminal: Terminal;
    private fitAddon: FitAddon;
    private panel: vscode.WebviewPanel;
    private readonly NEXUS_SHELL_WS_URL = 'ws://localhost:43210/nexus/v1/terminal/ws'; // Ajustar si el puerto cambia

    constructor(panel: vscode.WebviewPanel) {
        this.panel = panel;
        this.terminal = new Terminal({
            // Configuración básica de xterm.js
            convertEol: true,
            fontSize: 14,
            fontFamily: 'Cascadia Code, monospace',
            theme: {
                background: '#0a0e1a',
                foreground: '#c0caf5',
                cursor: '#c0caf5',
                selectionBackground: '#565f89',
                black: '#1a1b3a',
                red: '#ff5555',
                green: '#00ff88',
                yellow: '#ffcc00',
                blue: '#7aa2f7',
                magenta: '#a9b1d6',
                cyan: '#41a6ff',
                white: '#c0caf5',
                brightBlack: '#565f89',
                brightRed: '#ffaaaa',
                brightGreen: '#7affcc',
                brightYellow: '#ffdd77',
                brightBlue: '#aac5ff',
                brightMagenta: '#c5ccff',
                brightCyan: '#8ee0ff',
                brightWhite: '#ffffff'
            }
        });
        this.fitAddon = new FitAddon() as any;
        this.terminal.loadAddon(this.fitAddon as any);

        // Renderizar la terminal en el webview
        this.panel.webview.html = this.getTerminalWebviewContent();
        this.panel.webview.onDidReceiveMessage(message => {
            if (message.command === 'terminalReady') {
                this.terminal.open(document.getElementById('terminal-container')!);
                this.fitAddon.fit();
                this.connectWebSocket();
            }
        });

        // Manejar el redimensionamiento del panel del Webview
        this.panel.onDidChangeViewState(e => {
            // Un pequeño retraso para asegurar que el DOM se ha actualizado
            setTimeout(() => {
                this.fitAddon.fit();
                this.sendResizeMessage();
            }, 100);
        });

        // Manejar la entrada de la terminal (teclado)
        this.terminal.onData(data => {
            this.sendInput(data);
        });
    }

    private getTerminalWebviewContent(): string {
        return `<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NEXUS Terminal</title>
    <style>
        body { margin: 0; padding: 0; height: 100vh; overflow: hidden; background-color: #0a0e1a; }
        #terminal-container { width: 100%; height: 100%; }
    </style>
    <link rel="stylesheet" href="https://unpkg.com/xterm@5.5.0/css/xterm.css" />
    <script src="https://unpkg.com/xterm@5.5.0/lib/xterm.js"></script>
    <script src="https://unpkg.com/@xterm/addon-fit@0.8.0/lib/xterm-addon-fit.js"></script>
</head>
<body>
    <div id="terminal-container"></div>
    <script>
        const vscode = acquireVsCodeApi();
        const terminal = new Terminal();
        const fitAddon = new FitAddon();
        terminal.loadAddon(fitAddon);
        
        terminal.open(document.getElementById('terminal-container'));
        fitAddon.fit(); // Ajustar el tamaño inicial
        
        // Notificar a la extensión que la terminal está lista
        vscode.postMessage({ command: 'terminalReady' });

        // Evento de redimensionamiento del navegador (o panel del webview)
        window.addEventListener('resize', () => {
            fitAddon.fit();
            vscode.postMessage({ command: 'resize', cols: terminal.cols, rows: terminal.rows });
        });

        // Manejar la entrada de teclado desde xterm.js
        terminal.onData(data => {
            vscode.postMessage({ command: 'input', data: data });
        });

        // Escuchar mensajes de la extensión (salida del shell)
        window.addEventListener('message', event => {
            const message = event.data;
            if (message.type === 'output') {
                terminal.write(message.data);
            }
        });

        // Para pruebas, si necesitas enviar un mensaje de resize inicial
        setTimeout(() => {
             vscode.postMessage({ command: 'resize', cols: terminal.cols, rows: terminal.rows });
        }, 0);
    </script>
</body>
</html>`;
    }

    private connectWebSocket() {
        this.websocket = new WebSocket(this.NEXUS_SHELL_WS_URL);

        this.websocket.onopen = () => {
            this.terminal.write('\x1b[32mConectado a NEXUS Shell.\x1b[0m\r\n');
            this.sendResizeMessage(); // Enviar tamaño inicial al conectarse
        };

        this.websocket.onmessage = event => {
            try {
                const msg: TerminalMessage = JSON.parse(event.data.toString());
                if (msg.type === 'output' && msg.data) {
                    this.terminal.write(msg.data);
                } else if (msg.type === 'status' && msg.message) {
                    this.terminal.write(`\r\n\x1b[31m${msg.message}\x1b[0m\r\n`);
                }
            } catch (e) {
                this.terminal.write(`\r\n\x1b[31mError procesando mensaje del servidor: ${e}\x1b[0m\r\n`);
                console.error("Error procesando mensaje WebSocket:", e, event.data.toString());
            }
        };

        this.websocket.onclose = () => {
            this.terminal.write('\r\n\x1b[31mDesconectado de NEXUS Shell.\x1b[0m\r\n');
        };

        this.websocket.onerror = err => {
            this.terminal.write(`\r\n\x1b[31mError de conexión: ${err.message}\x1b[0m\r\n`);
            console.error("WebSocket error:", err);
        };
    }

    private sendInput(data: string) {
        if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
            const msg: TerminalMessage = { type: 'input', data };
            this.websocket.send(JSON.stringify(msg));
        }
    }

    private sendResizeMessage() {
        if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
            const msg: TerminalMessage = { type: 'resize', cols: this.terminal.cols, rows: this.terminal.rows };
            this.websocket.send(JSON.stringify(msg));
        }
    }

    public dispose() {
        if (this.websocket) {
            this.websocket.close();
        }
        this.terminal.dispose();
    }
}
