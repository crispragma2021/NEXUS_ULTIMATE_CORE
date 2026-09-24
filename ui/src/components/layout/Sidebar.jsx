import React, { useState, useRef, useEffect } from 'react';
import { 
  Bot, ChevronDown, Sparkles, Send, Loader2, Folder, File, ChevronRight,
  Paperclip, FileText, Image as ImageIcon, Music, FileSpreadsheet, X, Mic, MicOff,
  Search, Database, Palette, Plus, Wand2, Compass, Layers, GitBranch
} from 'lucide-react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import ProjectsManager from './ProjectsManager';

function cn(...inputs) {
  return twMerge(clsx(inputs));
}

function FileNode({ node, level = 0, onFileSelect, activeFilePath }) {
  const [isOpen, setIsOpen] = useState(false);
  const [children, setChildren] = useState(node.children || null);
  const [loading, setLoading] = useState(false);
  const isActive = activeFilePath === node.path;

  const toggleOpen = async () => {
    if (!isOpen && !children) {
      setLoading(true);
      try {
        const res = await fetch(`http://localhost:43210/api/fs/list?path=${encodeURIComponent(node.path)}`);
        if (res.ok) {
          const data = await res.json();
          const mappedChildren = data.files.map(f => ({
            ...f,
            path: node.path === '.' ? `./${f.name}` : `${node.path}/${f.name}`
          }));
          setChildren(mappedChildren);
        }
      } catch (e) {
        console.error(e);
      }
      setLoading(false);
    }
    setIsOpen(!isOpen);
  };

  return (
    <div className="flex flex-col">
      <div 
        className={cn(
          "flex items-center gap-1.5 py-1 px-2 hover:bg-zinc-800/50 cursor-pointer text-sm text-zinc-300 rounded-md transition-colors",
          level === 0 && "font-medium",
          isActive && "bg-zinc-800/80 text-emerald-400 font-semibold"
        )}
        style={{ paddingLeft: `${level * 12 + 8}px` }}
        onClick={() => {
          if (node.is_dir) {
            toggleOpen();
          } else if (onFileSelect) {
            onFileSelect(node.path);
          }
        }}
      >
        {node.is_dir ? (
          <>
            {loading ? (
                <Loader2 size={14} className="text-zinc-500 animate-spin" />
            ) : (
                <ChevronRight size={14} className={cn("text-zinc-500 transition-transform", isOpen && "rotate-90")} />
            )}
            <Folder size={14} className="text-emerald-500/80" />
          </>
        ) : (
          <>
            <div className="w-3.5" />
            <File size={14} className="text-zinc-500" />
          </>
        )}
        <span className="truncate">{node.name}</span>
      </div>
      {node.is_dir && isOpen && children && (
        <div className="flex flex-col">
          {children.map((child, i) => (
            <FileNode key={child.path || i} node={child} level={level + 1} onFileSelect={onFileSelect} activeFilePath={activeFilePath} />
          ))}
        </div>
      )}
    </div>
  );
}

export default function Sidebar({ activePanel, activeFilePath, onFileSelect }) {
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [fileTree, setFileTree] = useState(null);
  const [attachments, setAttachments] = useState([]);
  const [showPlusMenu, setShowPlusMenu] = useState(false);
  const [mode, setMode] = useState('crear'); // 'crear' | 'plan'
  const [isRecording, setIsRecording] = useState(false);
  const [showModeDropdown, setShowModeDropdown] = useState(false);
  const [isWebSearch, setIsWebSearch] = useState(false);

  const fileInputRef = useRef(null);
  const scrollRef = useRef(null);
  const recognitionRef = useRef(null);

  const [messages, setMessages] = useState([
    {
      role: 'system',
      content: 'Hola, soy Antigravity. He sincronizado la arquitectura NEXUS Sovereign. Puedes enviarme mensajes, fotos, audios, y documentos PDF, Word o Excel.'
    }
  ]);

  useEffect(() => {
    if (scrollRef.current && activePanel === 'chat') {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, activePanel]);

  // Atajo de teclado Alt + P para alternar entre modo 'Crear' y modo 'Plan'
  useEffect(() => {
    const handleKeyDown = (e) => {
      if (e.altKey && (e.key === 'p' || e.key === 'P')) {
        e.preventDefault();
        setMode(prev => prev === 'crear' ? 'plan' : 'crear');
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // IDE Daemon: Fetch Tree
  useEffect(() => {
    if (activePanel === 'explorer') {
      const fetchTree = async () => {
        try {
          const res = await fetch('http://localhost:43210/api/fs/list?path=.');
          if (res.ok) {
            const data = await res.json();
            const rootChildren = data.files.map(f => ({
                ...f,
                path: `./${f.name}`
            }));
            
            setFileTree({
                name: 'NEXUS_ULTIMATE_CORE',
                is_dir: true,
                path: '.',
                children: rootChildren
            });
          }
        } catch (e) {
          console.error("No se pudo obtener el árbol del IDE Daemon:", e);
        }
      };
      fetchTree();
    }
  }, [activePanel]);

  if (!activePanel || activePanel === 'terminal') {
    return null; 
  }

  // Carga de Archivos (PDF, Word, Excel, Fotos, Audios, TXT, Code)
  const handleFileUpload = (e) => {
    const files = Array.from(e.target.files || []);
    files.forEach((file) => {
      const reader = new FileReader();
      const ext = file.name.split('.').pop()?.toLowerCase() || '';
      const isImg = file.type.startsWith('image/');
      const isAudio = file.type.startsWith('audio/');
      const isPdf = file.type === 'application/pdf' || ext === 'pdf';
      const isDoc = ext === 'doc' || ext === 'docx';
      const isExcel = ext === 'xls' || ext === 'xlsx' || ext === 'csv';

      let category = 'file';
      if (isImg) category = 'image';
      else if (isAudio) category = 'audio';
      else if (isPdf) category = 'pdf';
      else if (isDoc) category = 'word';
      else if (isExcel) category = 'excel';

      if (isImg || isAudio) {
        reader.readAsDataURL(file);
        reader.onload = () => {
          setAttachments(prev => [...prev, {
            id: Date.now() + Math.random(),
            name: file.name,
            size: (file.size / 1024).toFixed(1) + ' KB',
            category,
            dataUrl: reader.result
          }]);
        };
      } else {
        reader.readAsText(file);
        reader.onload = () => {
          setAttachments(prev => [...prev, {
            id: Date.now() + Math.random(),
            name: file.name,
            size: (file.size / 1024).toFixed(1) + ' KB',
            category,
            content: reader.result
          }]);
        };
      }
    });
    setShowPlusMenu(false);
    if (fileInputRef.current) fileInputRef.current.value = '';
  };

  const removeAttachment = (id) => {
    setAttachments(prev => prev.filter(a => a.id !== id));
  };

  // Dictado por voz 🎤 (SpeechRecognition API)
  const toggleVoiceDictation = () => {
    if (isRecording) {
      if (recognitionRef.current) {
        recognitionRef.current.stop();
      }
      setIsRecording(false);
    } else {
      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
      if (!SpeechRecognition) {
        alert("El dictado por voz no está soportado en este navegador.");
        return;
      }
      const recognition = new SpeechRecognition();
      recognition.lang = 'es-ES';
      recognition.continuous = true;
      recognition.interimResults = true;

      recognition.onresult = (event) => {
        let transcript = '';
        for (let i = event.resultIndex; i < event.results.length; i++) {
          transcript += event.results[i][0].transcript;
        }
        setInput(prev => prev ? `${prev} ${transcript}` : transcript);
      };

      recognition.onerror = (err) => {
        console.error("Error en reconocimiento de voz:", err);
        setIsRecording(false);
      };

      recognition.onend = () => {
        setIsRecording(false);
      };

      recognition.start();
      recognitionRef.current = recognition;
      setIsRecording(true);
    }
  };

  const handleSend = async () => {
    if ((!input.trim() && attachments.length === 0) || isLoading) return;
    
    const userMsg = input.trim();
    const currentAttachments = [...attachments];
    const currentMode = mode;

    setInput('');
    setAttachments([]);

    setMessages(prev => [...prev, { 
      role: 'user', 
      content: userMsg,
      attachments: currentAttachments,
      mode: currentMode
    }]);
    setIsLoading(true);

    let contextPayload = userMsg;
    if (currentMode === 'plan') {
      contextPayload = `[MODO PLANIFICACIÓN (PLAN)]\n${contextPayload}`;
    }

    if (currentAttachments.length > 0) {
      const attInfo = currentAttachments.map(a => 
        `[Adjunto: ${a.name} | Tipo: ${a.category} | Tamaño: ${a.size}]${a.content ? `\nContenido:\n${a.content.slice(0, 4000)}` : ''}`
      ).join('\n\n');
      contextPayload = `${attInfo}\n\nInstrucción del Usuario: ${contextPayload}`;
    }

    // MAGIA AST: Inyección de contexto de archivo activo
    if (activeFilePath) {
      try {
        const astRes = await fetch(`http://localhost:43210/api/ide/symbols?path=${encodeURIComponent(activeFilePath)}`);
        if (astRes.ok) {
          const astData = await astRes.json();
          if (astData.status === 'ok' && astData.symbols.length > 0) {
            contextPayload = `[Contexto del archivo actual: ${activeFilePath}]\n[Estructura AST: ${JSON.stringify(astData.symbols)}]\n\n${contextPayload}`;
          }
        }
      } catch (e) {
        console.warn("Fallo al inyectar contexto AST", e);
      }
    }

    try {
      const res = await fetch('http://localhost:43210/api/consultar', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          prompt: contextPayload,
          modelo: 'nexus'
        })
      });

      if (!res.ok) throw new Error(`HTTP Error: ${res.status}`);

      const data = await res.json();
      setMessages(prev => [...prev, { role: 'assistant', content: data.respuesta || "No hubo respuesta válida." }]);
    } catch (err) {
      setMessages(prev => [...prev, { role: 'assistant', content: `[Error de Conexión] No pude conectar con el núcleo Rust en localhost:43210. Detalle: ${err.message}` }]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const renderFileIcon = (category) => {
    switch (category) {
      case 'pdf': return <FileText size={16} className="text-red-400" />;
      case 'word': return <FileText size={16} className="text-blue-400" />;
      case 'excel': return <FileSpreadsheet size={16} className="text-emerald-400" />;
      case 'image': return <ImageIcon size={16} className="text-purple-400" />;
      case 'audio': return <Music size={16} className="text-amber-400" />;
      default: return <File size={16} className="text-zinc-400" />;
    }
  };

  return (
    <div className="flex h-full w-full flex-col bg-[#060b0f]/90 backdrop-blur-xl border-r border-zinc-800/50">
      {/* Header */}
      <div className="flex h-14 shrink-0 items-center justify-between border-b border-zinc-800/50 px-4">
        <div className="flex items-center gap-2">
          <Bot size={18} className="text-emerald-400" />
          <span className="font-display font-semibold tracking-wide text-zinc-100 uppercase text-xs">
            {activePanel === 'chat' ? 'Blooming Icon Studio · NEXUS Assistant' : 
             activePanel === 'explorer' ? 'Proyectos & GitHub · Studio' : 
             activePanel === 'image' ? 'NEXUS Studio' : activePanel}
          </span>
        </div>
      </div>

      {/* Input Oculto de Archivos */}
      <input 
        type="file" 
        ref={fileInputRef} 
        onChange={handleFileUpload} 
        multiple 
        className="hidden" 
        accept=".pdf,.doc,.docx,.xls,.xlsx,.csv,.png,.jpg,.jpeg,.gif,.webp,.mp3,.wav,.ogg,.txt,.json,.rs,.js,.jsx,.py" 
      />

      {/* Content Area */}
      <div className="flex-1 overflow-y-auto p-2" ref={scrollRef}>
        {activePanel === 'chat' && (
          <div className="flex flex-col gap-4 p-2">
            {messages.map((msg, idx) => (
              <div 
                key={idx} 
                className={cn(
                  "rounded-xl border p-4 shadow-sm transition-all",
                  msg.role === 'system' ? "border-zinc-800 bg-surface-900/50" :
                  msg.role === 'user' ? "border-zinc-700 bg-zinc-800/30 ml-8" :
                  "border-emerald-900/30 bg-emerald-900/10 mr-8"
                )}
              >
                {msg.role === 'system' && (
                  <div className="mb-2 flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-emerald-400">
                    <Sparkles size={14} />
                    Sistema Inicializado
                  </div>
                )}
                {msg.role === 'user' && (
                  <div className="mb-2 flex items-center justify-between text-xs font-bold uppercase tracking-wider text-zinc-400">
                    <span>Tú</span>
                    {msg.mode === 'plan' && (
                      <span className="rounded bg-amber-500/20 px-1.5 py-0.5 text-[10px] text-amber-400 border border-amber-500/30">
                        Modo Plan
                      </span>
                    )}
                  </div>
                )}
                {msg.role === 'assistant' && (
                  <div className="mb-2 flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-emerald-400">
                    <Bot size={14} />
                    NEXUS Core
                  </div>
                )}

                {/* Adjuntos en Mensaje de Usuario */}
                {msg.attachments && msg.attachments.length > 0 && (
                  <div className="mb-3 flex flex-wrap gap-2">
                    {msg.attachments.map((att) => (
                      <div key={att.id} className="flex flex-col rounded-lg border border-zinc-700 bg-zinc-900/80 p-2 text-xs">
                        <div className="flex items-center gap-2">
                          {renderFileIcon(att.category)}
                          <span className="font-medium text-zinc-200 truncate max-w-[160px]">{att.name}</span>
                          <span className="text-[10px] text-zinc-400">({att.size})</span>
                        </div>
                        {att.category === 'image' && att.dataUrl && (
                          <img src={att.dataUrl} alt={att.name} className="mt-2 max-h-36 rounded-md object-contain border border-zinc-800" />
                        )}
                        {att.category === 'audio' && att.dataUrl && (
                          <audio src={att.dataUrl} controls className="mt-2 h-7 w-48" />
                        )}
                      </div>
                    ))}
                  </div>
                )}

                {msg.content && (
                  <p className="text-sm text-zinc-300 whitespace-pre-wrap leading-relaxed">
                    {msg.content}
                  </p>
                )}

                {msg.image && (
                  <div className="mt-3 overflow-hidden rounded-lg border border-zinc-700/80 bg-zinc-900/60 shadow-lg">
                    <img 
                      src={`data:image/png;base64,${msg.image}`} 
                      alt="Visión Nativa" 
                      className="w-full h-auto object-contain max-h-72 hover:scale-[1.02] transition-transform duration-200 cursor-pointer"
                      onClick={() => {
                        const win = window.open();
                        if (win) {
                          win.document.write(`<img src="data:image/png;base64,${msg.image}" style="max-width:100%;" />`);
                        }
                      }}
                    />
                  </div>
                )}
              </div>
            ))}
            
            {isLoading && (
              <div className="flex items-center gap-2 text-emerald-500 text-sm italic mr-8 p-4 rounded-xl border border-emerald-900/30 bg-emerald-900/10">
                <Loader2 size={16} className="animate-spin" />
                Sincronizando con el núcleo...
              </div>
            )}
            
            {messages.length === 1 && (
              <div className="flex flex-wrap gap-2 mt-4">
                {['Descarga en SVG', 'Crea versión monocromo', 'Genera variantes de color', 'Exporta en tamaños', 'Agrega menú'].map((pill) => (
                  <button 
                    key={pill}
                    onClick={() => { setInput(pill); }}
                    className="rounded-full border border-zinc-700 bg-zinc-800/50 px-3 py-1 text-xs font-medium text-zinc-300 transition-colors hover:border-emerald-500/50 hover:bg-emerald-500/10 hover:text-emerald-400"
                  >
                    {pill}
                  </button>
                ))}
              </div>
            )}
          </div>
        )}

        {activePanel === 'explorer' && (
          <ProjectsManager
            onFileSelect={onFileSelect}
            activeFilePath={activeFilePath}
          />
        )}
      </div>

      {/* Prompt Bar Lovable Layout */}
      {activePanel === 'chat' && (
        <div className="shrink-0 border-t border-zinc-800/50 p-3">
          {/* Previsualización de Adjuntos y Búsqueda Web */}
          {(attachments.length > 0 || isWebSearch) && (
            <div className="mb-2 flex flex-wrap gap-2 rounded-xl border border-zinc-800 bg-zinc-950/60 p-2">
              {isWebSearch && (
                <div className="flex items-center gap-1.5 rounded-lg border border-blue-500/40 bg-blue-500/10 px-2 py-1 text-xs text-blue-400 font-medium">
                  <Search size={13} />
                  <span>Búsqueda Web Activa</span>
                  <button 
                    onClick={() => setIsWebSearch(false)}
                    className="ml-1 text-blue-400 hover:text-red-400"
                  >
                    <X size={12} />
                  </button>
                </div>
              )}
              {attachments.map((att) => (
                <div key={att.id} className="flex items-center gap-1.5 rounded-lg border border-zinc-700 bg-zinc-800/70 px-2 py-1 text-xs text-zinc-200">
                  {renderFileIcon(att.category)}
                  <span className="truncate max-w-[120px]">{att.name}</span>
                  <button 
                    onClick={() => removeAttachment(att.id)}
                    className="ml-1 text-zinc-400 hover:text-red-400"
                  >
                    <X size={12} />
                  </button>
                </div>
              ))}
            </div>
          )}

          <div className="relative flex items-center rounded-xl border border-zinc-700 bg-zinc-900/80 p-1 focus-within:border-emerald-500/50 focus-within:ring-1 focus-within:ring-emerald-500/20">
            {/* Popover del Menú + (Lovable Specification) */}
            <div className="relative shrink-0">
              <button 
                onClick={() => setShowPlusMenu(!showPlusMenu)}
                title="Añadir adjuntos, búsquedas o conectores"
                className="flex h-8 w-8 items-center justify-center rounded-lg text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200 transition-colors"
              >
                <Plus size={16} />
              </button>

              {showPlusMenu && (
                <div className="absolute bottom-10 left-0 z-50 w-64 rounded-xl border border-zinc-700 bg-zinc-900/95 p-2 shadow-2xl backdrop-blur-md">
                  <div className="mb-1 px-2 py-1 text-[11px] font-bold uppercase tracking-wider text-zinc-500">
                    Opciones de NEXUS
                  </div>
                  <button 
                    onClick={() => {
                      setShowPlusMenu(false);
                      fileInputRef.current?.click();
                    }}
                    className="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-xs font-medium text-zinc-200 hover:bg-zinc-800/80 hover:text-emerald-400 transition-colors"
                  >
                    <Paperclip size={14} className="text-emerald-400" />
                    <span>Adjuntar (PDF, Word, Excel, Fotos, Audio)</span>
                  </button>
                  <button 
                    onClick={() => { 
                      setIsWebSearch(!isWebSearch); 
                      setShowPlusMenu(false); 
                    }}
                    className="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-xs font-medium text-zinc-200 hover:bg-zinc-800/80 hover:text-blue-400 transition-colors"
                  >
                    <Search size={14} className="text-blue-400" />
                    <span>{isWebSearch ? 'Desactivar Búsqueda Web' : 'Activar Búsqueda Web'}</span>
                  </button>
                </div>
              )}
            </div>

            {/* Selector de Modo (Crear ˅ vs Plan) */}
            <div className="relative shrink-0 border-r border-zinc-800 pr-1 mr-1">
              <button
                onClick={() => setShowModeDropdown(!showModeDropdown)}
                className="flex items-center gap-1 rounded-md px-2 py-1 text-xs font-semibold text-zinc-300 hover:bg-zinc-800 hover:text-emerald-400 transition-colors"
              >
                <span>{mode === 'crear' ? 'Crear' : 'Plan'}</span>
                <ChevronDown size={12} />
              </button>

              {showModeDropdown && (
                <div className="absolute bottom-9 left-0 z-50 w-28 rounded-lg border border-zinc-700 bg-zinc-900 p-1 shadow-xl">
                  <button
                    onClick={() => { setMode('crear'); setShowModeDropdown(false); }}
                    className={cn(
                      "w-full text-left px-2 py-1 text-xs rounded-md transition-colors",
                      mode === 'crear' ? "bg-emerald-500/20 text-emerald-400 font-semibold" : "text-zinc-300 hover:bg-zinc-800"
                    )}
                  >
                    Crear
                  </button>
                  <button
                    onClick={() => { setMode('plan'); setShowModeDropdown(false); }}
                    className={cn(
                      "w-full text-left px-2 py-1 text-xs rounded-md transition-colors",
                      mode === 'plan' ? "bg-amber-500/20 text-amber-400 font-semibold" : "text-zinc-300 hover:bg-zinc-800"
                    )}
                  >
                    Plan (Alt+P)
                  </button>
                </div>
              )}
            </div>

            {/* Input de Texto */}
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              disabled={isLoading}
              placeholder={isRecording ? "Escuchando dictado por voz..." : "Habla con NEXUS..."}
              className="flex-1 min-w-0 bg-transparent px-2 py-2 text-sm text-zinc-100 outline-none placeholder:text-zinc-500 font-mono disabled:opacity-50"
            />

            {/* Dictado por Voz 🎤 */}
            <button
              onClick={toggleVoiceDictation}
              title="Dictado por voz"
              className={cn(
                "shrink-0 flex h-8 w-8 items-center justify-center rounded-lg transition-colors mr-1",
                isRecording ? "bg-red-500/20 text-red-400 animate-pulse" : "text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
              )}
            >
              {isRecording ? <MicOff size={14} /> : <Mic size={14} />}
            </button>

            {/* Botón Enviar */}
            <button 
              onClick={handleSend}
              disabled={isLoading || (!input.trim() && attachments.length === 0)}
              className="shrink-0 flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30 disabled:opacity-50 disabled:hover:bg-emerald-500/20 transition-colors"
            >
              <Send size={14} />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
