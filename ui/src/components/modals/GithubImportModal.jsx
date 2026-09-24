import React, { useState } from 'react';
import { GitBranch, FolderPlus, Download, Loader2, CheckCircle2, AlertCircle, X, ExternalLink } from 'lucide-react';

const GithubIcon = ({ className = "w-5 h-5" }) => (
  <svg className={className} fill="currentColor" viewBox="0 0 24 24">
    <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
  </svg>
);

export default function GithubImportModal({ isOpen, onClose, onImportProject }) {
  const [repoUrl, setRepoUrl] = useState('');
  const [projectName, setProjectName] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState(null);
  const [success, setSuccess] = useState(false);

  if (!isOpen) return null;

  const handleImport = async (e) => {
    e.preventDefault();
    if (!repoUrl.trim()) return;

    setIsLoading(true);
    setError(null);

    try {
      // Extraer nombre del repositorio si no se especificó
      const derivedName = projectName.trim() || repoUrl.split('/').pop()?.replace('.git', '') || 'Proyecto GitHub';

      const res = await fetch('http://localhost:43210/api/consultar', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          prompt: `[DIRECTIVA IMPORTAR REPO GITHUB] Clona e importa el repositorio GitHub '${repoUrl}' en el espacio de trabajo del proyecto como '${derivedName}'.`,
          modelo: 'nexus'
        })
      });

      if (!res.ok) throw new Error(`Error HTTP ${res.status}`);
      const data = await res.json();

      setSuccess(true);
      setTimeout(() => {
        onImportProject({
          id: Date.now(),
          name: derivedName,
          url: repoUrl,
          status: 'up',
          source: 'github'
        });
        setIsLoading(false);
        setSuccess(false);
        setRepoUrl('');
        setProjectName('');
        onClose();
      }, 1200);
    } catch (err) {
      setError(`Error importando repositorio: ${err.message}`);
      setIsLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md p-4 animate-in fade-in duration-150">
      <div 
        className="w-full max-w-lg bg-[#090d12]/95 border border-zinc-800 rounded-xl shadow-2xl overflow-hidden flex flex-col backdrop-blur-xl ring-1 ring-white/10"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-zinc-800/80 bg-zinc-950/60">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-lg bg-zinc-800 text-zinc-100 border border-zinc-700/50">
              <GithubIcon className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-sm font-bold text-zinc-100">Importar desde GitHub</h3>
              <p className="text-[11px] text-zinc-400">Clona un repositorio directamente en tu estudio NEXUS</p>
            </div>
          </div>
          <button 
            onClick={onClose}
            className="p-1 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Formulario */}
        <form onSubmit={handleImport} className="p-5 space-y-4">
          <div>
            <label className="block text-xs font-semibold text-zinc-300 mb-1.5">
              URL del Repositorio GitHub
            </label>
            <div className="relative flex items-center">
              <input
                type="text"
                required
                className="w-full bg-zinc-900/90 border border-zinc-700/80 rounded-lg px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-emerald-500/60 focus:ring-1 focus:ring-emerald-500/20 font-mono"
                placeholder="https://github.com/usuario/mi-proyecto.git"
                value={repoUrl}
                onChange={(e) => setRepoUrl(e.target.value)}
                disabled={isLoading}
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-zinc-300 mb-1.5">
              Nombre del Proyecto en NEXUS (Opcional)
            </label>
            <input
              type="text"
              className="w-full bg-zinc-900/90 border border-zinc-700/80 rounded-lg px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-emerald-500/60 focus:ring-1 focus:ring-emerald-500/20"
              placeholder="Nombre personalizado (ej: Mi App React)"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value)}
              disabled={isLoading}
            />
          </div>

          {error && (
            <div className="flex items-center gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-xs text-red-400">
              <AlertCircle className="w-4 h-4 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {success && (
            <div className="flex items-center gap-2 p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-xs text-emerald-400">
              <CheckCircle2 className="w-4 h-4 shrink-0" />
              <span>¡Repositorio importado exitosamente en NEXUS Studio!</span>
            </div>
          )}

          {/* Actions */}
          <div className="flex items-center justify-end gap-2 pt-2 border-t border-zinc-800/60">
            <button
              type="button"
              onClick={onClose}
              disabled={isLoading}
              className="px-4 py-2 text-xs font-medium text-zinc-400 hover:text-zinc-200 transition-colors"
            >
              Cancelar
            </button>
            <button
              type="submit"
              disabled={isLoading || !repoUrl.trim()}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-500 text-zinc-950 font-bold text-xs hover:bg-emerald-400 transition-all shadow-md disabled:opacity-50"
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  <span>Clonando...</span>
                </>
              ) : (
                <>
                  <Download className="w-4 h-4" />
                  <span>Importar Proyecto</span>
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
