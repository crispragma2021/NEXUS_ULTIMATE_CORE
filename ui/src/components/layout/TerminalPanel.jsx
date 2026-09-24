import React, { useEffect, useRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { X, Maximize2, Minimize2 } from 'lucide-react';
import '@xterm/xterm/css/xterm.css';

export default function TerminalPanel({ onClose, onToggleMaximize, isMaximized }) {
  const terminalRef = useRef(null);
  
  useEffect(() => {
    if (!terminalRef.current) return;
    
    const term = new Terminal({
      theme: {
        background: '#030708',
        foreground: '#e2e8f0',
        cursor: '#00ff88',
        selectionBackground: 'rgba(0, 255, 136, 0.3)',
      },
      fontFamily: '"Fira Code", monospace',
      fontSize: 13,
      cursorBlink: true,
    });
    
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    
    term.open(terminalRef.current);
    fitAddon.fit();
    
    term.writeln('\x1b[1;32m[NEXUS Core Sovereign]\x1b[0m Conectando a la terminal PTY real...');
    
    // Conexión WebSocket al backend de Rust (Axum)
    const ws = new WebSocket('ws://localhost:43210/api/terminal/ws');
    
    ws.onopen = () => {
      term.writeln('\x1b[1;36m[Sistema]\x1b[0m Conexión WebSocket establecida exitosamente.');
      term.focus();
    };

    ws.onmessage = (event) => {
      try {
        const payload = JSON.parse(event.data);
        if (payload.type === 'output') {
          // Reemplazar \n (sin \r) con \r\n para xterm
          let text = payload.data.replace(/\n/g, '\r\n');
          term.write(text);
        }
      } catch(e) {
        if (typeof event.data === 'string') {
          term.write(event.data);
        }
      }
    };
    
    ws.onerror = (error) => {
      term.writeln('\r\n\x1b[1;31m[Error]\x1b[0m Conexión WebSocket fallida. ¿Está el servidor Rust ejecutándose en el puerto 43210?');
    };

    ws.onclose = () => {
      term.writeln('\r\n\x1b[1;33m[Desconectado]\x1b[0m Terminal cerrada.');
    };

    // Al escribir en la terminal, enviamos al backend como JSON keypress
    const onDataDisposable = term.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'keypress', key: data }));
      }
    });
    
    // Resize observer to auto-fit terminal
    const observer = new ResizeObserver(() => {
      fitAddon.fit();
    });
    observer.observe(terminalRef.current);
    
    return () => {
      observer.disconnect();
      onDataDisposable.dispose();
      ws.close();
      term.dispose();
    };
  }, []);

  return (
    <div className="flex h-full w-full flex-col bg-[#030708] border-t border-zinc-800">
      <div className="flex h-8 shrink-0 items-center justify-between border-b border-zinc-800/50 bg-[#060b0f] px-4">
        <div className="flex items-center gap-4 text-xs font-mono text-zinc-400">
          <span className="text-emerald-400 border-b-2 border-emerald-500 pb-[1px]">Terminal</span>
          <span className="hover:text-zinc-200 cursor-pointer">Logs</span>
          <span className="hover:text-zinc-200 cursor-pointer">Problemas (0)</span>
        </div>
        <div className="flex items-center gap-2 text-zinc-500">
          <button onClick={onToggleMaximize} className="hover:text-zinc-200 transition-colors">
            {isMaximized ? <Minimize2 size={14} /> : <Maximize2 size={14} />}
          </button>
          <button onClick={onClose} className="hover:text-zinc-200 transition-colors">
            <X size={14} />
          </button>
        </div>
      </div>
      <div className="flex-1 relative overflow-hidden bg-[#030708]">
        <div className="absolute inset-2" ref={terminalRef} />
      </div>
    </div>
  );
}
