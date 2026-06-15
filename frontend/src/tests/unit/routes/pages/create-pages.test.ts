import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/svelte';
import GroupCreate from '$routes/groups/create/+page.svelte';
import NoteCreate from '$routes/notes/create/+page.svelte';
import ClientCreate from '$routes/oauth-client/create/+page.svelte';
import PolicyCreate from '$routes/oauth-policy/create/+page.svelte';
import ScopeCreate from '$routes/oauth-scope/create/+page.svelte';
import UserCreate from '$routes/users/create/+page.svelte';
import ApodDate from '$routes/apod/[date]/+page.svelte';

const pr = async <T>(v: T) => Promise.resolve(v);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const mounts = (Cmp: any, data: unknown) => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const { container } = render(Cmp, { data } as any);
  expect(container.innerHTML.length).toBeGreaterThan(0);
};

describe('create pages mount', () => {
  it('group create', () => mounts(GroupCreate, { uuid: undefined }));
  it('note create', () => mounts(NoteCreate, { id: undefined }));
  it('oauth-client create', () =>
    mounts(ClientCreate, {
      client_id: undefined,
      client_secret: undefined,
      scopes: pr([])
    }));
  it('oauth-policy create', () => mounts(PolicyCreate, { uuid: undefined }));
  it('oauth-scope create', () =>
    mounts(ScopeCreate, { policies: pr([]), uuid: undefined }));
  it('user create', () =>
    mounts(UserCreate, { mailActive: pr(false), uuid: undefined }));
});

describe('apod [date] page mounts', () => {
  it('renders with image info', () => {
    mounts(ApodDate, { apodInfo: pr({ title: 'Galaxy' }), date: '2024-01-02' });
  });

  it('renders when the image is unavailable', () => {
    mounts(ApodDate, { apodInfo: pr(null), date: '2024-01-02' });
  });
});
