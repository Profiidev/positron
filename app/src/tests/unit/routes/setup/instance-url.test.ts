import { describe, expect, it, vi } from 'vitest';
import { createRawSnippet } from 'svelte';
import { render, screen } from '@testing-library/svelte';
import InstanceUrl from '$routes/setup/InstanceUrl.svelte';

const footer = createRawSnippet(() => ({
  render: () => '<div data-testid="footer">footer</div>'
}));

describe('InstanceUrl', () => {
  it('renders the labelled instance url field and footer', () => {
    render(InstanceUrl, {
      footer,
      isLoading: false,
      onsubmit: vi.fn()
    });
    expect(screen.getByText('Instance URL')).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText('https://positron.example.com')
    ).toBeInTheDocument();
    expect(screen.getByTestId('footer')).toBeInTheDocument();
  });

  it('marks the input readonly when requested', () => {
    render(InstanceUrl, {
      footer,
      isLoading: false,
      onsubmit: vi.fn(),
      readonly: true
    });
    expect(
      screen.getByPlaceholderText('https://positron.example.com')
    ).toHaveAttribute('readonly');
  });
});
