import React from 'react'
import Led from './led'
import { cn } from '@/lib/utils'

/**
 * ProjectCard — tarjeta ultra-limpia de alta densidad (Regla 3).
 *
 * SOLO dos elementos visibles:
 *   1. Indicador LED (estado)
 *   2. Nombre del proyecto
 *
 * Sin métricas ni textos extra. Los detalles se revelan al seleccionar
 * (principio de información bajo demanda).
 */
export default function ProjectCard({ project, selected, onSelect }) {
  return (
    <button
      onClick={() => onSelect(project)}
      className={cn(
        'group flex items-center gap-2 rounded-md border px-2 py-1.5 text-left transition-colors',
        'border-zinc-800 bg-surface-900 hover:bg-surface-850',
        selected && 'border-led-on/50 bg-surface-850 ring-1 ring-led-on/30'
      )}
    >
      <Led on={project.status === 'up'} />
      <span className="truncate text-[12px] text-zinc-200 group-hover:text-white">
        {project.name}
      </span>
    </button>
  )
}
