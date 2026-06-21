import { describe, expect, it } from 'vitest';
import {
  authStatus,
  confirmCode,
  logout,
  startAuth
} from '$lib/commands/auth.svelte';
import {
  mockCommand,
  mockCommandError,
  mockCommands
} from '$test_helpers/tauri';

describe('authStatus', () => {
  it('returns the backend boolean', async () => {
    mockCommand('auth_status', true);
    expect(await authStatus()).toBe(true);
  });

  it('returns undefined when the command fails', async () => {
    mockCommandError('auth_status');
    expect(await authStatus()).toBeUndefined();
  });
});

describe('startAuth', () => {
  it('returns the challenge string', async () => {
    mockCommand('start_auth', 'challenge-123');
    expect(await startAuth()).toBe('challenge-123');
  });

  it('returns undefined when the command fails', async () => {
    mockCommandError('start_auth');
    expect(await startAuth()).toBeUndefined();
  });
});

describe('confirmCode', () => {
  it('forwards the code and returns true on success', async () => {
    let received: unknown = undefined;
    mockCommands((cmd, payload) => {
      if (cmd === 'confirm_code') {
        received = payload;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await confirmCode('abc')).toBe(true);
    expect(received).toEqual({ code: 'abc' });
  });

  it('returns undefined when the command fails', async () => {
    mockCommandError('confirm_code');
    expect(await confirmCode('abc')).toBeUndefined();
  });
});

describe('logout', () => {
  it('returns true on success', async () => {
    mockCommand('logout', null);
    expect(await logout()).toBe(true);
  });

  it('returns undefined when the command fails', async () => {
    mockCommandError('logout');
    expect(await logout()).toBeUndefined();
  });
});
