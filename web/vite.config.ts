/// <reference types="vitest/config" />
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [svelte()],
  // The engine runs in a module worker that imports the wasm-pack output.
  worker: { format: 'es' },
  build: { target: 'es2022' },
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
  },
});
