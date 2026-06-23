import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { goto } from '$app/navigation';
import type { NoteInfo, NotesConfig } from '$lib/commands/notes.svelte';

const logout = vi.fn();
vi.mock('$lib/commands/auth.svelte', () => ({ logout }));

const isConnected = vi.fn(() => true);
vi.mock('$lib/updater/updater.svelte', () => ({ isConnected }));

const deleteNote = vi.fn<() => Promise<boolean>>(async () =>
  Promise.resolve(true)
);
vi.mock('$lib/commands/notes.svelte', () => ({ deleteNote }));

// The overview reads note data from the updater-backed reactive states (the
// Same mechanism as user info), so mock those rather than the raw commands.
let notesValue: NoteInfo[] | null = [];
const notesConfigValue: NotesConfig | undefined = undefined;
const notesState = {
  update: vi.fn(),
  get value() {
    return notesValue;
  }
};
const notesConfigState = {
  update: vi.fn(),
  get value() {
    return notesConfigValue;
  }
};
vi.mock('$lib/updater/state.svelte', () => ({ notesConfigState, notesState }));

const note = (title: string, id: string): NoteInfo => ({
  can_edit: true,
  id,
  is_owner: true,
  owner: { id: 'owner', name: 'Owner' },
  preview: '',
  shared_with: [],
  title
});

const Page = (await import('$routes/+page.svelte')).default;

describe('home page', () => {
  it('logs out when the Logout button is clicked', async () => {
    render(Page);
    screen.getByRole('button', { name: 'Logout' }).click();
    await vi.waitFor(() => expect(logout).toHaveBeenCalled());
  });

  it('navigates to /scan when the Scan button is clicked', async () => {
    render(Page);
    screen.getByRole('button', { name: 'Scan Login' }).click();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/scan'));
  });

  it('navigates home when the Notes button is clicked', async () => {
    render(Page);
    screen.getByRole('button', { name: 'Notes' }).click();
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/'));
  });

  it('hides the Disconnected badge while connected', () => {
    isConnected.mockReturnValue(true);
    render(Page);
    expect(screen.queryByText('Disconnected')).not.toBeInTheDocument();
  });

  it('shows the Disconnected badge while disconnected', () => {
    isConnected.mockReturnValue(false);
    render(Page);
    expect(screen.getByText('Disconnected')).toBeInTheDocument();
  });

  it('renders the loaded notes as links', async () => {
    notesValue = [note('First', 'a'), note('Second', 'b')];
    render(Page);
    const link = await screen.findByRole('link', { name: /First/ });
    expect(link).toHaveAttribute('href', '/notes/a');
    expect(screen.getByRole('link', { name: /Second/ })).toBeInTheDocument();
  });

  it('shows an empty state when there are no notes', async () => {
    notesValue = [];
    render(Page);
    expect(await screen.findByText('No notes yet')).toBeInTheDocument();
  });
});
