import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import { goto } from '$app/navigation';
import NoteCreate from '$routes/notes/create/+page.svelte';
import type { NoteInfo } from '$lib/client';

const ownedNote = (id: string): NoteInfo => ({
  can_edit: true,
  id,
  is_owner: true,
  last_updated: new Date().toISOString(),
  owner: { id: 'user-1', name: 'Me' },
  preview: '',
  shared_with: [],
  title: 'My note'
});

// The create page only reads `notes`/`notesConfig`; the rest of the inherited
// Layout data is irrelevant here, so cast to satisfy the full prop type.
const pageData = (notes: NoteInfo[], maxPerUser: number) =>
  ({
    notes: Promise.resolve(notes),
    notesConfig: Promise.resolve({ max_per_user: maxPerUser })
  }) as never;

describe('note create page', () => {
  beforeEach(() => {
    vi.mocked(goto).mockClear();
  });

  it('redirects to the list when already at the note limit on entry', async () => {
    render(NoteCreate, { data: pageData([ownedNote('note-1')], 1) });

    await waitFor(() => {
      expect(goto).toHaveBeenCalledWith('/notes');
    });
  });

  it('does not redirect to the list when invalidated data reaches the limit after entry', async () => {
    const { rerender } = render(NoteCreate, { data: pageData([], 1) });

    await waitFor(() => expect(goto).not.toHaveBeenCalled());

    await rerender({ data: pageData([ownedNote('note-new')], 1) });

    await new Promise((resolve) => setTimeout(resolve, 50));

    expect(goto).not.toHaveBeenCalledWith('/notes');
  });
});
