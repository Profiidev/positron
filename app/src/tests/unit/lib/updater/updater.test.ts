import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { goto } from '$app/navigation';
import { page } from '$app/state';
import {
  type UpdateMessage,
  UpdateMessageType
} from '$lib/updater/types.svelte';

const toast = vi.hoisted(() => ({
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn()
}));
vi.mock('@profidev/pleiades/components/util/general', () => ({ toast }));

// Connection toasts are now gated on the auth status, and every message forwards
// Its type to `triggerUpdates`. Mock the state module so both are controllable.
const authStatusState = vi.hoisted(() => ({ value: true as unknown }));
const triggerUpdates = vi.hoisted(() => vi.fn());
vi.mock('$lib/updater/state.svelte', () => ({
  authStatusState,
  triggerUpdates
}));

// Capture each Channel so the test can play backend messages through the
// `onmessage` handler the updater installs.
const channels = vi.hoisted(() => [] as { onmessage?: (m: unknown) => void }[]);
const invoke = vi.hoisted(() =>
  vi.fn(async (cmd: string) =>
    cmd === 'connect_updater' ? 'updater-uuid' : undefined
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

const { startListener, isConnected, setOnline } =
  await import('$lib/updater/updater.svelte');

const send = (message: UpdateMessage) => channels.at(-1)!.onmessage!(message);

beforeEach(() => {
  channels.length = 0;
  page.route.id = '/';
  authStatusState.value = true;
});

afterEach(() => vi.clearAllMocks());

describe('setOnline', () => {
  it('forwards the flag and returns true on success', async () => {
    expect(await setOnline(true)).toBe(true);
    expect(invoke).toHaveBeenCalledWith('set_online', { online: true });
  });

  it('returns false when the command fails', async () => {
    invoke.mockRejectedValueOnce(new Error('offline'));
    expect(await setOnline(false)).toBe(false);
  });
});

describe('startListener', () => {
  it('connects the channel and returns a disconnect cleanup', async () => {
    const cleanup = await startListener();
    expect(invoke).toHaveBeenCalledWith('connect_updater', {
      channel: channels.at(-1)
    });
    cleanup();
    expect(invoke).toHaveBeenCalledWith('disconnect_updater', {
      uuid: 'updater-uuid'
    });
  });
});

describe('handleMessage', () => {
  beforeEach(async () => {
    await startListener();
  });

  it('redirects to /auth on TokenInvalid', () => {
    send({ type: UpdateMessageType.TokenInvalid });
    expect(goto).toHaveBeenCalledWith('/auth');
  });

  it('does not redirect on TokenInvalid when already on /auth', () => {
    page.route.id = '/auth';
    send({ type: UpdateMessageType.TokenInvalid });
    expect(goto).not.toHaveBeenCalled();
  });

  it('warns and flips connection on Disconnected, restores on Connected', () => {
    expect(isConnected()).toBe(true);
    send({ type: UpdateMessageType.Disconnected });
    expect(toast.warning).toHaveBeenCalledWith('Failed to connect to server');
    expect(isConnected()).toBe(false);

    send({ type: UpdateMessageType.Connected });
    expect(toast.success).toHaveBeenCalledWith('Connection restored');
    expect(isConnected()).toBe(true);
  });

  it('does not warn twice while already disconnected', () => {
    send({ type: UpdateMessageType.Disconnected });
    toast.warning.mockClear();
    send({ type: UpdateMessageType.Disconnected });
    expect(toast.warning).not.toHaveBeenCalled();
    send({ type: UpdateMessageType.Connected });
  });

  it('does not toast connection changes while unauthenticated', () => {
    authStatusState.value = false;
    send({ type: UpdateMessageType.Disconnected });
    send({ type: UpdateMessageType.Connected });
    expect(toast.warning).not.toHaveBeenCalled();
    expect(toast.success).not.toHaveBeenCalled();
  });

  it('refreshes all data when the connection is restored', () => {
    send({ type: UpdateMessageType.Disconnected });
    triggerUpdates.mockClear();
    send({ type: UpdateMessageType.Connected });
    // Connected fires an untyped refresh plus the trailing typed one.
    expect(triggerUpdates).toHaveBeenCalledWith();
  });

  it('forwards the message type to triggerUpdates', () => {
    send({ type: UpdateMessageType.AuthStatusUpdated });
    expect(triggerUpdates).toHaveBeenCalledWith(
      UpdateMessageType.AuthStatusUpdated
    );
  });

  it.each([
    [UpdateMessageType.CodeExchangeFailed, 'Failed to exchange code', '/auth'],
    [
      UpdateMessageType.CodeExchangeMissingCode,
      'Code exchange missing code',
      '/auth'
    ],
    [
      UpdateMessageType.CodeExchangeMissingVerifier,
      'Code exchange missing verifier',
      '/auth'
    ],
    [
      UpdateMessageType.ConfirmAuthMissingCode,
      'Code missing for confirmation',
      '/'
    ]
  ])('toasts an error and redirects for %s', (type, message, target) => {
    send({ type } as UpdateMessage);
    expect(toast.error).toHaveBeenCalledWith(message);
    expect(goto).toHaveBeenCalledWith(target);
  });

  it('toasts success and goes home on AuthSuccess', () => {
    send({ type: UpdateMessageType.AuthSuccess });
    expect(toast.success).toHaveBeenCalledWith('Authenticated successfully');
    expect(goto).toHaveBeenCalledWith('/');
  });

  it('routes to /login with code and redirect on ConfirmAuth', () => {
    send({
      code: 'xyz',
      redirect: 'https://x.example.com',
      type: UpdateMessageType.ConfirmAuth
    });
    expect(goto).toHaveBeenCalledWith(
      '/login?code=xyz&redirect=https%3A%2F%2Fx.example.com'
    );
  });

  it('routes to /login with code only when redirect is absent', () => {
    send({ code: 'xyz', type: UpdateMessageType.ConfirmAuth });
    expect(goto).toHaveBeenCalledWith('/login?code=xyz');
  });
});
