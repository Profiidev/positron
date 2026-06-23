import { $, browser, expect } from '@wdio/globals';
import { MOCK_URL } from '../helpers/mock-server.js';
import {
  byButton,
  getRoute,
  resetAppData,
  waitForBodyText,
  waitForRoute
} from '../helpers/test-utils.js';

const INSTANCE_INPUT = 'input[placeholder="https://positron.example.com"]';

describe('Setup flow', () => {
  beforeEach(async () => {
    await resetAppData();
  });

  it('requires a URL before it can submit', async () => {
    // Empty submit fails the required/url schema, so the guard keeps us here.
    await byButton('Confirm').click();

    await browser.pause(500);
    expect(await getRoute()).toBe('/setup');
  });

  it('rejects an invalid URL and stays on setup', async () => {
    const input = $(INSTANCE_INPUT);
    await input.setValue('not-a-valid-url');
    await byButton('Confirm').click();

    // Client-side schema validation fails, so onsubmit never runs.
    await browser.pause(500);
    expect(await getRoute()).toBe('/setup');
  });

  it('shows an error when the server is unreachable', async () => {
    const input = $(INSTANCE_INPUT);
    await input.setValue('http://127.0.0.1:1');
    await byButton('Confirm').click();

    await waitForBodyText('Failed to connect to Positron');
    expect(await getRoute()).toBe('/setup');
  });

  // The form's success path runs a health check via the WebView's `fetch`. On
  // the Android emulator the WebView renderer process cannot reach the host mock
  // through the `adb reverse` tunnel (native `reqwest` can — see auth-flow.spec),
  // so this can't complete here. The setup -> /auth transition is covered via
  // the seeded path in auth.spec / routing.spec instead.
  it.skip('completes setup against a reachable server and moves to auth', async () => {
    const input = $(INSTANCE_INPUT);
    await input.setValue(MOCK_URL);
    await byButton('Confirm').click();

    await waitForRoute('/auth');
    expect(await getRoute()).toBe('/auth');
  });
});
