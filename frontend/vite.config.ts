import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { enhancedImages } from '@sveltejs/enhanced-img';

// https://vitejs.dev/config/
// @ts-ignore
export default defineConfig(() => ({
  plugins: [enhancedImages(), tailwindcss(), sveltekit()],
  server: {
    hmr: {
      port: 5174
    }
  }
}));
