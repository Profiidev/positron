import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import type { Component } from 'svelte';
import { editorStub } from '$test_helpers/editor';
import Headings from '$lib/components/tiptap/toolbar/headings.svelte';
import Alignment from '$lib/components/tiptap/toolbar/alignment.svelte';
import Link from '$lib/components/tiptap/toolbar/link.svelte';
import ColorHighlight from '$lib/components/tiptap/toolbar/color-and-highlight.svelte';

describe('headings overflow trigger label', () => {
  it('reads "Normal" when no heading is active', () => {
    const { editor } = editorStub({ isActive: () => false });
    render(Headings, { editor, inOverflowMenu: true });
    expect(screen.getByText('Normal')).toBeInTheDocument();
  });

  it('reads "Heading N" for the active level', () => {
    const { editor } = editorStub({
      isActive: (name, opts) =>
        name === 'heading' && (opts === undefined || opts.level === 2)
    });
    render(Headings, { editor, inOverflowMenu: true });
    expect(screen.getByText('Heading 2')).toBeInTheDocument();
  });
});

// The overflow-menu variant of each dropdown item uses a self-contained
// DropdownMenu/Popover root, so it renders without a tooltip provider.
const items: [string, Component<any>][] = [
  ['headings', Headings],
  ['alignment', Alignment],
  ['link', Link],
  ['color-and-highlight', ColorHighlight]
];

describe.each(items)('%s renders in the overflow menu', (_name, Cmp) => {
  it('mounts a trigger button without crashing', () => {
    const { editor } = editorStub();
    render(Cmp, { editor, inOverflowMenu: true });
    expect(screen.getAllByRole('button').length).toBeGreaterThan(0);
  });
});
