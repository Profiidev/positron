import { defineConfig } from '@playwright/test';

export default defineConfig({
  reporter: [
    ['list'],
    ['junit', { outputFile: 'test-results/frontend-e2e.xml' }]
  ],
  testMatch: 'src/tests/e2e/**/*.{test,spec}.{js,ts}',
  webServer: { command: 'npm run build && npm run preview', port: 4173 }
});
