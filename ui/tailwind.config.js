/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{js,jsx,ts,tsx}'],
  theme: {
    extend: {
      colors: {
        // Paleta "Antigravity" — dark mode nativo, alta densidad
        surface: {
          950: '#0a0a0f',
          900: '#101018',
          850: '#14141e',
          800: '#1a1a26'
        },
        led: {
          on: '#22ff88',   // verde neón
          off: '#3a3a44'   // gris neutro apagado
        }
      }
    }
  },
  plugins: []
}
