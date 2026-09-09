import React from 'react'
import ProjectCard from './project-card'

/**
 * ProjectGrid — Grid de ALTA DENSIDAD (Regla: high data density).
 * Permite visualizar docenas de proyectos sin scroll.
 * Compacto: [LED] + [Nombre]. Sin márgenes exagerados ni marcos redundantes.
 */
export default function ProjectGrid({ projects, selectedId, onSelect }) {
  if (projects.length === 0) {
    return (
      <div className="rounded-md border border-dashed border-zinc-800 p-4 text-center text-[12px] text-zinc-600">
        Sin proyectos registrados
      </div>
    )
  }

  return (
    <div className="grid grid-cols-2 gap-1 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
      {projects.map((p) => (
        <ProjectCard
          key={p.id}
          project={p}
          selected={p.id === selectedId}
          onSelect={onSelect}
        />
      ))}
    </div>
  )
}
