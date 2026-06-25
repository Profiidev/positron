import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

const anyUserAvatar = vi.fn(async (_uuid: string) => 'blob:avatar');
vi.mock('$lib/commands/user.svelte', () => ({ anyUserAvatar }));

const UserAvatar = (await import('$lib/components/UserAvatar.svelte')).default;

afterEach(() => vi.clearAllMocks());

describe('UserAvatar', () => {
  it('shows the uppercased initials of the first two name parts', () => {
    render(UserAvatar, { userId: 'u1', username: 'ada grace lovelace' });
    // Avatar.Image never reports a load under jsdom, so the fallback shows.
    expect(screen.getByText('AG')).toBeInTheDocument();
  });

  it('shows a single initial for a one-word name', () => {
    render(UserAvatar, { userId: 'u1', username: 'Cher' });
    expect(screen.getByText('C')).toBeInTheDocument();
  });

  it('falls back to "?" when no username is given', () => {
    render(UserAvatar, { userId: 'u1' });
    expect(screen.getByText('?')).toBeInTheDocument();
  });

  it('fetches the avatar for the given user id', async () => {
    render(UserAvatar, { userId: 'u2', username: 'Bob' });
    await vi.waitFor(() => expect(anyUserAvatar).toHaveBeenCalledWith('u2'));
  });

  it('does not fetch an avatar when no user id is provided', async () => {
    render(UserAvatar, { username: 'Bob' });
    await Promise.resolve();
    expect(anyUserAvatar).not.toHaveBeenCalled();
  });
});
