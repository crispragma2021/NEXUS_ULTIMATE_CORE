import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    port: 42220,
    proxy: {
      '/api': 'http://localhost:42210',
      '/ws': {
        target: 'ws://localhost:42210',
        ws: true,
      },
    },
  },
});
