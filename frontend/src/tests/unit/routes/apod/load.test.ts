import { describe, expect, it } from 'vitest';
import { load as datePageLoad } from '$routes/apod/[date]/+page';
import { load as listPageLoad } from '$routes/apod/list/+page';
import { load as redirectLoad } from '$routes/apod/+page.server';
import { catchRedirect, jsonFetch, runLoad } from '$test_helpers/load';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const ev = (props: Record<string, unknown>) => props as any;

describe('apod/+page.server.ts load', () => {
  it("redirects to today's apod page", async () => {
    const [today] = new Date().toISOString().split('T');
    const redirect = await catchRedirect(() => redirectLoad());
    expect(redirect.status).toBe(302);
    expect(redirect.location).toBe(`/apod/${today}`);
  });
});

describe('apod/[date]/+page.ts load', () => {
  it('returns the date and the image info on success', async () => {
    const result = await runLoad(
      datePageLoad,
      ev({
        fetch: jsonFetch({ title: 'Galaxy' }),
        params: { date: '2024-01-02' }
      })
    );
    expect(result.date).toBe('2024-01-02');
    await expect(result.apodInfo).resolves.toMatchObject({ title: 'Galaxy' });
  });

  it('resolves to null when the image is gone (410)', async () => {
    const result = await runLoad(
      datePageLoad,
      ev({
        fetch: jsonFetch({ title: 'x' }, 410),
        params: { date: '2024-01-02' }
      })
    );
    await expect(result.apodInfo).resolves.toBeNull();
  });
});

describe('apod/list/+page.ts load', () => {
  it('resolves the apod list data', async () => {
    const result = await runLoad(
      listPageLoad,
      ev({ fetch: jsonFetch([{ id: 1 }]) })
    );
    await expect(result.apodList).resolves.toEqual([{ id: 1 }]);
  });
});
