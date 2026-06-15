import { describe, expect, it } from 'vitest';
import { HttpResponse } from 'msw';
import { isSetup } from '$lib/client';
import { isSetupMswHandler } from '$lib/client/msw.gen';
import { server } from '$mocks/server';

// End-to-end check of the generated MSW handler factories: register a mock for
// An operation, call the real generated SDK function, assert the mocked data
// Comes back. The MSW server lifecycle is wired in `./setup.ts`.
describe('generated msw handlers', () => {
  it('mocks an operation via its generated factory', async () => {
    server.use(
      isSetupMswHandler(() =>
        HttpResponse.json({
          db_backend: 'sqlite',
          is_setup: true,
          storage_backend: 'local'
        })
      )
    );

    const { data } = await isSetup();

    expect(data).toEqual({
      db_backend: 'sqlite',
      is_setup: true,
      storage_backend: 'local'
    });
  });
});
