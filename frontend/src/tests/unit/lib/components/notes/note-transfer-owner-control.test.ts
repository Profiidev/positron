import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import TransferControl from '$lib/components/notes/NoteTransferOwnerControl.svelte';
import type { SimpleUserInfo } from '$lib/client';

const owner: SimpleUserInfo = { id: 'owner-1', name: 'Alice Owner' };

const candidateUsers: SimpleUserInfo[] = [
  { id: 'u1', name: 'Bob User' },
  { id: 'u2', name: 'Cara User' }
];

const base = {
  candidateUsers,
  onTransfer: vi.fn(),
  owner
};

describe('NoteTransferOwnerControl', () => {
  it('shows the current owner name on the trigger', () => {
    render(TransferControl, base);
    expect(screen.getByText('Alice Owner')).toBeInTheDocument();
  });

  it('opens the candidate list when the trigger is clicked', async () => {
    render(TransferControl, base);

    await fireEvent.click(
      screen.getByRole('button', {
        name: 'Transfer ownership from Alice Owner'
      })
    );

    expect(screen.getByPlaceholderText('Search people...')).toBeInTheDocument();
    expect(screen.getByText('Bob User')).toBeInTheDocument();
    expect(screen.getByText('Cara User')).toBeInTheDocument();
  });

  it('calls onTransfer when a candidate is selected', async () => {
    const onTransfer = vi.fn();
    render(TransferControl, { ...base, onTransfer });

    await fireEvent.click(
      screen.getByRole('button', {
        name: 'Transfer ownership from Alice Owner'
      })
    );
    await fireEvent.click(screen.getByRole('option', { name: /Cara User/ }));

    expect(onTransfer).toHaveBeenCalledWith('u2');
  });

  it('disables the trigger while saving', () => {
    render(TransferControl, { ...base, saving: true });
    expect(
      screen.getByRole('button', {
        name: 'Transfer ownership from Alice Owner'
      })
    ).toBeDisabled();
  });
});
