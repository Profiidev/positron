import { describe, expect, it } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import type { Component } from 'svelte';
import { editorStub } from '$test_helpers/editor';
import Bold from '$lib/components/tiptap/toolbar/bold.svelte';
import Italic from '$lib/components/tiptap/toolbar/italic.svelte';
import Underline from '$lib/components/tiptap/toolbar/underline.svelte';
import Strikethrough from '$lib/components/tiptap/toolbar/strikethrough.svelte';
import Blockquote from '$lib/components/tiptap/toolbar/blockquote.svelte';
import BulletList from '$lib/components/tiptap/toolbar/bullet-list.svelte';
import OrderedList from '$lib/components/tiptap/toolbar/ordered-list.svelte';
import CodeBlock from '$lib/components/tiptap/toolbar/code-block.svelte';
import Undo from '$lib/components/tiptap/toolbar/undo.svelte';
import Redo from '$lib/components/tiptap/toolbar/redo.svelte';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const buttons: [string, Component<any>][] = [
  ['bold', Bold],
  ['italic', Italic],
  ['underline', Underline],
  ['strikethrough', Strikethrough],
  ['blockquote', Blockquote],
  ['bullet-list', BulletList],
  ['ordered-list', OrderedList],
  ['code-block', CodeBlock],
  ['undo', Undo],
  ['redo', Redo]
];

describe.each(buttons)('%s toolbar button', (_name, Cmp) => {
  // Render the overflow-menu variant: it is a plain button with no tooltip
  // Provider context, so it isolates the command/disabled logic cleanly.
  it('runs its editor command on click', async () => {
    const { actionRun, editor } = editorStub({ canRun: true });
    render(Cmp, { editor, inOverflowMenu: true });
    await fireEvent.click(screen.getAllByRole('button')[0]);
    expect(actionRun).toHaveBeenCalled();
  });

  it('is disabled when the command cannot run', () => {
    const { editor } = editorStub({ canRun: false });
    render(Cmp, { editor, inOverflowMenu: true });
    expect(screen.getAllByRole('button')[0]).toBeDisabled();
  });
});
