import '@testing-library/jest-dom/vitest';
import { afterAll } from 'vitest';

// Bits-ui's pin-input (TOTP) schedules detached setTimeouts (0/10/50ms) that
// Call `input.dispatchEvent(new Event('input'))`. If the last test's timers
// Fire while jsdom is tearing the file's environment down, the `Event` realm is
// Gone and they throw an unhandled "parameter 1 is not of type 'Event'". Wait
// Past the longest (50ms) timer so they flush while the environment is alive.
afterAll(async () => {
  await new Promise((resolve) => setTimeout(resolve, 60));
});

// Jsdom is missing a few browser APIs that the pleiades/bits-ui components
// Touch on mount. Stub them so component tests can render.
class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as never;

if (!globalThis.matchMedia) {
  globalThis.matchMedia = ((query: string) => ({
    addEventListener: () => {},
    addListener: () => {},
    dispatchEvent: () => false,
    matches: false,
    media: query,
    onchange: null,
    removeEventListener: () => {},
    removeListener: () => {}
  })) as never;
}

if (!Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = () => {};
}

// Jsdom has no DataTransfer, which sveltekit-superforms' `fileProxy` constructs
// To seed an empty FileList for file inputs.
if (!globalThis.DataTransfer) {
  class DataTransferStub {
    items: unknown[] = [];
    // A fresh file input exposes a genuine (empty) FileList, which jsdom
    // Requires when assigning to `input.files`.
    get files(): FileList {
      const input = document.createElement('input');
      input.type = 'file';
      return input.files!;
    }
  }
  globalThis.DataTransfer = DataTransferStub as never;
}

// Jsdom lacks the CSS Object Model `CSS.supports`, which bits-ui's pin-input
// (TOTP) probes on mount to detect iOS.
if (!globalThis.CSS) {
  globalThis.CSS = { supports: () => false } as never;
} else if (typeof globalThis.CSS.supports !== 'function') {
  globalThis.CSS.supports = () => false;
}

// Jsdom in this runner ships without localStorage; mode-watcher (theming) reads
// It at import time, so provide an in-memory implementation.
class MemoryStorage {
  readonly #map = new Map<string, string>();
  get length() {
    return this.#map.size;
  }
  clear() {
    this.#map.clear();
  }
  getItem(key: string) {
    return this.#map.get(key) ?? null;
  }
  key(index: number) {
    return [...this.#map.keys()][index] ?? null;
  }
  removeItem(key: string) {
    this.#map.delete(key);
  }
  setItem(key: string, value: string) {
    this.#map.set(key, value);
  }
}
if (!globalThis.localStorage) {
  globalThis.localStorage = new MemoryStorage() as never;
}
if (!globalThis.sessionStorage) {
  globalThis.sessionStorage = new MemoryStorage() as never;
}
