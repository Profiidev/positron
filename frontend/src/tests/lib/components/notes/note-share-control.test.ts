import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ShareControl from '$lib/components/notes/NoteShareControl.svelte';
import type { SimpleUserInfo } from '$lib/client';

const users = (n: number): SimpleUserInfo[] =>
  Array.from(
    { length: n },
    (_, i) => ({ id: `u${i}`, name: `User${i}` }) as unknown as SimpleUserInfo
  );

const base = {
  onSelectChange: vi.fn(),
  shareableUsers: users(3)
};

describe('NoteShareControl (editable)', () => {
  it('shows the Share placeholder when nothing is selected', () => {
    render(ShareControl, { ...base, selected: [] });
    expect(screen.getByText('Share')).toBeInTheDocument();
  });

  it.each([1, 3])('shows the shared count for %i user(s)', (n) => {
    render(ShareControl, { ...base, selected: users(n) });
    expect(screen.getByText(`${n} shared`)).toBeInTheDocument();
  });

  it('shows a "+N" badge beyond four shared users', () => {
    render(ShareControl, { ...base, selected: users(6) });
    expect(screen.getByText('6 shared')).toBeInTheDocument();
    expect(screen.getByText('+2')).toBeInTheDocument();
  });

  it('disables the trigger while saving', () => {
    render(ShareControl, { ...base, saving: true, selected: [] });
    expect(screen.getByRole('button')).toBeDisabled();
  });
});

describe('NoteShareControl (readonly)', () => {
  it('renders a non-interactive Share placeholder when empty', () => {
    render(ShareControl, { ...base, readonly: true, selected: [] });
    expect(screen.getByText('Share')).toBeInTheDocument();
    expect(screen.queryByRole('button')).toBeNull();
  });

  it('shows the shared count without a button', () => {
    render(ShareControl, { ...base, readonly: true, selected: users(2) });
    expect(screen.getByText('2 shared')).toBeInTheDocument();
    expect(screen.queryByRole('button')).toBeNull();
  });
});
