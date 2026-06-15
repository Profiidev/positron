import type { RequestHandler } from 'msw';

/**
 * Default request handlers loaded into the MSW server for every test.
 *
 * Mocks are generated from the OpenAPI schema by the MSW plugin
 * (`src/lib/msw-plugin`). Running `npm run api` emits
 * `src/lib/client/msw.gen.ts` with one typed factory per operation, e.g.
 * `isSetupMswHandler`. Add happy-path defaults here and override per-test with
 * `server.use(...)`:
 *
 *   import { isSetupMswHandler } from '$lib/client/msw.gen';
 *   import { HttpResponse } from 'msw';
 *
 *   export const handlers = [
 *     isSetupMswHandler(() =>
 *       HttpResponse.json({
 *         db_backend: 'sqlite',
 *         is_setup: true,
 *         storage_backend: 'local'
 *       })
 *     )
 *   ];
 *
 * Left empty by default so tests opt into exactly the endpoints they need.
 */
export const handlers: RequestHandler[] = [];
