import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import BoldIcon from '@lucide/svelte/icons/bold';
import OverflowTrigger from '$lib/components/tiptap/toolbar/toolbar-overflow-trigger.svelte';

describe('ToolbarOverflowTrigger', () => {
  it('renders its label and forwards clicks', async () => {
    const onclick = vi.fn();
    render(OverflowTrigger, { icon: BoldIcon, label: 'Bold', onclick });
    expect(screen.getByText('Bold')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button'));
    expect(onclick).toHaveBeenCalledOnce();
  });

  it('marks the active state with the accent class', () => {
    render(OverflowTrigger, { active: true, icon: BoldIcon, label: 'Bold' });
    expect(screen.getByRole('button').className).toContain('bg-accent');
  });

  it('omits the accent class when inactive', () => {
    render(OverflowTrigger, { active: false, icon: BoldIcon, label: 'Bold' });
    expect(screen.getByRole('button').className).not.toContain('bg-accent');
  });
});
