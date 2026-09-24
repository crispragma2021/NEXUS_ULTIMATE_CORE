import React, { useState, useEffect } from 'react';
import AgentCanvas from '../agent-canvas';
import Editor from '@monaco-editor/react';
import { Monitor, Smartphone, RefreshCw, ExternalLink, Search, Globe, FileCode, Code, Layers } from 'lucide-react';

export default function Workspace({ activeTab = 'preview', onChangeTab, activeFilePath, DEMO_NODES, DEMO_EDGES, onOpenCommandPalette }) {
  const [fileContent, setFileContent] = useState("// Modo Código - NEXUS Antigravity IDE\n// Selecciona un archivo en el explorador...");
  const [deviceMode, setDeviceMode] = useState('desktop'); // 'desktop' | 'mobile'

  useEffect(() => {
    if (activeFilePath) {
      fetch(`http://localhost:43210/api/ide/read?path=${encodeURIComponent(activeFilePath)}`)
        .then(res => res.json())
        .then(data => {
          if (data.status === 'ok') {
            setFileContent(data.content);
          } else {
            setFileContent(`// Error leyendo archivo: ${data.message}`);
          }
        })
        .catch(err => setFileContent(`// Fallo al conectar con IDE Daemon: ${err.message}`));
    }
  }, [activeFilePath]);

  const tabs = [
    { id: 'preview', label: 'Vista previa', icon: Globe },
    { id: 'files', label: 'Archivos', icon: FileCode },
    { id: 'code', label: 'Código', icon: Code },
    { id: 'canvas', label: 'Capas / ERD', icon: Layers },
  ];

  return (
    <div className="flex flex-1 min-h-0 h-full w-full flex-col bg-[#030708]">
      {/* Header Bar */}
      <div className="flex h-12 shrink-0 items-center justify-between border-b border-zinc-800/50 bg-[#060b0f] px-4 z-10">
        {/* Branding NEXUS STUDIO */}
        <div className="flex items-center gap-2 pr-4 border-r border-zinc-800/40">
          <div className="h-2.5 w-2.5 rounded-full bg-emerald-500 animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.8)]" />
          <span className="font-bold text-xs tracking-wider uppercase bg-gradient-to-r from-emerald-400 to-teal-200 bg-clip-text text-transparent">
            NEXUS STUDIO
          </span>
        </div>

        {/* Pestañas de Navegación Centrales (Lovable Spec) */}
        <div className="flex h-full items-center gap-1">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.id;
            
            return (
              <button
                key={tab.id}
                onClick={() => onChangeTab(tab.id)}
                className={`relative flex h-full items-center gap-1.5 px-3.5 text-xs font-semibold tracking-wide transition-all ${
                  isActive 
                    ? 'text-emerald-400 bg-emerald-500/10' 
                    : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/30'
                }`}
              >
                <Icon className="w-3.5 h-3.5" />
                <span>{tab.label}</span>
                {isActive && (
                  <span className="absolute bottom-0 left-0 h-[2px] w-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                )}
              </button>
            );
          })}
        </div>

        {/* Acciones & Controls Top Derecha */}
        <div className="flex items-center gap-2">
          {/* Paleta de Comandos Trigger Button */}
          <button 
            onClick={onOpenCommandPalette}
            className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-zinc-800/50 hover:bg-zinc-800 border border-zinc-700/50 text-xs text-zinc-400 hover:text-zinc-200 transition-colors"
            title="Abrir paleta de comandos (Ctrl+K)"
          >
            <Search className="w-3.5 h-3.5 text-emerald-400" />
            <span className="hidden sm:inline font-mono text-[10px] bg-zinc-900 px-1 py-0.5 rounded border border-zinc-700/40">Ctrl K</span>
          </button>

          {/* Viewport Device Controls for Preview */}
          {activeTab === 'preview' && (
            <div className="flex items-center gap-1 px-1 py-0.5 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-400">
              <button
                onClick={() => setDeviceMode('desktop')}
                className={`p-1 rounded hover:text-zinc-200 transition-colors ${deviceMode === 'desktop' ? 'bg-zinc-800 text-emerald-400' : ''}`}
                title="Vista Escritorio"
              >
                <Monitor className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => setDeviceMode('mobile')}
                className={`p-1 rounded hover:text-zinc-200 transition-colors ${deviceMode === 'mobile' ? 'bg-zinc-800 text-emerald-400' : ''}`}
                title="Vista Móvil"
              >
                <Smartphone className="w-3.5 h-3.5" />
              </button>
            </div>
          )}

          <button className="rounded-md bg-zinc-800/60 px-3 py-1.5 text-xs font-medium text-zinc-300 hover:bg-zinc-700/80 transition-colors border border-zinc-700/40">
            Compartir
          </button>
          <button className="rounded-md bg-emerald-500 px-3 py-1.5 text-xs font-bold text-zinc-950 hover:bg-emerald-400 transition-all shadow-[0_0_12px_rgba(16,185,129,0.3)]">
            Publicar
          </button>
        </div>
      </div>

      {/* Main Content Area */}
      <div className="relative flex flex-col flex-1 min-h-0 h-full w-full overflow-hidden bg-[#030708]">
        {activeTab === 'canvas' && (
          <AgentCanvas nodes={DEMO_NODES} edges={DEMO_EDGES} />
        )}
        
        {activeTab === 'code' && (
          <div className="h-full w-full">
            <Editor
              height="100%"
              theme="vs-dark"
              language={activeFilePath?.endsWith('.rs') ? 'rust' : activeFilePath?.endsWith('.js') || activeFilePath?.endsWith('.jsx') ? 'javascript' : activeFilePath?.endsWith('.json') ? 'json' : activeFilePath?.endsWith('.css') ? 'css' : activeFilePath?.endsWith('.html') ? 'html' : 'plaintext'}
              value={fileContent}
              options={{
                fontFamily: '"Fira Code", monospace',
                fontSize: 14,
                minimap: { enabled: true },
                padding: { top: 16 },
                cursorBlinking: "smooth",
                smoothScrolling: true
              }}
            />
          </div>
        )}
        
        {activeTab === 'preview' && (
          <div className="flex h-full w-full flex-col items-center justify-center p-6 bg-[#030708] relative">
            <div className={`flex flex-col items-center justify-center h-full transition-all duration-300 ${deviceMode === 'mobile' ? 'w-[375px] max-h-[750px] border-8 border-zinc-800 rounded-[36px] shadow-2xl overflow-hidden bg-[#080d12]' : 'w-full rounded-xl border border-zinc-800/60 bg-[#060b0f] p-6 shadow-2xl'}`}>
              <div className="flex flex-col items-center justify-center text-center space-y-3 max-w-md">
                <div className="p-3 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shadow-[0_0_15px_rgba(16,185,129,0.2)]">
                  <Globe className="w-8 h-8 animate-pulse" />
                </div>
                <h3 className="text-sm font-bold text-zinc-200 tracking-wide uppercase">Lienzo de Vista Previa Live</h3>
                <p className="text-xs text-zinc-400 leading-relaxed">
                  Aquí se renderiza el resultado visual interactivo de las aplicaciones web y componentes que NEXUS construya para ti.
                </p>
                <div className="pt-2 flex items-center gap-2">
                  <span className="px-2.5 py-1 text-[11px] font-mono text-emerald-400 bg-emerald-950/60 rounded-md border border-emerald-800/40">
                    PROCESO LIVE • ORQUESTADOR NATIVO
                  </span>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'files' && (
          <div className="flex h-full w-full items-center justify-center text-zinc-400 text-xs font-mono p-4">
            <span>[Explorador de Archivos activo en el panel lateral]</span>
          </div>
        )}
      </div>
    </div>
  );
}

