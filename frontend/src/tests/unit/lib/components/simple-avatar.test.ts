import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import SimpleAvatar from '$lib/components/SimpleAvatar.svelte';

describe('SimpleAvatar', () => {
  it('renders the "?" fallback', () => {
    render(SimpleAvatar, { class: 'size-8', src: '' });
    expect(screen.getByText('?')).toBeInTheDocument();
  });

  it('applies the provided class to the avatar root', () => {
    const { container } = render(SimpleAvatar, {
      class: 'custom-class',
      src: ''
    });
    expect(container.querySelector('.custom-class')).not.toBeNull();
  });
});
