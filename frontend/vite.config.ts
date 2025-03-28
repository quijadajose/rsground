import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin()],
  optimizeDeps: {
    include: ['@codemirror/state', '@codemirror/view'],
  },
  server: {
    port: 3000,
  },
  build: {
    target: 'esnext',
  },
});
