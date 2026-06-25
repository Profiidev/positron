import { describe, expect, it, vi } from 'vitest';
import {
  fireEvent,
  render,
  screen,
  waitFor,
  within
} from '@testing-library/svelte';
import NoteSnapshot from '$lib/components/notes/NoteSnapshot.svelte';
import type { NoteSnapshotInfo } from '$lib/client';

// `D.DateTime` is loaded lazily from luxon only in the browser, so it stays
// Undefined under jsdom. Stub it with a deterministic formatter so the snapshot
// Items render readable, selectable labels.
vi.mock('@profidev/pleiades/util/time.svelte', () => ({
  DateTime: {
    DateTime: {
      DATETIME_MED_WITH_WEEKDAY: {},
      fromISO: (iso: string) => ({ toLocaleString: () => `at ${iso}` })
    }
  }
}));

const snapshots: NoteSnapshotInfo[] = [
  {
    created_at: '2024-01-02T00:00:00Z',
    id: 'snapshot-1',
    note_id: 'note-1',
    preview: 'Preview one'
  },
  {
    created_at: '2024-01-03T00:00:00Z',
    id: 'snapshot-2',
    note_id: 'note-1',
    preview: 'Preview two'
  }
];

const base = {
  onDelete: vi.fn(),
  onOpen: vi.fn(),
  onRestore: vi.fn(),
  snapshots
};

const openManager = async () => {
  await fireEvent.click(
    screen.getByRole('button', { name: 'Snapshot manager' })
  );
};

describe('NoteSnapshot', () => {
  it('lists every snapshot when the manager is opened', async () => {
    render(NoteSnapshot, base);
    await openManager();

    expect(
      screen.getByPlaceholderText('Search snapshots...')
    ).toBeInTheDocument();
    expect(screen.getAllByRole('option')).toHaveLength(2);
    expect(screen.getByText('at 2024-01-02T00:00:00Z')).toBeInTheDocument();
    expect(screen.getByText('at 2024-01-03T00:00:00Z')).toBeInTheDocument();
  });

  it('calls onOpen with the snapshot id when an item is selected', async () => {
    const onOpen = vi.fn();
    render(NoteSnapshot, { ...base, onOpen });
    await openManager();

    await fireEvent.click(screen.getAllByRole('option')[0]);

    expect(onOpen).toHaveBeenCalledWith('snapshot-1');
  });

  it('calls onRestore from the restore button without opening the snapshot', async () => {
    const onOpen = vi.fn();
    const onRestore = vi.fn();
    render(NoteSnapshot, { ...base, onOpen, onRestore });
    await openManager();

    // First button inside the option is the restore (ArchiveRestore) action.
    const [restore] = within(screen.getAllByRole('option')[0]).getAllByRole(
      'button'
    );
    await fireEvent.click(restore);

    expect(onRestore).toHaveBeenCalledWith('snapshot-1');
    // StopPropagation keeps the item's onSelect (onOpen) from firing.
    expect(onOpen).not.toHaveBeenCalled();
  });

  it('calls onDelete from the destructive button without opening the snapshot', async () => {
    const onOpen = vi.fn();
    const onDelete = vi.fn();
    render(NoteSnapshot, { ...base, onDelete, onOpen });
    await openManager();

    // The destructive delete button is the last button in the option.
    const buttons = within(screen.getAllByRole('option')[1]).getAllByRole(
      'button'
    );
    await fireEvent.click(buttons[buttons.length - 1]);

    expect(onDelete).toHaveBeenCalledWith('snapshot-2');
    expect(onOpen).not.toHaveBeenCalled();
  });

  it('closes the popover after selecting a snapshot', async () => {
    render(NoteSnapshot, base);
    await openManager();
    expect(
      screen.getByPlaceholderText('Search snapshots...')
    ).toBeInTheDocument();

    await fireEvent.click(screen.getAllByRole('option')[0]);

    await waitFor(() =>
      expect(
        screen.queryByPlaceholderText('Search snapshots...')
      ).not.toBeInTheDocument()
    );
  });

  it('shows the empty state when there are no snapshots', async () => {
    render(NoteSnapshot, { ...base, snapshots: [] });
    await openManager();

    expect(screen.getByText('No snapshots found')).toBeInTheDocument();
    expect(screen.queryByRole('option')).toBeNull();
  });

  it('disables the trigger while saving', () => {
    render(NoteSnapshot, { ...base, saving: true });
    expect(
      screen.getByRole('button', { name: 'Snapshot manager' })
    ).toBeDisabled();
  });
});
