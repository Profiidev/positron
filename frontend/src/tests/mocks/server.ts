import { setupServer } from 'msw/node';
import { handlers } from './handlers';

/**
 * MSW server for tests. Unit/component tests run in Node (jsdom), so the
 * Node interceptor is used rather than a service worker.
 *
 * Import this in a test to override handlers for a single case:
 *
 *   import { server } from '$mocks/server';
 *   server.use(http.get('*\/api/setup', () => new HttpResponse(null, { status: 500 })));
 */
export const server = setupServer(...handlers);
