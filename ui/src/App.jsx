import React, { useState, useRef, useEffect } from 'react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

function cn(...inputs) {
  return twMerge(clsx(inputs));
}
import ActivityBar from './components/layout/ActivityBar';
import Sidebar from './components/layout/Sidebar';
import Workspace from './components/layout/Workspace';
import TerminalPanel from './components/layout/TerminalPanel';
import CommandPalette from './components/modals/CommandPalette';

// Nodos interactivos de arquitectura para el lienzo de capas
const DEMO_NODES = [
  { 
    id: '1', 
    type: 'input', 
    data: { label: '⚡ Extracción AST & Análisis Código' }, 
    position: { x: 250, y: 40 },
    style: { 
      background: '#060b0f', 
      color: '#34d399', 
      border: '1px solid rgba(16, 185, 129, 0.4)', 
      borderRadius: '12px',
      padding: '12px 20px',
      fontWeight: 'bold',
      fontSize: '13px',
      boxShadow: '0 0 20px rgba(16, 185, 129, 0.15)',
      cursor: 'grab'
    }
  },
  { 
    id: '2', 
    data: { label: '🧠 Motor de Inferencia R1 & Orquestación' }, 
    position: { x: 60, y: 180 },
    style: { 
      background: '#060b0f', 
      color: '#38bdf8', 
      border: '1px solid rgba(56, 189, 248, 0.4)', 
      borderRadius: '12px',
      padding: '12px 20px',
      fontWeight: 'bold',
      fontSize: '13px',
      boxShadow: '0 0 20px rgba(56, 189, 248, 0.15)',
      cursor: 'grab'
    }
  },
  { 
    id: '3', 
    data: { label: '🎨 Generación UI & Compilación Lovable' }, 
    position: { x: 440, y: 180 },
    style: { 
      background: '#060b0f', 
      color: '#c084fc', 
      border: '1px solid rgba(192, 132, 252, 0.4)', 
      borderRadius: '12px',
      padding: '12px 20px',
      fontWeight: 'bold',
      fontSize: '13px',
      boxShadow: '0 0 20px rgba(192, 132, 252, 0.15)',
      cursor: 'grab'
    }
  },
  { 
    id: '4', 
    data: { label: '🛢️ Bóveda SQLite & Estado Persistente' }, 
    position: { x: 250, y: 320 },
    style: { 
      background: '#060b0f', 
      color: '#fbbf24', 
      border: '1px solid rgba(251, 191, 36, 0.4)', 
      borderRadius: '12px',
      padding: '12px 20px',
      fontWeight: 'bold',
      fontSize: '13px',
      boxShadow: '0 0 20px rgba(251, 191, 36, 0.15)',
      cursor: 'grab'
    }
  },
];

const DEMO_EDGES = [
  { id: 'e1-2', source: '1', target: '2', animated: true, style: { stroke: '#10b981', strokeWidth: 2 } },
  { id: 'e1-3', source: '1', target: '3', animated: true, style: { stroke: '#10b981', strokeWidth: 2 } },
  { id: 'e2-4', source: '2', target: '4', animated: true, style: { stroke: '#38bdf8', strokeWidth: 2 } },
  { id: 'e3-4', source: '3', target: '4', animated: true, style: { stroke: '#c084fc', strokeWidth: 2 } },
];

function App() {
  const [activeSidebar, setActiveSidebar] = useState('chat');
  const [activeTab, setActiveTab] = useState('preview');
  const [activeFilePath, setActiveFilePath] = useState(null);
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);
  const [terminalHeight, setTerminalHeight] = useState(250);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const isDraggingRef = useRef(false);

  const handleTogglePanel = (panelId) => {
    if (panelId === 'terminal') {
      setIsTerminalOpen(!isTerminalOpen);
    } else {
      // Garantizar que la barra lateral Lovable (w-[420px]) siempre esté activa y visible
      setActiveSidebar(panelId);
      if (panelId === 'chat') {
        setActiveTab('preview');
      }
    }
  };

  const handleFileSelect = (path) => {
    setActiveFilePath(path);
    setActiveTab('code');
    if (typeof window !== 'undefined' && window.innerWidth < 768) {
      setActiveSidebar(null);
    }
  };

  const handleCommandAction = (actionId) => {
    if (actionId === 'tab-preview') setActiveTab('preview');
    if (actionId === 'tab-files') { setActiveSidebar('files'); setActiveTab('files'); }
    if (actionId === 'tab-code') setActiveTab('code');
    if (actionId === 'tab-erd') setActiveTab('canvas');
    if (actionId === 'cmd-terminal') setIsTerminalOpen(true);
  };

  const handleMouseMove = (e) => {
    if (!isDraggingRef.current) return;
    const newHeight = window.innerHeight - e.clientY;
    setTerminalHeight(Math.max(100, Math.min(newHeight, window.innerHeight - 100)));
  };

  const handleMouseUp = () => {
    isDraggingRef.current = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = 'default';
  };

  const startResize = (e) => {
    e.preventDefault();
    isDraggingRef.current = true;
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = 'row-resize';
  };

  // Limpiar event listeners si se desmonta
  useEffect(() => {
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = 'default';
    };
  }, []);

  return (
    <div className="fixed inset-0 flex flex-row overflow-hidden bg-[#030708] text-zinc-100 font-sans">
      <ActivityBar activePanel={activeSidebar} onTogglePanel={handleTogglePanel} />
      
      {/* Sidebar - Flexbox nativo (ancho estándar w-[420px] según AGENTS.md) */}
      {(activeSidebar || 'chat') !== 'terminal' && (
        <div className="w-[420px] max-w-[90vw] shrink-0 border-r border-zinc-800/50 bg-[#060b0f] flex flex-col h-full overflow-hidden shadow-2xl z-10">
          <Sidebar 
            activePanel={activeSidebar || 'chat'} 
            activeFilePath={activeFilePath}
            onFileSelect={handleFileSelect}
          />
        </div>
      )}

      {/* Área de trabajo - Toma el resto del espacio */}
      <div className="flex-1 flex flex-col min-w-0 h-full overflow-hidden relative z-0">
        
        <div className="flex-1 min-h-0 h-full w-full overflow-hidden flex flex-col relative">
          <Workspace 
            activeTab={activeTab} 
            onChangeTab={setActiveTab}
            activeFilePath={activeFilePath}
            DEMO_NODES={DEMO_NODES}
            DEMO_EDGES={DEMO_EDGES}
            onOpenCommandPalette={() => setIsCommandPaletteOpen(true)}
          />
        </div>

        {isTerminalOpen && (
          <div 
            className="shrink-0 flex flex-col bg-[#030708] border-t border-zinc-800/50 relative z-20"
            style={{ height: `${terminalHeight}px` }}
          >
            {/* Drag Handle */}
            <div 
              className="absolute top-0 left-0 right-0 h-1 -mt-[0.5px] cursor-row-resize hover:bg-emerald-500/50 transition-colors z-30"
              onMouseDown={startResize}
            />
            
            <div className="flex-1 min-h-0 overflow-hidden">
              <TerminalPanel 
                onClose={() => setIsTerminalOpen(false)} 
                isMaximized={false}
                onMaximize={() => setTerminalHeight(window.innerHeight - 50)} 
              />
            </div>
          </div>
        )}

      </div>

      {/* Modal de Paleta de Comandos Ctrl+K */}
      <CommandPalette 
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
        onSelectAction={handleCommandAction}
      />
    </div>
  );
}

export default App;

