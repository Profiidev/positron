import { beforeEach, describe, expect, it, vi } from 'vitest';

const invalidate = vi.fn(async (_arg: unknown) => Promise.resolve());
const connect = vi.fn((_handler: unknown) => {});
const disconnect = vi.fn(() => {});

vi.mock('$app/navigation', () => ({
  invalidate: async (arg: unknown) => invalidate(arg)
}));
vi.mock('@profidev/pleiades/backend', () => ({
  createWebsocket: () => ({
    connect: (handler: unknown) => connect(handler),
    disconnect: () => disconnect()
  })
}));

const { UpdateType, connectWebsocket, disconnectWebsocket } =
  await import('$lib/backend/updater.svelte');

type Handler = (
  msg: { type: string; uuid?: string; note_id?: string },
  user: string
) => void;

/** Registers the websocket and returns the message handler pleiades received. */
const getHandler = (user = 'me'): Handler => {
  connectWebsocket(user);
  return connect.mock.calls.at(-1)?.[0] as Handler;
};

/** All string urls passed to invalidate (ignores predicate-function calls). */
const invalidatedUrls = () =>
  invalidate.mock.calls
    .map((c) => c[0])
    .filter((a): a is string => typeof a === 'string');

beforeEach(() => {
  invalidate.mockClear();
  connect.mockClear();
  disconnect.mockClear();
});

describe('UpdateType enum', () => {
  it('exposes every update kind as a matching string', () => {
    expect(UpdateType.Settings).toBe('Settings');
    expect(UpdateType.Note).toBe('Note');
    expect(UpdateType.NoteSnapshot).toBe('NoteSnapshot');
    expect(Object.values(UpdateType)).toContain('OAuthClient');
  });
});

describe('connect / disconnect delegation', () => {
  it('registers the user and a handler with pleiades', () => {
    connectWebsocket('alice');
    expect(connect).toHaveBeenCalledWith(expect.any(Function));
  });

  it('delegates disconnect', () => {
    disconnectWebsocket();
    expect(disconnect).toHaveBeenCalledOnce();
  });
});

describe('handleMessage', () => {
  it('invalidates settings via a path predicate', () => {
    const handler = getHandler();
    handler({ type: UpdateType.Settings }, 'me');
    const predicate = invalidate.mock.calls[0]?.[0] as (u: URL) => boolean;
    expect(typeof predicate).toBe('function');
    expect(predicate(new URL('http://x/api/settings/mail'))).toBe(true);
    expect(predicate(new URL('http://x/api/other'))).toBe(false);
  });

  it('invalidates the current user info only when the uuid matches', () => {
    const matching = getHandler('me');
    matching({ type: UpdateType.User, uuid: 'me' }, 'me');
    expect(invalidatedUrls()).toContain('/api/user/info');

    invalidate.mockClear();
    const other = getHandler('me');
    other({ type: UpdateType.User, uuid: 'someone-else' }, 'me');
    expect(invalidatedUrls()).not.toContain('/api/user/info');
  });

  it('falls through from OAuthClient into scope and policy invalidations', () => {
    const handler = getHandler();
    handler({ type: UpdateType.OAuthClient, uuid: 'c1' }, 'me');
    const urls = invalidatedUrls();
    expect(urls).toContain('/api/oauth_management/client');
    // Missing `break` means scope + policy invalidations run too
    expect(urls).toContain('/api/oauth_management/scope');
    expect(urls).toContain('/api/oauth_management/policy');
  });

  it('invalidates note endpoints with the uuid', () => {
    const handler = getHandler();
    handler({ type: UpdateType.Note, uuid: 'n1' }, 'me');
    expect(invalidatedUrls()).toEqual([
      '/api/notes/management',
      '/api/notes/management/n1'
    ]);
  });

  it('invalidates the snapshot list and info on a NoteSnapshot update', () => {
    const handler = getHandler();
    handler(
      { note_id: 'n1', type: UpdateType.NoteSnapshot, uuid: 's1' },
      'me'
    );
    expect(invalidatedUrls()).toEqual([
      '/api/notes/snapshots/n1',
      '/api/notes/snapshots/s1/info'
    ]);
  });

  it('does nothing for an unknown message type', () => {
    const handler = getHandler();
    handler({ type: 'Nonsense' }, 'me');
    expect(invalidate).not.toHaveBeenCalled();
  });
});
