import React, { useState, useEffect } from 'react';
import { 
  Search, Database, Palette, FileCode, Layers, Monitor, Code, 
  Terminal, Sparkles, X, ArrowRight, ShieldCheck, Cpu, Image as ImageIcon
} from 'lucide-react';

export default function CommandPalette({ isOpen, onClose, onSelectAction }) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);

  const actions = [
    { id: 'tab-preview', category: 'Navegación', title: 'Ver Vista Previa (Live UI)', icon: Monitor, tag: 'Vista' },
    { id: 'tab-files', category: 'Navegación', title: 'Explorar Archivos del Proyecto', icon: FileCode, tag: 'Archivos' },
    { id: 'tab-code', category: 'Navegación', title: 'Abrir Editor de Código (Monaco)', icon: Code, tag: 'Código' },
    { id: 'tab-erd', category: 'Navegación', title: 'Ver Capas / Diagrama ERD (Database)', icon: Layers, tag: 'ERD' },
    { id: 'db-postgres', category: 'Conectores', title: 'Conectar Base de Datos PostgreSQL', icon: Database, tag: 'BD' },
    { id: 'db-sqlite', category: 'Conectores', title: 'Inspeccionar Hipocampo SQLite FTS5', icon: Cpu, tag: 'Memoria' },
    { id: 'ui-svg', category: 'Diseño', title: 'Exportar componentes en SVG', icon: Palette, tag: 'UI' },
    { id: 'ui-monocromo', category: 'Diseño', title: 'Generar versión monocromo / Glassmorphism', icon: Sparkles, tag: 'Diseño' },
    { id: 'cmd-terminal', category: 'Herramientas', title: 'Abrir Terminal PTY Integrada', icon: Terminal, tag: 'Sistema' },
  ];

  const filteredActions = actions.filter(action =>
    action.title.toLowerCase().includes(query.toLowerCase()) ||
    action.category.toLowerCase().includes(query.toLowerCase()) ||
    action.tag.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  useEffect(() => {
    const handleKeyDown = (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        isOpen ? onClose() : null;
      }
      if (!isOpen) return;

      if (e.key === 'Escape') {
        onClose();
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1) % (filteredActions.length || 1));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 + filteredActions.length) % (filteredActions.length || 1));
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (filteredActions[selectedIndex]) {
          onSelectAction(filteredActions[selectedIndex].id);
          onClose();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, filteredActions, selectedIndex, onClose, onSelectAction]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-20 bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-150">
      <div 
        className="w-full max-w-2xl bg-[#090d12]/95 border border-zinc-800 rounded-xl shadow-2xl overflow-hidden flex flex-col backdrop-blur-xl ring-1 ring-white/10"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Barra de Búsqueda */}
        <div className="flex items-center px-4 py-3.5 border-b border-zinc-800/80 bg-zinc-900/40">
          <Search className="w-5 h-5 text-emerald-400 mr-3 shrink-0" />
          <input
            type="text"
            className="w-full bg-transparent text-sm text-zinc-100 placeholder-zinc-500 focus:outline-none font-medium"
            placeholder="Escribe un comando o busca una herramienta (ej: 'código', 'erd', 'postgres')..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            autoFocus
          />
          <kbd className="hidden sm:inline-flex items-center gap-1 px-2 py-0.5 text-[10px] font-mono font-semibold text-zinc-400 bg-zinc-800/80 border border-zinc-700/50 rounded">
            ESC
          </kbd>
        </div>

        {/* Lista de Acciones */}
        <div className="max-h-80 overflow-y-auto p-2 space-y-1">
          {filteredActions.length === 0 ? (
            <div className="py-8 text-center text-xs text-zinc-500">
              No se encontraron herramientas que coincidan con "{query}"
            </div>
          ) : (
            filteredActions.map((action, index) => {
              const Icon = action.icon;
              const isSelected = index === selectedIndex;

              return (
                <div
                  key={action.id}
                  onClick={() => {
                    onSelectAction(action.id);
                    onClose();
                  }}
                  onMouseEnter={() => setSelectedIndex(index)}
                  className={`flex items-center justify-between px-3 py-2.5 rounded-lg cursor-pointer transition-all text-xs font-medium ${
                    isSelected
                      ? 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/30'
                      : 'text-zinc-300 hover:bg-zinc-800/50'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div className={`p-1.5 rounded-md ${isSelected ? 'bg-emerald-500/20 text-emerald-400' : 'bg-zinc-800 text-zinc-400'}`}>
                      <Icon className="w-4 h-4" />
                    </div>
                    <span>{action.title}</span>
                  </div>

                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 text-[10px] font-semibold text-zinc-400 bg-zinc-800/60 rounded border border-zinc-700/40 uppercase tracking-wider">
                      {action.tag}
                    </span>
                    {isSelected && <ArrowRight className="w-3.5 h-3.5 text-emerald-400" />}
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-2 border-t border-zinc-800/60 bg-zinc-950/60 text-[11px] text-zinc-500">
          <div className="flex items-center gap-3">
            <span><kbd className="px-1 py-0.5 bg-zinc-800 rounded text-zinc-300">↑↓</kbd> Navegar</span>
            <span><kbd className="px-1 py-0.5 bg-zinc-800 rounded text-zinc-300">↵</kbd> Seleccionar</span>
          </div>
          <span className="flex items-center gap-1 text-emerald-400/80 font-mono">
            <ShieldCheck className="w-3.5 h-3.5" /> NEXUS Control Engine
          </span>
        </div>
      </div>
    </div>
  );
}
