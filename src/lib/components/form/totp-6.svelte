<script lang="ts">
  import * as InputOtp from "$lib/components/ui/input-otp";

  interface Props {
    totp: string;
    class: string | undefined;
  }

  let { totp = $bindable(), class: className}: Props = $props();
</script>

<InputOtp.Root
  maxlength={6}
  bind:value={totp}
  class={className}
  required
  autofocus
>
  {#snippet children({ cells })}
    <InputOtp.Group>
      {#each cells.slice(0, 3) as cell}
        <InputOtp.Slot {cell} />
      {/each}
    </InputOtp.Group>
    <InputOtp.Separator />
    <InputOtp.Group>
      {#each cells.slice(3, 6) as cell}
        <InputOtp.Slot {cell} />
      {/each}
    </InputOtp.Group>
  {/snippet}
</InputOtp.Root>
