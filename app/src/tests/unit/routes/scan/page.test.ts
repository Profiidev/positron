import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from '@testing-library/svelte';
import { goto } from '$app/navigation';

const checkPermissions = vi.fn();
const requestPermissions = vi.fn();
const scan = vi.fn();
const cancel = vi.fn();
vi.mock('@tauri-apps/plugin-barcode-scanner', () => ({
  Format: { QRCode: 'QR_CODE' },
  cancel,
  checkPermissions,
  requestPermissions,
  scan
}));

const toast = { error: vi.fn(), success: vi.fn(), warning: vi.fn() };
vi.mock('@profidev/pleiades/components/util/general', () => ({ toast }));

const Page = (await import('$routes/scan/+page.svelte')).default;

beforeEach(() => {
  checkPermissions.mockResolvedValue('granted');
  requestPermissions.mockResolvedValue('granted');
});
afterEach(() => vi.clearAllMocks());

describe('scan page', () => {
  it('routes to /login with code and redirect from a scanned url', async () => {
    scan.mockResolvedValue({
      content:
        'positron://x/login?code=abc&redirect=https%3A%2F%2Fx.example.com'
    });
    render(Page);
    await vi.waitFor(() =>
      expect(goto).toHaveBeenCalledWith(
        '/login?code=abc&redirect=https%3A%2F%2Fx.example.com'
      )
    );
  });

  it('routes to /login with no params when the qr has none', async () => {
    scan.mockResolvedValue({ content: 'positron://x/login' });
    render(Page);
    await vi.waitFor(() => expect(goto).toHaveBeenCalledWith('/login?'));
  });

  it('requests permission and continues when initially not granted', async () => {
    checkPermissions.mockResolvedValue('prompt');
    requestPermissions.mockResolvedValue('granted');
    scan.mockResolvedValue({ content: 'positron://x/login?code=abc' });
    render(Page);
    await vi.waitFor(() => expect(requestPermissions).toHaveBeenCalled());
    expect(goto).toHaveBeenCalledWith('/login?code=abc');
  });

  it('toasts and goes home when permission is denied', async () => {
    checkPermissions.mockResolvedValue('denied');
    requestPermissions.mockResolvedValue('denied');
    render(Page);
    await vi.waitFor(() =>
      expect(toast.error).toHaveBeenCalledWith('Permission denied')
    );
    expect(goto).toHaveBeenCalledWith('/');
    expect(scan).not.toHaveBeenCalled();
  });

  it('toasts when scanning fails', async () => {
    scan.mockRejectedValue(new Error('camera gone'));
    render(Page);
    await vi.waitFor(() =>
      expect(toast.error).toHaveBeenCalledWith('Failed to scan QR code')
    );
  });

  it('cancels the scanner on destroy', async () => {
    scan.mockResolvedValue({ content: 'positron://x/login?code=abc' });
    const { unmount } = render(Page);
    await vi.waitFor(() => expect(goto).toHaveBeenCalled());
    unmount();
    await vi.waitFor(() => expect(cancel).toHaveBeenCalled());
  });
});
