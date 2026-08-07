import React from 'react'
import { cn } from '@/lib/utils'

/**
 * Badge — shadcn/ui minimalista para estados.
 */
function Badge({ className, variant = 'default', ...props }) {
  const variants = {
    default: 'bg-surface-800 text-zinc-300 border-zinc-700',
    outline: 'border border-zinc-700 text-zinc-400',
    on: 'bg-led-on/10 text-led-on border border-led-on/30',
    off: 'bg-surface-850 text-zinc-500 border border-zinc-800'
  }
  return (
    <span
      className={cn(
        'inline-flex items-center rounded px-1.5 py-0.5 text-[10px] font-medium border',
        variants[variant],
        className
      )}
      {...props}
    />
  )
}

export { Badge }
