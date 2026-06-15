import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/svelte';
import Account from '$routes/account/+page.svelte';
import Password from '$routes/password/+page.svelte';
import Settings from '$routes/settings/+page.svelte';
import Apod from '$routes/apod/+page.svelte';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const mounts = (Cmp: any) => {
  const { container } = render(Cmp);
  // These are redirect placeholders; they should mount without throwing
  expect(container).toBeTruthy();
};

describe('placeholder/redirect pages mount', () => {
  it('account index', () => mounts(Account));
  it('password index', () => mounts(Password));
  it('settings index', () => mounts(Settings));
  it('apod index', () => mounts(Apod));
});
