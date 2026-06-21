import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach } from 'vitest';

export type CommandHandler = (
  cmd: string,
  payload?: Record<string, unknown>
) => unknown;

// Wires `@tauri-apps/api/core`'s `invoke` to the given handler. Returning a
// Value resolves the command; throwing (or returning a rejected promise)
// Rejects it, exercising the `try/catch` fallbacks in the command wrappers.
export const mockCommands = (handler: CommandHandler) => {
  mockIPC((cmd, payload) =>
    handler(cmd, payload as Record<string, unknown> | undefined)
  );
};

// Convenience for the common "single command returns X" / "single command
// Throws" cases.
export const mockCommand = (name: string, result: unknown) => {
  mockCommands((cmd) => {
    if (cmd === name) {
      return result;
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
};

export const mockCommandError = (name: string, message = 'failed') => {
  mockCommands((cmd) => {
    if (cmd === name) {
      throw new Error(message);
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
};

afterEach(() => clearMocks());
