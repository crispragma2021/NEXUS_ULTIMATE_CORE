import React from 'react'
import { Slot } from '@radix-ui/react-slot'
import { cn } from '@/lib/utils'

/**
 * Button — componente shadcn/ui minimalista (dark mode).
 */
const Button = React.forwardRef(
  ({ className, variant = 'default', size = 'default', asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button'
    const variants = {
      default: 'bg-surface-800 hover:bg-surface-850 text-zinc-100 border border-zinc-800',
      ghost: 'hover:bg-surface-850 text-zinc-400 hover:text-zinc-100',
      outline: 'border border-zinc-700 text-zinc-300 hover:bg-surface-850',
      primary: 'bg-led-on/90 hover:bg-led-on text-black font-medium',
      danger: 'bg-red-600/80 hover:bg-red-600 text-white'
    }
    const sizes = {
      default: 'h-8 px-3 text-xs',
      sm: 'h-6 px-2 text-[11px]',
      lg: 'h-10 px-4 text-sm'
    }
    return (
      <Comp
        ref={ref}
        className={cn(
          'inline-flex items-center justify-center gap-1.5 rounded-md transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-led-on disabled:opacity-50 disabled:pointer-events-none',
          variants[variant],
          sizes[size],
          className
        )}
        {...props}
      />
    )
  }
)
Button.displayName = 'Button'

export { Button }
