import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import UserAvatar from '$lib/components/UserAvatar.svelte';

describe('UserAvatar', () => {
  it('shows two-letter uppercase initials from the username', () => {
    render(UserAvatar, { username: 'john doe' });
    expect(screen.getByText('JD')).toBeInTheDocument();
  });

  it('uses only the first two words for initials', () => {
    render(UserAvatar, { username: 'a b c d' });
    expect(screen.getByText('AB')).toBeInTheDocument();
  });

  it('falls back to "?" when there is no username', () => {
    render(UserAvatar, {});
    expect(screen.getByText('?')).toBeInTheDocument();
  });
});
