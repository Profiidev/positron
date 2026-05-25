import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { enhancedImages } from '@sveltejs/enhanced-img';

export default defineConfig(() => ({
  define: {
    __version__: JSON.stringify(process.env.npm_package_version)
  },
  plugins: [enhancedImages(), tailwindcss(), sveltekit()],
  server: {
    hmr: {
      port: 5176
    }
  }
}));
