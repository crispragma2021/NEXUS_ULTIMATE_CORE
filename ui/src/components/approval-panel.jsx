import React from 'react'
import { Button } from './ui/button'

/**
 * ApprovalPanel — Panel de aprobación de cambios (Regla 4).
 * El agente propone diffs/artefactos; el humano los aprueba o rechaza
 * ANTES de que se ejecuten comandos en el sistema.
 */
export default function ApprovalPanel({ pending }) {
  if (!pending || pending.length === 0) {
    return (
      <div className="flex items-center justify-between rounded-md border border-zinc-800 bg-surface-850 px-2 py-1.5">
        <span className="text-[11px] text-zinc-500">Sin cambios pendientes</span>
      </div>
    )
  }

  return (
    <div className="space-y-1.5">
      {pending.map((item, i) => (
        <div
          key={i}
          className="rounded-md border border-amber-500/30 bg-surface-850 px-2 py-1.5"
        >
          <div className="mb-1 flex items-center justify-between">
            <span className="text-[11px] font-medium text-amber-300">
              {item.type} · {item.target}
            </span>
          </div>
          <pre className="max-h-24 overflow-y-auto rounded bg-black/40 p-1.5 text-[10px] text-zinc-300">
            {item.diff}
          </pre>
          <div className="mt-1.5 flex gap-1.5">
            <Button size="sm" variant="primary">Aprobar</Button>
            <Button size="sm" variant="danger">Rechazar</Button>
          </div>
        </div>
      ))}
    </div>
  )
}
