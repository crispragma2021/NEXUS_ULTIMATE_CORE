import React from 'react'
import { cn } from '@/lib/utils'

/**
 * LED de estado — Regla 3 (Monitor Minimalista).
 *   🟢 verde neón pulsante = ENCENDIDO (healthcheck confirmado)
 *   ⚫ gris sin luz        = APAGADO
 */
export default function Led({ on }) {
  return (
    <span
      className={cn('shrink-0', on ? 'led-on' : 'led-off')}
      aria-label={on ? 'encendido' : 'apagado'}
      title={on ? 'ENCENDIDO' : 'APAGADO'}
    />
  )
}
