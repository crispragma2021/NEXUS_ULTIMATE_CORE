import React, { useState } from 'react';
import { 
  FolderPlus, GitBranch, Folder, ChevronRight, Plus, ExternalLink, Sparkles, 
  Code2, CheckCircle2, Clock, Trash2, Search, ArrowUpRight
} from 'lucide-react';
import GithubImportModal from '../modals/GithubImportModal';

const GithubIcon = ({ className = "w-4 h-4" }) => (
  <svg className={className} fill="currentColor" viewBox="0 0 24 24">
    <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
  </svg>
);

export default function ProjectsManager({ onSelectProject, onFileSelect, activeFilePath }) {
  const [isGithubModalOpen, setIsGithubModalOpen] = useState(false);
  
  // Cargar proyectos persistentes o arrancar limpio (sin datos de prueba falsos)
  const [projects, setProjects] = useState(() => {
    try {
      const saved = localStorage.getItem('nexus_user_projects');
      return saved ? JSON.parse(saved) : [];
    } catch (e) {
      return [];
    }
  });

  const [selectedProjectId, setSelectedProjectId] = useState(null);
  const [newProjectName, setNewProjectName] = useState('');
  const [showNewProjectForm, setShowNewProjectForm] = useState(false);

  const saveProjects = (newProjects) => {
    setProjects(newProjects);
    try {
      localStorage.setItem('nexus_user_projects', JSON.stringify(newProjects));
    } catch (e) {
      console.warn("No se pudo guardar proyectos en localStorage", e);
    }
  };

  const handleCreateProject = (e) => {
    e.preventDefault();
    if (!newProjectName.trim()) return;

    const newProj = {
      id: Date.now(),
      name: newProjectName.trim(),
      type: 'Proyecto Web (NEXUS)',
      source: 'local',
      status: 'active',
      updated: 'Justo ahora'
    };

    const updated = [newProj, ...projects];
    saveProjects(updated);
    setSelectedProjectId(newProj.id);
    setNewProjectName('');
    setShowNewProjectForm(false);
  };

  const handleImportGithub = (newProj) => {
    const updated = [newProj, ...projects];
    saveProjects(updated);
    setSelectedProjectId(newProj.id);
  };

  const handleDeleteProject = (e, projId) => {
    e.stopPropagation();
    const updated = projects.filter(p => p.id !== projId);
    saveProjects(updated);
    if (selectedProjectId === projId) {
      setSelectedProjectId(null);
    }
  };

  return (
    <div className="flex flex-col h-full w-full bg-[#060b0f] text-zinc-100 p-3 space-y-4 font-sans">
      {/* Botones de Acción Principales */}
      <div className="grid grid-cols-2 gap-2">
        <button
          onClick={() => setShowNewProjectForm(true)}
          className="flex items-center justify-center gap-2 p-2.5 rounded-xl bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 font-semibold text-xs hover:bg-emerald-500/25 transition-all shadow-sm"
        >
          <Plus className="w-4 h-4" />
          <span>Nuevo Proyecto</span>
        </button>

        <button
          onClick={() => setIsGithubModalOpen(true)}
          className="flex items-center justify-center gap-2 p-2.5 rounded-xl bg-zinc-800/80 border border-zinc-700/60 text-zinc-200 font-semibold text-xs hover:bg-zinc-700 transition-all shadow-sm"
        >
          <GithubIcon className="w-4 h-4 text-emerald-400" />
          <span>Importar GitHub</span>
        </button>
      </div>

      {/* Formulario Crear Proyecto */}
      {showNewProjectForm && (
        <form onSubmit={handleCreateProject} className="p-3 rounded-xl bg-zinc-900/90 border border-zinc-700/80 space-y-2.5 animate-in fade-in duration-150">
          <div className="flex items-center justify-between text-xs font-bold text-zinc-200">
            <span>Crear Proyecto en Blanco</span>
            <button type="button" onClick={() => setShowNewProjectForm(false)} className="text-zinc-500 hover:text-zinc-300">✕</button>
          </div>
          <input
            type="text"
            required
            placeholder="Nombre del proyecto (ej: Mi App Web)"
            value={newProjectName}
            onChange={(e) => setNewProjectName(e.target.value)}
            className="w-full bg-zinc-950 border border-zinc-700/70 rounded-lg px-2.5 py-1.5 text-xs text-zinc-100 focus:outline-none focus:border-emerald-500/60"
            autoFocus
          />
          <div className="flex justify-end gap-2">
            <button
              type="submit"
              className="px-3 py-1 bg-emerald-500 text-zinc-950 font-bold rounded-lg text-xs hover:bg-emerald-400"
            >
              Crear
            </button>
          </div>
        </form>
      )}

      {/* Lista de Proyectos Registrados */}
      <div className="flex-1 overflow-y-auto space-y-2 pr-1">
        <div className="flex items-center justify-between px-1 text-[11px] font-bold uppercase tracking-wider text-zinc-500">
          <span>Tus Proyectos ({projects.length})</span>
          <Sparkles className="w-3.5 h-3.5 text-emerald-400" />
        </div>

        {projects.length === 0 ? (
          <div className="py-12 text-center text-xs text-zinc-500 border border-dashed border-zinc-800 rounded-xl p-4 space-y-2">
            <p className="font-semibold text-zinc-400">Sin proyectos creados</p>
            <p className="text-[11px] text-zinc-500">Haz clic en '+ Nuevo Proyecto' o 'Importar GitHub' para comenzar.</p>
          </div>
        ) : (
          projects.map((proj) => {
            const isSelected = proj.id === selectedProjectId;
            return (
              <div
                key={proj.id}
                onClick={() => {
                  setSelectedProjectId(proj.id);
                  if (onSelectProject) onSelectProject(proj);
                }}
                className={`group flex items-center justify-between p-3 rounded-xl border cursor-pointer transition-all ${
                  isSelected
                    ? 'bg-emerald-500/10 border-emerald-500/40 text-zinc-100 shadow-md ring-1 ring-emerald-500/20'
                    : 'bg-zinc-900/50 border-zinc-800/80 text-zinc-300 hover:bg-zinc-800/50 hover:border-zinc-700'
                }`}
              >
                <div className="flex items-center gap-3 min-w-0">
                  <div className={`p-2 rounded-lg shrink-0 ${proj.source === 'github' ? 'bg-purple-500/20 text-purple-400' : 'bg-emerald-500/20 text-emerald-400'}`}>
                    {proj.source === 'github' ? <GithubIcon className="w-4 h-4" /> : <Folder className="w-4 h-4" />}
                  </div>
                  <div className="min-w-0">
                    <div className="font-semibold text-xs flex items-center gap-1.5 truncate">
                      <span className="truncate">{proj.name}</span>
                      {proj.source === 'github' && (
                        <span className="px-1.5 py-0.2 text-[9px] bg-purple-950/60 text-purple-300 rounded border border-purple-800/40 shrink-0">GitHub</span>
                      )}
                    </div>
                    <div className="text-[10px] text-zinc-500 flex items-center gap-2 mt-0.5">
                      <span>{proj.type}</span>
                      <span>•</span>
                      <span className="flex items-center gap-1 text-zinc-400"><Clock className="w-2.5 h-2.5" /> {proj.updated}</span>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-1">
                  <button
                    onClick={(e) => handleDeleteProject(e, proj.id)}
                    className="opacity-0 group-hover:opacity-100 p-1 text-zinc-500 hover:text-red-400 hover:bg-red-500/10 rounded transition-all"
                    title="Eliminar proyecto"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                  <ChevronRight className={`w-4 h-4 transition-transform ${isSelected ? 'text-emerald-400 translate-x-0.5' : 'text-zinc-600'}`} />
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Modal GitHub */}
      <GithubImportModal
        isOpen={isGithubModalOpen}
        onClose={() => setIsGithubModalOpen(false)}
        onImportProject={handleImportGithub}
      />
    </div>
  );
}
