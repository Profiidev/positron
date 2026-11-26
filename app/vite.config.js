import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';
import { enhancedImages } from '@sveltejs/enhanced-img';
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  define: {
    __desktop__: process.env.DESKTOP_APP_TARGET === 'true',
    __version__: JSON.stringify(process.env.npm_package_version)
  },
  plugins: [
    enhancedImages(),
    tailwindcss(),
    sveltekit({
      // Tauri doesn't have a Node.js server to do proper SSR
      // So we use adapter-static with a fallback to index.html to put the site in SPA mode
      // See: https://svelte.dev/docs/kit/single-page-apps
      // See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
      adapter: adapter({ fallback: 'index.html' }),
      preprocess: vitePreprocess()
    })
  ],
  resolve: process.env.VITEST
    ? {
        conditions: ['browser']
      }
    : undefined,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    hmr: host
      ? {
          host,
          port: 1421,
          protocol: 'ws'
        }
      : undefined,
    host: host || false,
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**']
    }
  },
  test: {
    clearMocks: true,
    environment: 'jsdom',
    include: ['src/tests/unit/**/*.{test,spec}.{js,ts}'],
    setupFiles: [
      './src/tests/setup/vitest-setup.ts',
      './src/tests/setup/vitest-setup-sveltekit.ts'
    ]
  }
}));
