import React, { useState } from 'react';
import { 
  Paperclip, Database, Palette, Globe, Layers, FileText, Image as ImageIcon, 
  Music, FileSpreadsheet, Sparkles, Terminal, Cpu, HardDrive
} from 'lucide-react';

export default function PlusPopover({ isOpen, onClose, onSelectOption }) {
  const [activeCategory, setActiveCategory] = useState('attach');

  if (!isOpen) return null;

  const categories = [
    { id: 'attach', label: 'Adjuntar', icon: Paperclip },
    { id: 'database', label: 'BDs & ERD', icon: Database },
    { id: 'design', label: 'Diseño & UI', icon: Palette },
    { id: 'connectors', label: 'Conectores', icon: Globe },
  ];

  return (
    <div 
      className="absolute bottom-16 left-0 z-40 w-80 bg-[#090d12]/95 border border-zinc-800 rounded-xl shadow-2xl overflow-hidden backdrop-blur-xl ring-1 ring-white/10 animate-in slide-in-from-bottom-2 duration-150"
      onClick={(e) => e.stopPropagation()}
    >
      {/* Category Tabs */}
      <div className="flex items-center border-b border-zinc-800/80 bg-zinc-950/60 p-1">
        {categories.map((cat) => {
          const Icon = cat.icon;
          const isActive = activeCategory === cat.id;
          return (
            <button
              key={cat.id}
              onClick={() => setActiveCategory(cat.id)}
              className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-lg text-xs font-medium transition-all ${
                isActive
                  ? 'bg-emerald-500/15 text-emerald-300 font-semibold border border-emerald-500/30'
                  : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/40'
              }`}
            >
              <Icon className="w-3.5 h-3.5" />
              <span>{cat.label}</span>
            </button>
          );
        })}
      </div>

      {/* Category Content */}
      <div className="p-2 space-y-1">
        {activeCategory === 'attach' && (
          <>
            <button 
              onClick={() => { onSelectOption('file-document'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <FileText className="w-4 h-4 text-emerald-400" />
              <div>
                <div className="font-medium">Documento (PDF / Word)</div>
                <div className="text-[10px] text-zinc-500">Cargar documentos para lectura y resumen</div>
              </div>
            </button>

            <button 
              onClick={() => { onSelectOption('file-image'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <ImageIcon className="w-4 h-4 text-teal-400" />
              <div>
                <div className="font-medium">Imágenes & Screenshots</div>
                <div className="text-[10px] text-zinc-500">Analizar imágenes con Vision Bridge</div>
              </div>
            </button>

            <button 
              onClick={() => { onSelectOption('file-excel'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <FileSpreadsheet className="w-4 h-4 text-emerald-500" />
              <div>
                <div className="font-medium">Hoja de Cálculo (Excel / CSV)</div>
                <div className="text-[10px] text-zinc-500">Extraer datos y gráficos numéricos</div>
              </div>
            </button>

            <button 
              onClick={() => { onSelectOption('file-audio'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <Music className="w-4 h-4 text-cyan-400" />
              <div>
                <div className="font-medium">Archivo de Audio</div>
                <div className="text-[10px] text-zinc-500">Transcripción y dictado por voz</div>
              </div>
            </button>
          </>
        )}

        {activeCategory === 'database' && (
          <>
            <button 
              onClick={() => { onSelectOption('db-postgres'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <Database className="w-4 h-4 text-emerald-400" />
              <div>
                <div className="font-medium">PostgreSQL Server</div>
                <div className="text-[10px] text-zinc-500">Conectar base de datos relacional</div>
              </div>
            </button>

            <button 
              onClick={() => { onSelectOption('db-erd'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <Layers className="w-4 h-4 text-teal-400" />
              <div>
                <div className="font-medium">Ver Gráfico ERD / Capas</div>
                <div className="text-[10px] text-zinc-500">Visualizar esquema Mermaid interactivo</div>
              </div>
            </button>

            <button 
              onClick={() => { onSelectOption('db-fts5'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <Cpu className="w-4 h-4 text-emerald-500" />
              <div>
                <div className="font-medium">Hipocampo SQLite FTS5</div>
                <div className="text-[10px] text-zinc-500">Consultar memoria semántica</div>
              </div>
            </button>
          </>
        )}

        {activeCategory === 'design' && (
          <>
            <button 
              onClick={() => { onSelectOption('ui-components'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <Sparkles className="w-4 h-4 text-emerald-400" />
              <div>
                <div className="font-medium">Componentes Lovable UI</div>
                <div className="text-[10px] text-zinc-500">Inyectar botones, modals y cards</div>
              </div>
            </button>

            <button 
              onClick={() => { onSelectOption('ui-svg'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <Palette className="w-4 h-4 text-teal-400" />
              <div>
                <div className="font-medium">Exportación SVG & Monocromo</div>
                <div className="text-[10px] text-zinc-500">Generar variantes visuales de iconos</div>
              </div>
            </button>
          </>
        )}

        {activeCategory === 'connectors' && (
          <>
            <button 
              onClick={() => { onSelectOption('conn-web'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <Globe className="w-4 h-4 text-emerald-400" />
              <div>
                <div className="font-medium">Conector Web Scraping & Search</div>
                <div className="text-[10px] text-zinc-500">Búsqueda directa en internet</div>
              </div>
            </button>

            <button 
              onClick={() => { onSelectOption('conn-terminal'); onClose(); }}
              className="w-full flex items-center gap-2.5 px-3 py-2 text-xs text-zinc-300 hover:bg-zinc-800/60 rounded-lg transition-colors text-left"
            >
              <Terminal className="w-4 h-4 text-teal-400" />
              <div>
                <div className="font-medium">Terminal PTY Daemon</div>
                <div className="text-[10px] text-zinc-500">Conectar a shell local en segundo plano</div>
              </div>
            </button>
          </>
        )}
      </div>
    </div>
  );
}
