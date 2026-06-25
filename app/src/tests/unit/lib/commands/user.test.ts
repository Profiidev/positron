import { beforeAll, describe, expect, it, vi } from 'vitest';
import { anyUserAvatar, userAvatar, userInfo } from '$lib/commands/user.svelte';
import {
  mockCommand,
  mockCommandError,
  mockCommands
} from '$test_helpers/tauri';

beforeAll(() => {
  // Jsdom has no object-URL implementation; userAvatar builds one from the
  // Webp bytes returned by the backend. Assign unconditionally so later
  // `vi.spyOn` calls have a method to wrap.
  globalThis.URL.createObjectURL = vi.fn(() => 'blob:mock');
});

describe('userInfo', () => {
  it('returns the user info object', async () => {
    const info = { email: 'ada@example.com', name: 'Ada', uuid: 'u1' };
    mockCommand('user_info', info);
    expect(await userInfo()).toEqual(info);
  });

  it('returns undefined when the command fails', async () => {
    mockCommandError('user_info');
    expect(await userInfo()).toBeUndefined();
  });
});

describe('userAvatar', () => {
  it('wraps the returned bytes in an object URL', async () => {
    const spy = vi
      .spyOn(globalThis.URL, 'createObjectURL')
      .mockReturnValue('blob:avatar');
    mockCommand('user_avatar', [1, 2, 3]);
    expect(await userAvatar()).toBe('blob:avatar');
    const blob = spy.mock.calls[0][0] as Blob;
    expect(blob.type).toBe('image/webp');
    spy.mockRestore();
  });

  it('returns undefined when the command fails', async () => {
    mockCommandError('user_avatar');
    expect(await userAvatar()).toBeUndefined();
  });
});

describe('anyUserAvatar', () => {
  it('forwards the uuid and wraps the returned bytes in an object URL', async () => {
    const spy = vi
      .spyOn(globalThis.URL, 'createObjectURL')
      .mockReturnValue('blob:any-avatar');
    let payload: unknown = undefined;
    mockCommands((cmd, p) => {
      if (cmd === 'any_user_avatar') {
        payload = p;
        return [7, 8, 9];
      }
      throw new Error('unexpected');
    });
    expect(await anyUserAvatar('u2')).toBe('blob:any-avatar');
    expect(payload).toEqual({ uuid: 'u2' });
    const blob = spy.mock.calls[0][0] as Blob;
    expect(blob.type).toBe('image/webp');
    spy.mockRestore();
  });

  it('returns undefined when the command fails', async () => {
    mockCommandError('any_user_avatar');
    expect(await anyUserAvatar('u2')).toBeUndefined();
  });
});
