import { beforeEach, describe, expect, it, vi } from 'vitest';

const invalidate = vi.fn(async (_arg: unknown) => Promise.resolve());
const connect = vi.fn((_handler: unknown, _path?: string) => {});
const disconnect = vi.fn(() => {});

vi.mock('$app/navigation', () => ({
  invalidate: async (arg: unknown) => invalidate(arg)
}));
vi.mock('@profidev/pleiades/backend', () => ({
  createWebsocket: () => ({
    connect: (handler: unknown, path?: string) => connect(handler, path),
    disconnect: () => disconnect()
  })
}));

const { connectPublicNoteUpdater, disconnectPublicNoteUpdater } =
  await import('$lib/backend/public-note-updater.svelte');

type Handler = () => void;

const getHandler = (noteId = 'n1'): Handler => {
  connectPublicNoteUpdater(noteId);
  return connect.mock.calls.at(-1)?.[0] as Handler;
};

beforeEach(() => {
  invalidate.mockClear();
  connect.mockClear();
  disconnect.mockClear();
});

describe('public note updater', () => {
  it('connects with a handler and the note update websocket path', () => {
    connectPublicNoteUpdater('note-42');
    expect(connect).toHaveBeenCalledWith(
      expect.any(Function),
      '/api/notes/update/note-42'
    );
  });

  it('invalidates the public note info on a websocket message', () => {
    const handler = getHandler('note-7');
    handler();
    expect(invalidate).toHaveBeenCalledWith(
      '/api/notes/management/note-7/public'
    );
  });

  it('swallows invalidate rejections', async () => {
    invalidate.mockRejectedValueOnce(new Error('boom'));
    const handler = getHandler('note-9');
    // Must not throw even though invalidate rejects
    expect(() => handler()).not.toThrow();
    await Promise.resolve();
  });

  it('delegates disconnect to the websocket', () => {
    disconnectPublicNoteUpdater();
    expect(disconnect).toHaveBeenCalled();
  });
});
