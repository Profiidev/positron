import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import type * as SettingsCommands from '$lib/commands/settings.svelte';

const saveSettings = vi.fn();
vi.mock('$lib/commands/settings.svelte', async () => {
  const actual = await vi.importActual<typeof SettingsCommands>(
    '$lib/commands/settings.svelte'
  );
  return { ...actual, saveSettings };
});

const toast = { error: vi.fn(), success: vi.fn(), warning: vi.fn() };
vi.mock('@profidev/pleiades/components/util/general', () => ({ toast }));

// BaseForm's own error path toasts through 'svelte-sonner' directly (not the
// Pleiades util re-export the page uses for its success toast), so a failed
// Save has to be observed there.
const sonnerToast = { error: vi.fn() };
vi.mock('svelte-sonner', () => ({ toast: sonnerToast }));

const appSettingsState = { value: undefined as unknown };
vi.mock('$lib/updater/state.svelte', () => ({ appSettingsState }));

const Page = (await import('$routes/(app)/settings/+page.svelte')).default;

beforeEach(() => {
  appSettingsState.value = undefined;
});
afterEach(() => vi.clearAllMocks());

describe('settings page', () => {
  it('renders the width and height fields', () => {
    render(Page);
    expect(screen.getByLabelText('Width')).toBeInTheDocument();
    expect(screen.getByLabelText('Height')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
  });

  it('pre-fills width and height from the current app settings', () => {
    appSettingsState.value = {
      height: 700,
      horizontal_layout: 'Left',
      vertical_layout: 'Bottom',
      width: 900
    };
    render(Page);
    expect(screen.getByLabelText('Width')).toHaveValue(900);
    expect(screen.getByLabelText('Height')).toHaveValue(700);
  });

  it('saves the edited size and shows a success toast', async () => {
    appSettingsState.value = {
      height: 500,
      horizontal_layout: 'Center',
      vertical_layout: 'Center',
      width: 800
    };
    saveSettings.mockResolvedValue(true);
    render(Page);

    await fireEvent.input(screen.getByLabelText('Width'), {
      target: { value: '1000' }
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    await vi.waitFor(() =>
      expect(saveSettings).toHaveBeenCalledWith({
        height: 500,
        horizontal_layout: 'Center',
        vertical_layout: 'Center',
        width: 1000
      })
    );
    expect(toast.success).toHaveBeenCalledWith('Settings saved successfully.');
  });

  it('shows an error toast when saving fails', async () => {
    appSettingsState.value = {
      height: 500,
      horizontal_layout: 'Center',
      vertical_layout: 'Center',
      width: 800
    };
    saveSettings.mockResolvedValue(false);
    render(Page);

    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    await vi.waitFor(() =>
      expect(sonnerToast.error).toHaveBeenCalledWith('Failed to save settings')
    );
    expect(toast.success).not.toHaveBeenCalled();
  });
});
