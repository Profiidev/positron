import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Indicator from '$lib/components/notes/NoteActiveEditorsIndicator.svelte';
import type { NoteActiveEditor } from '$lib/components/notes/types';

const editors = (n: number): NoteActiveEditor[] =>
  Array.from({ length: n }, (_, i) => ({
    clientId: i,
    color: i % 2 ? '#fff' : undefined,
    id: `u${i}`,
    name: `User${i}`
  }));

describe('NoteActiveEditorsIndicator', () => {
  it('renders nothing when there are no editors', () => {
    const { container } = render(Indicator, { editors: [] });
    expect(container.textContent).not.toMatch(/editing/);
  });

  it.each([1, 2, 4])('shows the count for %i editor(s)', (n) => {
    render(Indicator, { editors: editors(n) });
    expect(screen.getByText(`${n} editing`)).toBeInTheDocument();
  });

  it('does not show an overflow badge at four or fewer editors', () => {
    render(Indicator, { editors: editors(4) });
    expect(screen.queryByText(/^\+/)).toBeNull();
  });

  it('shows a "+N" overflow badge beyond four editors', () => {
    render(Indicator, { editors: editors(7) });
    expect(screen.getByText('7 editing')).toBeInTheDocument();
    expect(screen.getByText('+3')).toBeInTheDocument();
  });

  it('renders avatar initials for the visible editors', () => {
    render(Indicator, { editors: [{ clientId: 1, name: 'Jane Doe' }] });
    // First four editors are rendered as avatars in the trigger
    expect(screen.getAllByText('JD').length).toBeGreaterThan(0);
  });
});
