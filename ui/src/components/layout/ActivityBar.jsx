import React from 'react';
import { MessageSquare, FolderTree, Image as ImageIcon, Terminal, Settings } from 'lucide-react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

// Helper de clases estilo shadcn
function cn(...inputs) {
  return twMerge(clsx(inputs));
}

export default function ActivityBar({ activePanel, onTogglePanel }) {
  const items = [
    { id: 'chat', icon: MessageSquare, label: 'Chat (IA)' },
    { id: 'explorer', icon: FolderTree, label: 'Explorador de Archivos' },
    { id: 'image', icon: ImageIcon, label: 'Diseño e Imágenes' },
    { id: 'terminal', icon: Terminal, label: 'Terminal' }
  ];

  return (
    <div className="flex w-14 shrink-0 flex-col items-center border-r border-zinc-800/80 bg-[#080d12] py-4 shadow-xl z-20">
      <div className="flex flex-1 flex-col gap-4">
        {items.map((item) => {
          const Icon = item.icon;
          const isActive = activePanel === item.id;
          return (
            <button
              key={item.id}
              title={item.label}
              onClick={() => onTogglePanel(item.id)}
              className={cn(
                "relative flex h-10 w-10 items-center justify-center rounded-xl transition-all duration-200",
                isActive 
                  ? "bg-emerald-500/10 text-emerald-400 shadow-[0_0_10px_rgba(16,185,129,0.2)]" 
                  : "text-zinc-500 hover:bg-zinc-800/50 hover:text-zinc-300"
              )}
            >
              <Icon size={20} strokeWidth={isActive ? 2.5 : 2} />
              {isActive && (
                <span className="absolute left-0 top-1/2 h-5 w-1 -translate-y-1/2 rounded-r-full bg-emerald-500" />
              )}
            </button>
          );
        })}
      </div>

      <div className="flex flex-col gap-4">
        <button
          title="Ajustes"
          className="flex h-10 w-10 items-center justify-center rounded-xl text-zinc-500 transition-all hover:bg-zinc-800/50 hover:text-zinc-300"
        >
          <Settings size={20} />
        </button>
      </div>
    </div>
  );
}
