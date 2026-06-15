import { defineConfig } from '@playwright/test';

export default defineConfig({
  testMatch: 'src/tests/e2e/**/*.{test,spec}.{js,ts}',
  webServer: { command: 'npm run build && npm run preview', port: 4173 }
});
