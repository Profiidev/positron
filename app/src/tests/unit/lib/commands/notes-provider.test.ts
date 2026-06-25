import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as Y from 'yjs';

// Capture every Channel the provider opens so a test can play backend messages
// Through the `onmessage` handler, and record every `invoke` call.
const channels = vi.hoisted(() => [] as { onmessage?: (m: unknown) => void }[]);
const invoke = vi.hoisted(() =>
  vi.fn(async (cmd: string, _args?: Record<string, unknown>) =>
    cmd === 'connect_note' ? 'conn-uuid' : undefined
  )
);
vi.mock('@tauri-apps/api/core', () => ({
  Channel: class {
    onmessage?: (m: unknown) => void;
    constructor() {
      channels.push(this);
    }
  },
  invoke
}));

const { TauriWebsocketProvider } = await import('$lib/commands/notes.svelte');

const calls = (cmd: string) =>
  invoke.mock.calls.filter(([name]) => name === cmd);

beforeEach(() => {
  channels.length = 0;
});

afterEach(() => vi.clearAllMocks());

describe('TauriWebsocketProvider construction', () => {
  it('exposes the note id, doc and awareness without connecting when connect:false', () => {
    const doc = new Y.Doc();
    const provider = new TauriWebsocketProvider('n1', doc, { connect: false });
    expect(provider.noteId).toBe('n1');
    expect(provider.doc).toBe(doc);
    expect(provider.awareness).toBeDefined();
    expect(invoke).not.toHaveBeenCalled();
    provider.destroy();
  });
});

describe('synced setter', () => {
  it('emits sync only when the value changes', () => {
    const doc = new Y.Doc();
    const provider = new TauriWebsocketProvider('n1', doc, { connect: false });
    const sync = vi.fn();
    provider.on('sync', sync);

    expect(provider.synced).toBe(false);
    provider.synced = true;
    expect(provider.synced).toBe(true);
    expect(sync).toHaveBeenCalledWith(true);

    // Setting the same value again is a no-op.
    provider.synced = true;
    expect(sync).toHaveBeenCalledTimes(1);

    provider.synced = false;
    expect(sync).toHaveBeenCalledTimes(2);
    provider.destroy();
  });
});

describe('connect / disconnect lifecycle', () => {
  it('connects through connect_note and sends the initial sync step', async () => {
    const doc = new Y.Doc();
    const status: string[] = [];
    const provider = new TauriWebsocketProvider('n1', doc, { connect: false });
    provider.on('status', ({ status: s }) => status.push(s));

    provider.connect();
    // The "connecting" status is emitted synchronously.
    expect(status).toContain('connecting');

    await vi.waitFor(() => expect(calls('connect_note')).toHaveLength(1));
    expect(calls('connect_note')[0][1]).toMatchObject({ note: 'n1' });

    // Once connected it flushes sync step 1 over the channel.
    await vi.waitFor(() => expect(status).toContain('connected'));
    await vi.waitFor(() =>
      expect(calls('send_note').length).toBeGreaterThan(0)
    );
    expect(calls('send_note')[0][1]).toMatchObject({ uuid: 'conn-uuid' });

    provider.destroy();
  });

  it('disconnects through disconnect_note', async () => {
    const doc = new Y.Doc();
    const provider = new TauriWebsocketProvider('n1', doc, { connect: false });
    provider.connect();
    await vi.waitFor(() => expect(calls('connect_note')).toHaveLength(1));
    await vi.waitFor(() =>
      expect(calls('send_note').length).toBeGreaterThan(0)
    );

    provider.disconnect();
    await vi.waitFor(() => expect(calls('disconnect_note')).toHaveLength(1));
    expect(calls('disconnect_note')[0][1]).toEqual({ uuid: 'conn-uuid' });
    provider.destroy();
  });

  it('does not establish a connection when connect is never called and connect:false', async () => {
    const doc = new Y.Doc();
    const provider = new TauriWebsocketProvider('n1', doc, { connect: false });
    // Give any stray microtasks a chance to run.
    await Promise.resolve();
    expect(calls('connect_note')).toHaveLength(0);
    provider.destroy();
  });
});
