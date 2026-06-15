import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Mail from '$routes/settings/mail/+page.svelte';
import ApodList from '$routes/apod/list/+page.svelte';
import Setup from '$routes/setup/+page.svelte';

const P =  async <T,>(v: T) => Promise.resolve(v);
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const r = (Cmp: any, data: unknown) => render(Cmp, { data } as any);

describe('misc pages', () => {
  it('mail settings renders its heading', () => {
    r(Mail, {
      settings: P({ from_env: [], smtp_enabled: false, smtp_use_tls: false }),
      user: P(undefined)
    });
    expect(screen.getByText('Mail Settings')).toBeInTheDocument();
  });

  it('apod list mounts with an empty list', () => {
    const { container } = r(ApodList, { apodList: P([]) });
    expect(container.innerHTML.length).toBeGreaterThan(0);
  });

  it('setup wizard renders the first stage', () => {
    r(Setup, { db_backend: 'sqlite', storage_backend: 'local' });
    expect(screen.getByText('Database Setup')).toBeInTheDocument();
  });
});
