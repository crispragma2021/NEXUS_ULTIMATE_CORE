import { clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

/** Combina clases Tailwind de forma segura (patrón shadcn/ui). */
export function cn(...inputs) {
  return twMerge(clsx(inputs))
}
