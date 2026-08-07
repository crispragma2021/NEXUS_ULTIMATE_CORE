import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import * as http from 'http';

const NEXUS_API = 'http://localhost:43210';

interface AgentInfo {
	name: string;
	status: 'idle' | 'active' | 'error';
}

export class AgentControlPanel {
	private panel: vscode.WebviewPanel | undefined;
	private context: vscode.ExtensionContext;
	private agents: Map<string, AgentInfo> = new Map();
	private selectedAgent: string | null = null;

	constructor(context: vscode.ExtensionContext) {
		this.context = context;
		this.loadAgents();
	}

	private loadAgents() {
		const workspaceFolders = vscode.workspace.workspaceFolders;
		if (!workspaceFolders) return;

		const agentDir = path.join(workspaceFolders[0].uri.fsPath, '.agent', 'agents');
		
		try {
			if (fs.existsSync(agentDir)) {
				const agents = fs.readdirSync(agentDir);
				agents.forEach(agent => {
					if (!this.agents.has(agent)) {
						this.agents.set(agent, { name: agent, status: 'idle' });
					}
				});
			}
		} catch (e) {
			console.error('Error cargando agentes:', e);
		}
	}

	public show() {
		if (this.panel) {
			this.panel.reveal(vscode.ViewColumn.Beside);
		} else {
			this.panel = vscode.window.createWebviewPanel(
				'antigravityControl',
				'🤖 Antigravity Agent Control',
				vscode.ViewColumn.Beside,
				{ enableScripts: true, retainContextWhenHidden: true }
			);

			this.panel.webview.html = this.getWebviewContent();
			
			this.panel.webview.onDidReceiveMessage(
				message => this.handleMessage(message),
				undefined,
				this.context.subscriptions
			);

			this.panel.onDidDispose(
				() => { this.panel = undefined; },
				null,
				this.context.subscriptions
			);

			// Auto-verificar conexión con NEXUS
			this.verificarConexionNexus();
		}
	}

	private async verificarConexionNexus() {
		try {
			const alive = await this.healthCheck();
			if (this.panel) {
				this.panel.webview.postMessage({
					command: 'nexusStatus',
					connected: alive
				});
			}
		} catch {
			if (this.panel) {
				this.panel.webview.postMessage({
					command: 'nexusStatus',
					connected: false
				});
			}
		}
	}

	private healthCheck(): Promise<boolean> {
		return new Promise((resolve) => {
			const req = http.get(`${NEXUS_API}/api/health`, (res) => {
				resolve(res.statusCode === 200);
			});
			req.on('error', () => resolve(false));
			req.setTimeout(2000, () => { req.destroy(); resolve(false); });
		});
	}

	private consultarNexus(prompt: string): Promise<string> {
		return new Promise((resolve, reject) => {
			const body = JSON.stringify({ prompt, modelo: 'orquestador' });
			const req = http.request(`${NEXUS_API}/api/consultar`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'Content-Length': Buffer.byteLength(body)
				}
			}, (res) => {
				let data = '';
				res.on('data', chunk => data += chunk);
				res.on('end', () => {
					if (res.statusCode === 200) {
						try {
							const parsed = JSON.parse(data);
							resolve(parsed.respuesta || parsed.response || data);
						} catch {
							resolve(data);
						}
					} else {
						reject(new Error(`NEXUS respondió ${res.statusCode}: ${data}`));
					}
				});
			});
			req.on('error', reject);
			req.setTimeout(10000, () => { req.destroy(); reject(new Error('Timeout consultando NEXUS')); });
			req.write(body);
			req.end();
		});
	}


	private async getActionHistoryFromNexus(): Promise<any[]> {
		const result = await vscode.commands.executeCommand('tauri.invoke', 'get_historial_acciones');
		return JSON.parse(result as string);
	}

	private async deleteActionFromNexus(contexto: number): Promise<void> {
		await vscode.commands.executeCommand('tauri.invoke', 'eliminar_historial_accion', { contexto });
	}

	private async handleMessage(message: any) {
		switch (message.command) {
			case 'getAgents':
				this.loadAgents();
				this.panel?.webview.postMessage({
					command: 'agentList',
					agents: Array.from(this.agents.values())
				});
				break;

			case 'executeAgent':
				console.log(`🔗 Conectando agente: ${message.agent} a NEXUS...`);
				this.selectedAgent = message.agent;
				this.agents.set(message.agent, { name: message.agent, status: 'active' });
				this.panel?.webview.postMessage({
					command: 'agentStarted',
					agent: message.agent
				});
				vscode.window.showInformationMessage(`🚀 ${message.agent} conectado a NEXUS Orquestador`);

				// Verificar salud de NEXUS
				const alive = await this.healthCheck();
				this.panel?.webview.postMessage({
					command: 'nexusStatus',
					connected: alive
				});
				break;

			case 'sendCommand':
				if (!this.selectedAgent) {
					this.panel?.webview.postMessage({
						command: 'commandResponse',
						response: '❌ Selecciona un agente primero'
					});
					break;

				}
				console.log(`📤 Enviando a NEXUS [${this.selectedAgent}]: ${message.command}`);
				try {
					// Timeout visible en UI
					this.panel?.webview.postMessage({
						command: 'thinking',
						agent: this.selectedAgent
					});

					const respuesta = await this.consultarNexus(message.command);
					this.panel?.webview.postMessage({
						command: 'commandResponse',
						response: respuesta
					});
				} catch (err: any) {
					console.error('❌ Error consultando NEXUS:', err.message);
					this.panel?.webview.postMessage({
						command: 'commandResponse',
						response: `⚠️ Error: ${err.message}`
					});
					this.panel?.webview.postMessage({
						command: 'nexusStatus',
						connected: false
					});
				}
				break;

			case 'checkNexus':
				const connected = await this.healthCheck();
				if (this.panel) {
					this.panel.webview.postMessage({
						command: 'nexusStatus',
						connected
					});
				}
				break;

			case 'getActionHistory':
				try {
					const history = await this.getActionHistoryFromNexus();
					this.panel?.webview.postMessage({ command: 'actionHistoryLoaded', history });
				} catch (error: any) {
					console.error('Error obteniendo historial:', error.message);
					this.panel?.webview.postMessage({ command: 'actionHistoryLoaded', history: [] }); // Enviar vacío en caso de error
				}
				break;

			case 'deleteAction':
				try {
					await vscode.commands.executeCommand('tauri.invoke', 'eliminar_historial_accion', { contexto: message.contexto });
					this.panel?.webview.postMessage({ command: 'actionDeleted', contexto: message.contexto });
				} catch (error: any) {
					console.error('Error eliminando acción:', error.message);
					vscode.window.showErrorMessage(`Error eliminando acción: ${error.message}`);
				}
				break;
		}
	}

	public updateAgentList() {
		this.loadAgents();
		if (this.panel) {
			this.panel.webview.postMessage({ command: 'agentListUpdated', agents: Array.from(this.agents.values()) });
		}
	}

	public selectAgent(agentName: string) {
		this.selectedAgent = agentName;
		if (this.panel) {
			this.panel.webview.postMessage({ command: 'selectAgent', agent: agentName });
		}
	}

	private getWebviewContent(): string {
		return `<!DOCTYPE html>
<html lang="es">
<head>
	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>Antigravity Control</title>
	<style>
		* { margin: 0; padding: 0; box-sizing: border-box; }
		body {
			font-family: 'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace;
			background: #0a0e1a;
			color: #c0caf5;
			height: 100vh;
			overflow: hidden;
		}
		.container {
			display: flex;
			flex-direction: column;
			height: 100vh;
			background: linear-gradient(135deg, #0a0e1a 0%, #1a1b3a 100%);
		}
		.header {
			background: linear-gradient(90deg, #0f1535, #1a1b4e);
			padding: 12px 20px;
			border-bottom: 1px solid #2a2d5e;
			display: flex;
			justify-content: space-between;
			align-items: center;
			animation: glitch 3s infinite;
		}
		@keyframes glitch {
			2%, 64% { transform: translate(0); }
			4%, 60% { transform: translate(-1px, 0); }
			62% { transform: translate(1px, 0); }
		}
		.header h1 {
			font-size: 14px;
			color: #00ff88;
			text-shadow: 0 0 10px rgba(0,255,136,0.3);
			letter-spacing: 2px;
		}
		.nexus-badge {
			font-size: 11px;
			padding: 3px 10px;
			border-radius: 3px;
			border: 1px solid #2a2d5e;
		}
		.nexus-badge.connected {
			color: #00ff88;
			border-color: #00ff88;
			background: rgba(0,255,136,0.1);
		}
		.nexus-badge.disconnected {
			color: #ff5555;
			border-color: #ff5555;
			background: rgba(255,85,85,0.1);
		}
		.main {
			display: flex;
			flex: 1;
			overflow: hidden;
		}
		.sidebar {
			width: 200px;
			background: #0d1230;
			border-right: 1px solid #1e2250;
			padding: 10px 0;
			display: flex;
			flex-direction: column;
		}
		.sidebar-title {
			padding: 8px 15px;
			font-size: 10px;
			color: #565f89;
			text-transform: uppercase;
			letter-spacing: 2px;
		}
		.agent-item {
			padding: 8px 15px;
			cursor: pointer;
			display: flex;
			justify-content: space-between;
			align-items: center;
			transition: all 0.2s;
			border-left: 2px solid transparent;
		}
		.agent-item:hover {
			background: rgba(255,255,255,0.03);
			border-left-color: #565f89;
		}
		.agent-item.active {
			background: rgba(0,255,136,0.08);
			border-left-color: #00ff88;
		}
		.agent-name {
			font-size: 12px;
			color: #c0caf5;
		}
		.agent-status {
			font-size: 10px;
			color: #565f89;
			margin-top: 2px;
		}
		.status-indicator {
			width: 8px;
			height: 8px;
			border-radius: 50%;
			background: #565f89;
		}
		.status-indicator.active {
			background: #00ff88;
			box-shadow: 0 0 6px rgba(0,255,136,0.5);
		}
		.empty-state {
			padding: 20px 15px;
			font-size: 11px;
			color: #565f89;
			text-align: center;
		}
		.chat-area {
			flex: 1;
			display: flex;
			flex-direction: column;
		}
		.messages {
			flex: 1;
			overflow-y: auto;
			padding: 15px;
		}
		.message {
			margin-bottom: 8px;
			padding: 8px 12px;
			border-radius: 4px;
			font-size: 12px;
			line-height: 1.5;
		}
		.message.system {
			background: rgba(255,255,255,0.03);
			border-left: 2px solid #565f89;
			color: #a9b1d6;
		}
		.message.user {
			background: rgba(0,255,136,0.05);
			border-left: 2px solid #00ff88;
			color: #00ff88;
		}
		.message.agent {
			background: rgba(122,162,247,0.05);
			border-left: 2px solid #7aa2f7;
			color: #c0caf5;
			white-space: pre-wrap;
		}
		.message.thinking {
			background: rgba(255,204,0,0.05);
			border-left: 2px solid #ffcc00;
			color: #a9b1d6;
			font-style: italic;
		}
		.message.error {
			background: rgba(255,85,85,0.05);
			border-left: 2px solid #ff5555;
			color: #ff5555;
		}
		.input-area {
			padding: 10px 15px;
			background: #0d1230;
			border-top: 1px solid #1e2250;
			display: flex;
			gap: 8px;
		}
		.input-area input {
			flex: 1;
			padding: 8px 12px;
			background: #1a1b3a;
			border: 1px solid #2a2d5e;
			border-radius: 4px;
			color: #c0caf5;
			font-family: inherit;
			font-size: 12px;
			outline: none;
			transition: border-color 0.2s;
		}
		.input-area input:focus {
			border-color: #7aa2f7;
		}
		.input-area button {
			padding: 8px 16px;
			background: #2a2d5e;
			border: 1px solid #3a3d7e;
			border-radius: 4px;
			color: #c0caf5;
			cursor: pointer;
			font-family: inherit;
			font-size: 12px;
			transition: all 0.2s;
		}
		.input-area button:hover {
			background: #3a3d7e;
			border-color: #7aa2f7;
		}
		.toolbar {
			display: flex;
			gap: 4px;
			padding: 6px 15px;
			background: #0d1230;
			border-bottom: 1px solid #1e2250;
		}
		.toolbar button {
			padding: 3px 10px;
			background: transparent;
			border: 1px solid #2a2d5e;
			border-radius: 3px;
			color: #565f89;
			cursor: pointer;
			font-size: 10px;
			transition: all 0.2s;
		}
		.toolbar button:hover {
			color: #c0caf5;
			border-color: #565f89;
		}
		.agent-status-msg {
			padding: 10px 15px;
			font-size: 11px;
			color: #565f89;
			border-bottom: 1px solid #1e2250;
		}
		::-webkit-scrollbar {
			width: 4px;
		}
		::-webkit-scrollbar-track {
			background: #0d1230;
		}
		::-webkit-scrollbar-thumb {
			background: #2a2d5e;
			border-radius: 2px;
		}
		.action-item {
			padding: 8px 15px;
			cursor: pointer;
			display: flex;
			justify-content: space-between;
			align-items: center;
			transition: all 0.2s;
			border-left: 2px solid transparent;
		}
		.action-item:hover {
			background: rgba(255,255,255,0.03);
			border-left-color: #565f89;
		}
		.action-item:hover .delete-action-btn {
			opacity: 1;
		}
		.action-title {
			font-size: 12px;
			color: #c0caf5;
			white-space: nowrap;
			overflow: hidden;
			text-overflow: ellipsis;
			flex-grow: 1;
		}
		.action-time {
			font-size: 10px;
			color: #565f89;
			margin-left: 10px;
			flex-shrink: 0;
		}
		.delete-action-btn {
			background: none;
			border: none;
			color: #ff5555;
			cursor: pointer;
			font-size: 14px;
			margin-left: 10px;
			padding: 0 5px;
			opacity: 0; /* Hidden by default */
			transition: opacity 0.2s;
			flex-shrink: 0;
		}
	
        .roo-dropdown {
            background: #1e1e1e;
            border: 1px solid #333;
            border-radius: 4px;
            color: #ccc;
            padding: 8px 12px;
            font-size: 13px;
            cursor: pointer;
            display: flex;
            justify-content: space-between;
            align-items: center;
            width: 250px;
            position: relative;
        }
        .roo-dropdown-menu {
            display: none;
            position: absolute;
            top: 100%;
            left: 0;
            width: 250px;
            background: #1e1e1e;
            border: 1px solid #333;
            border-radius: 4px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.3);
            z-index: 1000;
            margin-top: 4px;
            max-height: 300px;
            overflow-y: auto;
        }
        .roo-dropdown-menu.show {
            display: block;
        }
        .roo-item {
            padding: 8px 12px;
            cursor: pointer;
            display: flex;
            justify-content: space-between;
            align-items: center;
            color: #ccc;
        }
        .roo-item:hover {
            background: #2a2d2e;
        }
        .roo-item.selected {
            color: #cca700;
        }
        .roo-item-title {
            font-weight: 500;
        }
        .roo-item-subtitle {
            font-size: 11px;
            color: #666;
            margin-left: 8px;
        }
        .roo-label {
            font-size: 10px;
            color: #666;
            text-transform: uppercase;
            padding: 8px 12px 4px 12px;
            letter-spacing: 1px;
        }

	</style>
</head>
<body>
	<div class="container">
		<div class="header">
			<h1>⎔ ANTIGRAVITY</h1>
			<span id="nexusBadge" class="nexus-badge disconnected">⏳ NEXUS...</span>
		</div>
		<div class="main">
			<div class="sidebar">
				
        <div style="padding: 15px; border-bottom: 1px solid #1e2250;">
            <div class="roo-label">MODEL</div>
            <div class="roo-dropdown" onclick="document.getElementById('rooMenu').classList.toggle('show')">
                <span id="rooSelectedText" style="color: #cca700; font-weight: bold;">NEXUS Local</span>
                <span>▼</span>
                <div class="roo-dropdown-menu" id="rooMenu">
                    <div class="roo-item selected" onclick="selectRooModel('NEXUS Local', event)">
                        <div><span class="roo-item-title">NEXUS Local</span></div>
                        <span>✓</span>
                    </div>
                    <div class="roo-item" onclick="selectRooModel('NEXUS Puro', event)">
                        <div><span class="roo-item-title">NEXUS Puro</span></div>
                    </div>
                    <div class="roo-item" onclick="selectRooModel('DeepSeek v4 flash', event)">
                        <div><span class="roo-item-title">DeepSeek v4 flash</span><span class="roo-item-subtitle">deepseek-v4-flash</span></div>
                    </div>
                    <div class="roo-item" onclick="selectRooModel('Gemini 3.1 Flash', event)">
                        <div><span class="roo-item-title">Gemini 3.1 Flash</span><span class="roo-item-subtitle">pro-preview</span></div>
                    </div>
                </div>
            </div>
        </div>

				<div class="sidebar-title">🤖 Agentes</div>
				<div id="agentList"></div>
				<div class="sidebar-title" style="margin-top: 20px;">📜 Historial</div>
				<div id="actionHistoryList" style="overflow-y: auto; flex-grow: 1;">
					<div class="empty-state">Cargando historial...</div>
				</div>
			</div>
			<div class="chat-area" id="chatArea">
				<div class="toolbar">
					<button onclick="refreshAgents()">🔄 Recargar</button>
					<button onclick="startAgent()">🚀 Conectar</button>
					<button onclick="clearChat()">🗑️ Limpiar</button>
					<button onclick="checkNexus()">📡 NEXUS</button>
				</div>
				<div class="agent-status-msg" id="agentStatus">
					Selecciona un agente de la lista
				</div>
				<div class="messages" id="messages"></div>
				<div class="input-area">
					<input type="text" id="commandInput" placeholder="Escribe tu comando para NEXUS..." />
					<button onclick="sendCommand()">⚡</button>
				</div>
			</div>
		</div>
	</div>

	<script>
		const vscode = acquireVsCodeApi();
		let selectedAgent = null;
		let agents = [];

		function loadAgents() {
			vscode.postMessage({ command: 'getAgents' });
		}

		function renderAgents() {
			const agentList = document.getElementById('agentList');
			
			if (agents.length === 0) {
				agentList.innerHTML = '<div class="empty-state">No hay agentes disponibles</div>';
				return;
			}

			agentList.innerHTML = agents.map(agent => \`
				<div class="agent-item \${selectedAgent === agent.name ? 'active' : ''}" onclick="selectAgent('\${agent.name}')">
					<div>
						<div class="agent-name">🤖 \${agent.name}</div>
						<div class="agent-status">\${agent.status === 'active' ? '🟢 Conectado' : '⚪ Inactivo'}</div>
					</div>
					<span><span class="status-indicator \${agent.status === 'active' ? 'active' : ''}"></span></span>
				</div>
			\`).join('');
		}

		
        function selectRooModel(name, event) {
            event.stopPropagation();
            document.getElementById('rooSelectedText').textContent = name;
            document.getElementById('rooMenu').classList.remove('show');
            
            // Remove selected class from all
            document.querySelectorAll('.roo-item').forEach(el => {
                el.classList.remove('selected');
                let check = el.querySelector('span:last-child');
                if(check && check.textContent === '✓') check.remove();
            });
            
            // Add selected class to clicked
            let target = event.currentTarget;
            target.classList.add('selected');
            let checkSpan = document.createElement('span');
            checkSpan.textContent = '✓';
            target.appendChild(checkSpan);
            
            addSystemMessage(\`Modelo cambiado a: \${name}\`);
            // Aquí iría el vscode.postMessage para notificar al backend
        }

		function selectAgent(agentName) {
			selectedAgent = agentName;
			renderAgents();
			document.getElementById('chatArea').classList.add('visible');
			document.getElementById('agentStatus').innerHTML = 
				\`<span style="color: #00ff88;">◆ Conectado con: <strong>\${agentName}</strong></span>\`;
			addSystemMessage(\`Conectado con el agente \${agentName}\`);
			addSystemMessage(\`💡 Listo para consultar a NEXUS Orquestador\`);
		}

		function addSystemMessage(text) {
			const messages = document.getElementById('messages');
			const msg = document.createElement('div');
			msg.className = 'message system';
			msg.textContent = \`[SYSTEM] \${text}\`;
			messages.appendChild(msg);
			messages.scrollTop = messages.scrollHeight;
		}

		function addAgentMessage(text) {
			const messages = document.getElementById('messages');
			const msg = document.createElement('div');
			msg.className = 'message agent';
			msg.textContent = text;
			messages.appendChild(msg);
			messages.scrollTop = messages.scrollHeight;
		}

		function addThinkingMessage() {
			const messages = document.getElementById('messages');
			const msg = document.createElement('div');
			msg.className = 'message thinking';
			msg.id = 'thinkingMsg';
			msg.textContent = '⏳ Consultando a NEXUS Orquestador...';
			messages.appendChild(msg);
			messages.scrollTop = messages.scrollHeight;
		}

		function removeThinkingMessage() {
			const msg = document.getElementById('thinkingMsg');
			if (msg) msg.remove();
		}

		function sendCommand() {
			const input = document.getElementById('commandInput');
			const text = input.value.trim();
			
			if (!text) return;
			if (!selectedAgent) {
				addSystemMessage('❌ Selecciona un agente primero');
				return;
			}

			const messages = document.getElementById('messages');
			const msg = document.createElement('div');
			msg.className = 'message user';
			msg.textContent = \`> \${text}\`;
			messages.appendChild(msg);
			messages.scrollTop = messages.scrollHeight;

			input.value = '';
			addThinkingMessage();
			vscode.postMessage({ command: 'sendCommand', command: text });
		}

		function startAgent() {
			if (!selectedAgent) {
				addSystemMessage('❌ Selecciona un agente primero');
				return;
			}
			addSystemMessage(\`🔗 Conectando \${selectedAgent} a NEXUS...\`);
			vscode.postMessage({ command: 'executeAgent', agent: selectedAgent });
		}

		function stopAgent() {
			if (!selectedAgent) {
				addSystemMessage('❌ Selecciona un agente primero');
				return;
			}
			addSystemMessage(\`⛔ \${selectedAgent} desconectado\`);
		}

		function refreshAgents() {
			addSystemMessage('🔄 Recargando agentes...');
			loadAgents();
		}

		function clearChat() {
			document.getElementById('messages').innerHTML = '';
			addSystemMessage('🧹 Chat limpiado');
		}

		function checkNexus() {
			addSystemMessage('📡 Verificando conexión con NEXUS...');
			vscode.postMessage({ command: 'checkNexus' });
		}

		document.getElementById('commandInput')?.addEventListener('keypress', (e) => {
			if (e.key === 'Enter') sendCommand();
		});

		// Escuchar mensajes desde VSCode
		window.addEventListener('message', (e) => {
			const message = e.data;
			switch (message.command) {
				case 'agentList':
					agents = message.agents;
					renderAgents();
					break;
				case 'agentStarted':
					addSystemMessage(\`🚀 \${message.agent} iniciado\`);
					break;
				case 'thinking':
					removeThinkingMessage();
					addThinkingMessage();
					break;
				case 'commandResponse':
					removeThinkingMessage();
					addAgentMessage(message.response);
					break;
				case 'nexusStatus':
					const badge = document.getElementById('nexusBadge');
					if (message.connected) {
						badge.className = 'nexus-badge connected';
						badge.textContent = '🟢 NEXUS Online';
						addSystemMessage('📡 NEXUS Orquestador: CONECTADO');
					} else {
						badge.className = 'nexus-badge disconnected';
						badge.textContent = '🔴 NEXUS Offline';
						addSystemMessage('⚠️ NEXUS Orquestador: DESCONECTADO');
					}
					break;
				case 'selectAgent':
					selectedAgent = message.agent;
					renderAgents();
					break;
				case 'actionHistoryLoaded':
					renderActionHistory(message.history);
					break;
				case 'actionDeleted':
					addSystemMessage(\`🗑️ Acción \${message.contexto} eliminada.\`);
					loadActionHistory();
					break;
			}
		});

        // --------------------------------------------------------------------------------
        // Lógica de Historial de Acciones
        // --------------------------------------------------------------------------------
        function formatTimeAgo(timestamp_secs) {
            const now = Math.floor(Date.now() / 1000); // segundos
            const diff = now - timestamp_secs;

            if (diff < 60) return \`\${diff}s\`;
            if (diff < 3600) return \`\${Math.floor(diff / 60)}m\`;
            if (diff < 86400) return \`\${Math.floor(diff / 3600)}h\`;
            return \`\${Math.floor(diff / 86400)}d\`;
        }

        function escapeHtml(unsafe) {
            return unsafe
                .replace(/&/g, "&amp;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;")
                .replace(/"/g, "&quot;")
                .replace(/'/g, "&#039;");
        }

        function loadActionHistory() {
            vscode.postMessage({ command: 'getActionHistory' });
        }

        function renderActionHistory(history) {
            const historyList = document.getElementById('actionHistoryList');
            if (!historyList) return;

            if (history.length === 0) {
                historyList.innerHTML = '<div class="empty-state">Sin acciones recientes.</div>';
                return;
            }

            historyList.innerHTML = history.map(item => {
                let displayTitle = item.prompt;
                if (item.acciones && item.acciones.length > 0) {
                    displayTitle = item.acciones[0]; // Muestra la primera acción como título
                } else if (item.respuesta) {
                    displayTitle = item.respuesta.split('\\n')[0]; // Primera línea de la respuesta
                }
                displayTitle = displayTitle.replace(/\\x60.*?\\x60/g, '').trim(); // Eliminar bloques de código markdown
                if (displayTitle.length > 50) {
                    displayTitle = displayTitle.substring(0, 47) + "...";
                }
                if (!displayTitle) displayTitle = "Acción sin descripción";

                return \`<div class="action-item" data-contexto="\${item.contexto}">
                            <span class="action-title">\${escapeHtml(displayTitle)}</span>
                            <span class="action-time">\${formatTimeAgo(item.timestamp_secs)}</span>
                            <button class="delete-action-btn" data-contexto="\${item.contexto}" title="Eliminar">🗑️</button>
                        </div>\`;
            }).join('');

            // Adjuntar event listeners a los botones de eliminar
            historyList.querySelectorAll('.delete-action-btn').forEach(button => {
                button.addEventListener('click', (e) => {
                    e.stopPropagation(); // Previene que el evento se propague al item padre
                    const contexto = parseInt(button.dataset.contexto);
                    if (confirm(\`¿Eliminar acción con contexto \${contexto}?\`)) {
                        vscode.postMessage({ command: 'deleteAction', contexto });
                    }
                });
            });

            // Adjuntar event listeners para cargar la acción en el input del chat
            historyList.querySelectorAll('.action-item').forEach(item => {
                item.addEventListener('click', (e) => {
                    // Si el clic no fue en el botón de eliminar
                    if (!e.target.closest('.delete-action-btn')) {
                        const contexto = parseInt(item.dataset.contexto);
                        // Necesitamos encontrar el item original para obtener el prompt completo
                        const clickedItem = history.find(h => h.contexto === contexto);
                        if (clickedItem) {
                            const commandInput = document.getElementById('commandInput');
                            const messagesDiv = document.getElementById('messages');
                            if (commandInput) {
                                commandInput.value = clickedItem.prompt;
                                commandInput.focus();
                                addSystemMessage(\`Acción cargada: "\${clickedItem.prompt}"\`);
                                if (messagesDiv) messagesDiv.scrollTop = messagesDiv.scrollHeight;
                            }
                        }
                    }
                });
            });
        }

		// Inicializar
		loadAgents();
		checkNexus();
		loadActionHistory();
	</script>
</body>
</html>`;
	}
}
