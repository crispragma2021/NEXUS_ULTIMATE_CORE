import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  base: './', // For Tauri, assets path should be relative
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    port: 5174, // Unique port for nexus-ui frontend dev server
    strictPort: true,
    host: '0.0.0.0',
    allowedHosts: [
      'nexus.autosasistente.app',
      'sovereign.autosasistente.app',
      'localhost',
      '127.0.0.1'
    ],
  },
});