import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import SimpleAvatar from '$lib/components/SimpleAvatar.svelte';

describe('SimpleAvatar', () => {
  it('applies the passed class to the avatar root', () => {
    const { container } = render(SimpleAvatar, {
      class: 'size-14',
      src: 'https://example.com/a.webp'
    });
    expect(container.querySelector('.size-14')).not.toBeNull();
  });

  it('renders the fallback while the image has not loaded', () => {
    render(SimpleAvatar, { class: 'size-14', src: '' });
    // Avatar.Image never reports a successful load under jsdom, so the "?"
    // Fallback is what is shown.
    expect(screen.getByText('?')).toBeInTheDocument();
  });
});
