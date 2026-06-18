import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import ShareControl from '$lib/components/notes/NoteShareControl.svelte';
import type { SharedUserInfo, SimpleUserInfo } from '$lib/client';

const shareableUsers: SimpleUserInfo[] = [
  { id: 'u1', name: 'Alice' },
  { id: 'u2', name: 'Bob' },
  { id: 'u3', name: 'Cara' }
];

const shared = (users: SharedUserInfo[]) => users;

const base = {
  noteId: 'note-1',
  onPublicAccessChange: vi.fn(),
  onShareChange: vi.fn(),
  shareableUsers
};

describe('NoteShareControl (editable)', () => {
  it('shows the Share placeholder when nothing is selected', () => {
    render(ShareControl, { ...base, selected: [] });
    expect(screen.getByText('Share')).toBeInTheDocument();
  });

  it.each([1, 3])('shows the shared count for %i user(s)', (n) => {
    render(ShareControl, {
      ...base,
      selected: shared(
        shareableUsers.slice(0, n).map((user) => ({
          access: 'edit' as const,
          id: user.id,
          name: user.name
        }))
      )
    });
    expect(screen.getByText(`${n} shared`)).toBeInTheDocument();
  });

  it('shows a "+N" badge beyond four shared users', () => {
    const selected = Array.from({ length: 6 }, (_, i) => ({
      access: 'edit' as const,
      id: `u${i}`,
      name: `User${i}`
    }));
    render(ShareControl, { ...base, selected });
    expect(screen.getByText('6 shared')).toBeInTheDocument();
    expect(screen.getByText('+2')).toBeInTheDocument();
  });

  it('disables the trigger while saving', () => {
    render(ShareControl, { ...base, saving: true, selected: [] });
    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('adds a user with edit access when Edit is clicked', async () => {
    const onShareChange = vi.fn();
    render(ShareControl, { ...base, onShareChange, selected: [] });

    await fireEvent.click(screen.getByRole('button', { name: 'Share' }));
    await fireEvent.click(screen.getAllByRole('button', { name: 'Edit' })[1]);

    expect(onShareChange).toHaveBeenCalledWith([
      { access: 'edit', userId: 'u1' }
    ]);
  });

  it('adds a user with view access when View is clicked', async () => {
    const onShareChange = vi.fn();
    render(ShareControl, { ...base, onShareChange, selected: [] });

    await fireEvent.click(screen.getByRole('button', { name: 'Share' }));
    await fireEvent.click(screen.getAllByRole('button', { name: 'View' })[1]);

    expect(onShareChange).toHaveBeenCalledWith([
      { access: 'view', userId: 'u1' }
    ]);
  });

  it('revokes share when the active permission is clicked again', async () => {
    const onShareChange = vi.fn();
    render(ShareControl, {
      ...base,
      onShareChange,
      selected: [{ access: 'edit', id: 'u1', name: 'Alice' }]
    });

    await fireEvent.click(screen.getByRole('button', { name: /1 shared/ }));
    await fireEvent.click(screen.getAllByRole('button', { name: 'Edit' })[1]);

    expect(onShareChange).toHaveBeenCalledWith([]);
  });

  it('switches permission when the inactive button is clicked', async () => {
    const onShareChange = vi.fn();
    render(ShareControl, {
      ...base,
      onShareChange,
      selected: [{ access: 'edit', id: 'u1', name: 'Alice' }]
    });

    await fireEvent.click(screen.getByRole('button', { name: /1 shared/ }));
    await fireEvent.click(screen.getAllByRole('button', { name: 'View' })[1]);

    expect(onShareChange).toHaveBeenCalledWith([
      { access: 'view', userId: 'u1' }
    ]);
  });

  it('toggles public view access', async () => {
    const onPublicAccessChange = vi.fn();
    render(ShareControl, { ...base, onPublicAccessChange, selected: [] });

    await fireEvent.click(screen.getByRole('button', { name: 'Share' }));
    await fireEvent.click(screen.getAllByRole('button', { name: 'View' })[0]);

    expect(onPublicAccessChange).toHaveBeenCalledWith('view');
  });

  it('shows copy link when public access is enabled', async () => {
    render(ShareControl, {
      ...base,
      publicAccess: 'view',
      selected: []
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Public' }));
    expect(screen.getByText('Copy share link')).toBeInTheDocument();
  });
});

describe('NoteShareControl (readonly)', () => {
  it('renders a non-interactive Share placeholder when empty', () => {
    render(ShareControl, { ...base, readonly: true, selected: [] });
    expect(screen.getByText('Share')).toBeInTheDocument();
    expect(screen.queryByRole('button')).toBeNull();
  });

  it('shows the shared count without a button', () => {
    render(ShareControl, {
      ...base,
      readonly: true,
      selected: shared(
        shareableUsers.slice(0, 2).map((user) => ({
          access: 'view' as const,
          id: user.id,
          name: user.name
        }))
      )
    });
    expect(screen.getByText('2 shared')).toBeInTheDocument();
    expect(screen.queryByRole('button')).toBeNull();
  });
});
