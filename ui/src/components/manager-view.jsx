import React from 'react'
import { Badge } from './ui/badge'
import { Button } from './ui/button'

/**
 * Manager View — panel lateral de supervisión (Regla 4).
 * Muestra la ejecución asíncrona de los agentes: tareas en vuelo,
 * puertos asignados (inmutables) y estado por proyecto.
 */
export default function ManagerView({ projects, onRefresh }) {
  const up = projects.filter((p) => p.status === 'up').length

  return (
    <aside className="flex w-64 shrink-0 flex-col border-l border-zinc-800 bg-surface-900">
      <header className="flex items-center justify-between border-b border-zinc-800 px-3 py-2">
        <span className="text-[11px] font-medium uppercase tracking-wider text-zinc-500">
          Manager
        </span>
        <Button variant="ghost" size="sm" onClick={onRefresh}>
          ↻
        </Button>
      </header>

      <div className="flex-1 space-y-2 overflow-y-auto p-2">
        <div className="flex items-center justify-between rounded-md border border-zinc-800 bg-surface-850 px-2 py-1.5">
          <span className="text-[11px] text-zinc-400">Servicios activos</span>
          <Badge variant="on">{up} / {projects.length}</Badge>
        </div>

        {/* Puertos inmutables por proyecto */}
        {projects.map((p) => (
          <div
            key={p.id}
            className="flex items-center justify-between rounded-md border border-zinc-800 bg-surface-850 px-2 py-1.5"
          >
            <span className="truncate text-[11px] text-zinc-300">{p.name}</span>
            <span className="font-mono text-[10px] text-zinc-500">
              :{p.port ?? '—'}
            </span>
          </div>
        ))}
      </div>
    </aside>
  )
}
