import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import type { NoteInfo, NotesConfig } from '$lib/commands/notes.svelte';

const deleteNote = vi.fn<() => Promise<boolean>>(async () => true);
const listNotesStore = vi.fn<() => Promise<NoteInfo[] | undefined>>(
  async () => undefined
);
vi.mock('$lib/commands/notes.svelte', () => ({ deleteNote, listNotesStore }));

const toast = { error: vi.fn(), success: vi.fn(), warning: vi.fn() };
vi.mock('@profidev/pleiades/components/util/general', async (orig) => ({
  ...(await orig<Record<string, unknown>>()),
  toast
}));

// The overview reads note data from the updater-backed reactive states, so mock
// Those getters rather than the raw commands.
let notesValue: NoteInfo[] | null = null;
let notesConfigValue: NotesConfig | undefined = undefined;
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

const note = (title: string, id: string, isOwner = true): NoteInfo => ({
  can_edit: true,
  id,
  is_owner: isOwner,
  owner: { id: 'owner', name: 'Owner' },
  preview: '',
  shared_with: [],
  title
});

const Page = (await import('$routes/(app)/+page.svelte')).default;

beforeEach(() => {
  notesValue = null;
  notesConfigValue = undefined;
});
afterEach(() => vi.clearAllMocks());

describe('notes overview', () => {
  it('renders the loaded notes as links to their detail pages', async () => {
    notesValue = [note('First', 'a'), note('Second', 'b')];
    render(Page);
    const link = await screen.findByRole('link', { name: /First/ });
    expect(link).toHaveAttribute('href', '/notes/a');
    expect(screen.getByRole('link', { name: /Second/ })).toHaveAttribute(
      'href',
      '/notes/b'
    );
  });

  it('shows an empty state when there are no notes', async () => {
    notesValue = [];
    render(Page);
    expect(await screen.findByText('No notes yet')).toBeInTheDocument();
  });

  it('falls back to the local store while remote notes are still loading', async () => {
    // NotesState stays null (loading); the local store resolves first.
    listNotesStore.mockResolvedValueOnce([note('Cached', 'c')]);
    render(Page);
    expect(
      await screen.findByRole('link', { name: /Cached/ })
    ).toBeInTheDocument();
  });

  it('lists who a note is shared with', async () => {
    notesValue = [
      {
        ...note('Team', 'a'),
        shared_with: [{ access: 'edit', id: 'u2', name: 'Bob' }]
      }
    ];
    render(Page);
    expect(await screen.findByText('Shared with Bob')).toBeInTheDocument();
  });

  it('shows the create affordance with a link to the create page when under the limit', () => {
    notesConfigValue = { max_per_user: 5 };
    notesValue = [note('Only', 'a')];
    render(Page);
    const create = screen.getByRole('link', { name: /Create/ });
    expect(create).toHaveAttribute('href', '/notes/create');
  });

  it('disables the create affordance when the per-user note limit is reached', () => {
    notesConfigValue = { max_per_user: 1 };
    notesValue = [note('Only', 'a')];
    render(Page);
    // At the limit the Button drops its href and renders a disabled button.
    expect(
      screen.queryByRole('link', { name: /Create/ })
    ).not.toBeInTheDocument();
    const create = screen.getByRole('button', { name: /Create/ });
    expect(create).toBeDisabled();
  });

  it('only counts owned notes toward the limit', () => {
    // One owned + one shared note, limit of 2 → not at the limit yet.
    notesConfigValue = { max_per_user: 2 };
    notesValue = [note('Mine', 'a', true), note('Theirs', 'b', false)];
    render(Page);
    expect(screen.getByRole('link', { name: /Create/ })).toBeInTheDocument();
  });

  it('opens a confirmation dialog and deletes the note on confirm', async () => {
    notesValue = [note('First', 'a')];
    const { container } = render(Page);
    await screen.findByRole('link', { name: /First/ });

    const trash = container.querySelector<HTMLButtonElement>(
      'a[href="/notes/a"] button'
    );
    expect(trash).not.toBeNull();
    trash!.click();

    expect(
      await screen.findByText(/Do you really want to delete the note First/)
    ).toBeInTheDocument();

    screen.getByRole('button', { name: 'Delete' }).click();
    await vi.waitFor(() => expect(deleteNote).toHaveBeenCalledWith('a'));
    await vi.waitFor(() =>
      expect(toast.success).toHaveBeenCalledWith(
        'Note First deleted successfully'
      )
    );
  });

  it('does not render a delete button on notes the user does not own', async () => {
    notesValue = [note('Theirs', 'b', false)];
    const { container } = render(Page);
    await screen.findByRole('link', { name: /Theirs/ });
    expect(container.querySelector('a[href="/notes/b"] button')).toBeNull();
  });
});
