<script lang="ts">
  import * as Avatar from '@profidev/pleiades/components/ui/avatar';
  import { avatarUrl } from '$lib/permissions.svelte';

  let {
    userId,
    username,
    class: className
  }: {
    class?: string;
    userId?: string;
    username?: string;
  } = $props();

  const initials = (name: string) =>
    name
      .split(' ')
      .map((part) => part[0])
      .slice(0, 2)
      .join('')
      .toUpperCase();
</script>

<Avatar.Root class={className}>
  <Avatar.Image src={userId ? `${avatarUrl}/${userId}` : ''} alt={username} />
  <Avatar.Fallback>{username ? initials(username) : '?'}</Avatar.Fallback>
</Avatar.Root>
