import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { enhancedImages } from '@sveltejs/enhanced-img';
import { svelteTesting } from '@testing-library/svelte/vite';

export default defineConfig(() => ({
  define: {
    __version__: JSON.stringify(process.env.npm_package_version)
  },
  plugins: [enhancedImages(), tailwindcss(), sveltekit(), svelteTesting()],
  resolve: process.env.VITEST
    ? {
        conditions: ['browser']
      }
    : undefined,
  server: {
    hmr: {
      port: 5176
    }
  },
  test: {
    clearMocks: true,
    environment: 'jsdom',
    include: ['src/tests/**/*.{test,spec}.{js,ts}'],
    setupFiles: ['./vitest-setup.ts', './src/mocks/setup.ts']
  }
}));
