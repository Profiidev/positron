import { afterAll, afterEach, beforeAll } from 'vitest';
import { client } from '$mocks/msw-runtime';
import { client as appClient } from '$lib/client/client.gen';
import { server } from './server';

// In the browser the client uses relative `/api/...` URLs, but Node's `fetch`
// (used under jsdom) rejects relative URLs, so requests must be absolute. Pin a
// Test origin: the generated MSW handlers read this same `baseUrl`, so the
// Request URL and the handler pattern stay aligned.
const TEST_BASE_URL = 'http://localhost';

// Start the MSW server once for the whole run. `onUnhandledRequest: 'error'`
// Makes any un-mocked `/api/...` call fail the test loudly; non-API requests
// (e.g. assets) are let through.
beforeAll(() => {
  // The MSW handlers (msw.gen.ts) read their `baseUrl` from the mock client,
  // while the app's `load` functions issue requests through the generated SDK,
  // which uses the `client.gen` client. Pin both to the same origin so request
  // URLs and handler patterns stay aligned.
  client.setConfig({ ...client.getConfig(), baseUrl: TEST_BASE_URL });
  appClient.setConfig({ ...appClient.getConfig(), baseUrl: TEST_BASE_URL });
  server.listen({
    onUnhandledRequest: (request, print) => {
      if (new URL(request.url).pathname.startsWith('/api/')) {
        print.error();
      }
    }
  });
});

// Reset any per-test handler overrides so tests stay isolated.
afterEach(() => server.resetHandlers());

afterAll(() => server.close());
