import { describe, expect, it } from 'vitest';
import { resetSetup, setup, setupStatus } from '$lib/commands/setup.svelte';
import {
  mockCommand,
  mockCommandError,
  mockCommands
} from '$test_helpers/tauri';

describe('setup', () => {
  it('forwards the url and returns true on success', async () => {
    let received: unknown = undefined;
    mockCommands((cmd, payload) => {
      if (cmd === 'setup') {
        received = payload;
        return null;
      }
      throw new Error('unexpected');
    });
    expect(await setup('https://example.com')).toBe(true);
    expect(received).toEqual({ url: 'https://example.com' });
  });

  it('returns false when the command fails', async () => {
    mockCommandError('setup');
    expect(await setup('https://example.com')).toBe(false);
  });
});

describe('setupStatus', () => {
  it('returns the status object', async () => {
    mockCommand('setup_status', { url: 'https://example.com' });
    expect(await setupStatus()).toEqual({ url: 'https://example.com' });
  });

  it('returns the status object with no url', async () => {
    mockCommand('setup_status', {});
    expect(await setupStatus()).toEqual({});
  });

  it('returns undefined when the command fails', async () => {
    mockCommandError('setup_status');
    expect(await setupStatus()).toBeUndefined();
  });
});

describe('resetSetup', () => {
  it('returns true on success', async () => {
    mockCommand('reset_setup', null);
    expect(await resetSetup()).toBe(true);
  });

  it('returns false when the command fails', async () => {
    mockCommandError('reset_setup');
    expect(await resetSetup()).toBe(false);
  });
});
