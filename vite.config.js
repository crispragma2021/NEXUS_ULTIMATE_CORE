import { defineConfig } from 'vite'

export default defineConfig({
  root: '.',
  build: {
    outDir: 'dist',
    emptyOutDir: false
  },
  server: {
    port: 5173,
    strictPort: true,
    host: '0.0.0.0', // Escucha en toda la red — acceso LAN habilitado
    allowedHosts: [
      'nexus.autosasistente.app',
      'sovereign.autosasistente.app',
      'localhost',
      '127.0.0.1'
    ],
    watch: {
      ignored: [
        '**/target/**',
        '**/.cargo-cache/**',
        '**/.cargo/**',
        '**/data/**',
        '**/brain/**',
        '**/.git/**',
        '**/tools/**',
        '**/venv/**',
        '**/.venv/**',
        '**/*.py'
      ]
    }
  }
})
