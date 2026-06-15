import { vi } from 'vitest';
import type { Editor } from '@tiptap/core';

/**
 * A minimal chainable tiptap `Editor` stub for toolbar-button tests.
 *
 * `editor.chain().focus().<anything>().run()` and
 * `editor.can().chain().focus().<anything>().run()` both work via a proxy that
 * returns itself for any method. `can`'s terminal `run()` returns `canRun`
 * (drives the disabled state); the action chain's `run()` is a spy so a click
 * can be asserted.
 */
export const editorStub = ({
  active = false,
  canRun = true,
  isActive,
  attributes = {}
}: {
  active?: boolean;
  canRun?: boolean;
  isActive?: (name: string, opts?: Record<string, unknown>) => boolean;
  attributes?: Record<string, unknown>;
} = {}) => {
  const actionRun = vi.fn(() => true);

  const canChain: Record<string, unknown> = new Proxy(
    {},
    { get: (_t, prop) => (prop === 'run' ? () => canRun : () => canChain) }
  );
  const actionChain: Record<string, unknown> = new Proxy(
    {},
    { get: (_t, prop) => (prop === 'run' ? actionRun : () => actionChain) }
  );

  const editor = {
    can: () => ({ chain: () => canChain }),
    chain: () => actionChain,
    getAttributes: () => attributes,
    isActive: isActive ?? (() => active)
  } as unknown as Editor;

  return { actionRun, editor };
};
