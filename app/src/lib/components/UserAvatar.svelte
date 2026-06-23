<script lang="ts">
  import { anyUserAvatar } from '$lib/commands/user.svelte';
  import * as Avatar from '@profidev/pleiades/components/ui/avatar';

  let {
    userId,
    username,
    class: className
  }: {
    class?: string;
    userId?: string;
    username?: string;
  } = $props();

  let avatar: string | undefined = $state();
  $effect(() => {
    userId;
    if (!userId) return;
    anyUserAvatar(userId).then((res) => {
      avatar = res;
    });
  });

  const initials = (name: string) =>
    name
      .split(' ')
      .map((part) => part[0])
      .slice(0, 2)
      .join('')
      .toUpperCase();
</script>

<Avatar.Root class={className}>
  <Avatar.Image src={avatar ?? ''} alt={username} />
  <Avatar.Fallback>{username ? initials(username) : '?'}</Avatar.Fallback>
</Avatar.Root>
