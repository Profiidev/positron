import { $, browser, expect } from '@wdio/globals';
import { getRoute, resetAppData, waitForRoute } from '../helpers/test-utils.js';

const INSTANCE_INPUT = 'input[placeholder="https://positron.example.com"]';

describe('App launch', () => {
  beforeEach(async () => {
    await resetAppData();
  });

  it('starts on the setup page for a fresh install', async () => {
    await waitForRoute('/setup');
    expect(await getRoute()).toBe('/setup');
  });

  it('renders the instance URL setup form', async () => {
    const input = $(INSTANCE_INPUT);
    await expect(input).toBeDisplayed();
  });

  it('exposes the Tauri IPC bridge to the webview', async () => {
    const hasBridge = await browser.execute(
      () =>
        typeof (
          window as unknown as { __TAURI_INTERNALS__?: { invoke?: unknown } }
        ).__TAURI_INTERNALS__?.invoke === 'function'
    );
    expect(hasBridge).toBe(true);
  });
});
