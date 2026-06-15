import { describe, expect, it } from 'vitest';
import { load } from '$routes/setup/+page';
import { catchRedirect, jsonFetch } from '../../_helpers/load';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const event = (fetch: typeof globalThis.fetch) => ({ fetch }) as any;

describe('setup load', () => {
  it('redirects to / when setup is already complete', async () => {
    const redirect = await catchRedirect(() =>
      load(
        event(
          jsonFetch({
            db_backend: 'sqlite',
            is_setup: true,
            storage_backend: 'local'
          })
        )
      )
    );
    expect(redirect.status).toBe(302);
    expect(redirect.location).toBe('/');
  });

  it('returns the backends when setup is not complete', async () => {
    const result = await load(
      event(
        jsonFetch({
          db_backend: 'postgres',
          is_setup: false,
          storage_backend: 's3'
        })
      )
    );
    expect(result).toEqual({ db_backend: 'postgres', storage_backend: 's3' });
  });

  it('falls back to "unknown" backends when data is missing', async () => {
    const result = await load(event(jsonFetch(null)));
    expect(result).toEqual({
      db_backend: 'unknown',
      storage_backend: 'unknown'
    });
  });
});
