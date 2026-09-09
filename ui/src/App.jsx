import React, { useCallback, useEffect, useState } from 'react'
import ProjectGrid from './components/project-grid'
import ManagerView from './components/manager-view'
import ApprovalPanel from './components/approval-panel'
import AgentCanvas from './components/agent-canvas'
import { Button } from './components/ui/button'
import { fetchProjects } from './lib/api'

/**
 * NEXUS · Antigravity UI — Orquestador Agent-First
 *
 * Vista dual (Regla 4):
 *   - Panel principal: grid de alta densidad de proyectos + lienzo de agentes
 *   - Manager View lateral: supervisión de ejecución asíncrona
 *   - Panel de aprobación de cambios (diffs) antes de ejecutar comandos
 *
 * Dark mode nativo · alta densidad · información bajo demanda.
 */

// Estilos de nodos/aristas — DEFINIDOS ANTES de usarlos en los arrays (evita
// el temporal dead zone de const que rompía el montaje de React).
const nodeStyle = {
  background: '#14141e',
  color: '#e4e4e7',
  border: '1px solid #27272a',
  borderRadius: '6px',
  fontSize: '12px',
  padding: '8px 12px'
}
const edgeStyle = { stroke: '#22ff88', strokeWidth: 1.5 }

// Datos de demostración para el lienzo (se alimentan del backend en producción).
const DEMO_NODES = [
  { id: 'daemon', position: { x: 0, y: 100 }, data: { label: '🕸️ Scraper Daemon' }, style: nodeStyle },
  { id: 'rag', position: { x: 220, y: 0 }, data: { label: '🧠 Cerebro RAG' }, style: nodeStyle },
  { id: 'sae', position: { x: 220, y: 200 }, data: { label: '⚡ SAE · IGG' }, style: nodeStyle },
  { id: 'judge', position: { x: 440, y: 100 }, data: { label: '⚖️ Juez' }, style: nodeStyle }
]
const DEMO_EDGES = [
  { id: 'e1', source: 'daemon', target: 'rag', animated: true, style: edgeStyle },
  { id: 'e2', source: 'daemon', target: 'sae', animated: true, style: edgeStyle },
  { id: 'e3', source: 'rag', target: 'judge', animated: true, style: edgeStyle },
  { id: 'e4', source: 'sae', target: 'judge', animated: true, style: edgeStyle }
]

export default function App() {
  const [projects, setProjects] = useState([])
  const [selectedId, setSelectedId] = useState(null)
  const [pendingChanges, setPendingChanges] = useState([])

  const loadProjects = useCallback(async () => {
    try {
      const data = await fetchProjects()
      setProjects(data)
    } catch (e) {
      // Fallback demo: sin backend corriendo, mostrar ejemplo minimalista.
      setProjects([
        { id: 'trader', name: 'Página Trader', status: 'up', port: 8000 },
        { id: 'telegram', name: 'Bot Telegram', status: 'down', port: 8001 },
        { id: 'scraper', name: 'Scraper Daemon', status: 'up', port: 8002 }
      ])
    }
  }, [])

  useEffect(() => {
    loadProjects()
    // Polling cada 5s para el LED (estado en vivo).
    const t = setInterval(loadProjects, 5000)
    return () => clearInterval(t)
  }, [loadProjects])

  const selected = projects.find((p) => p.id === selectedId) || null

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-surface-950 text-zinc-100">
      {/* ═══ Manager View lateral (Regla 4) ═══ */}
      <ManagerView projects={projects} onRefresh={loadProjects} />

      {/* ═══ Panel principal ═══ */}
      <main className="flex flex-1 flex-col overflow-hidden">
        {/* Barra superior: título + acciones */}
        <header className="flex items-center justify-between border-b border-zinc-800 px-3 py-2">
          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold tracking-wide">NEXUS</span>
            <span className="rounded bg-surface-800 px-1.5 py-0.5 text-[10px] text-zinc-400">
              Antigravity UI
            </span>
          </div>
          <div className="flex gap-1.5">
            <Button size="sm" variant="ghost">+ Proyecto</Button>
            <Button size="sm" variant="ghost">Consola</Button>
            <Button size="sm" variant="ghost">Logs</Button>
          </div>
        </header>

        {/* ═══ Grid de ALTA DENSIDAD de proyectos (Regla 3) ═══ */}
        <section className="border-b border-zinc-800 p-2">
          <ProjectGrid
            projects={projects}
            selectedId={selectedId}
            onSelect={(p) => setSelectedId(p.id)}
          />
        </section>

        {/* ═══ Lienzo de agentes (Regla 4) ═══ */}
        <section className="relative flex-1">
          <AgentCanvas nodes={DEMO_NODES} edges={DEMO_EDGES} />

          {/* Panel de aprobación de cambios — bottom right */}
          <div className="absolute bottom-3 right-3 w-72">
            <div className="mb-1 text-[10px] uppercase tracking-wider text-zinc-500">
              Aprobación de cambios
            </div>
            <ApprovalPanel pending={pendingChanges} />
          </div>

          {/* Detalles del proyecto seleccionado — bottom left (bajo demanda) */}
          {selected && (
            <div className="absolute bottom-3 left-3 w-64 rounded-md border border-zinc-800 bg-surface-900 p-2">
              <div className="text-[12px] font-medium">{selected.name}</div>
              <div className="mt-1 font-mono text-[10px] text-zinc-500">
                puerto: :{selected.port ?? '—'} · {selected.status === 'up' ? 'ENCENDIDO' : 'APAGADO'}
              </div>
            </div>
          )}
        </section>
      </main>
    </div>
  )
}
