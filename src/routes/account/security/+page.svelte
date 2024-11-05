<script lang="ts">
  import { get_token, TokenType } from "$lib/auth/token.svelte";
  import { interval } from "$lib/util/interval.svelte";
  import Separator from "$lib/components/ui/separator/separator.svelte";
  import AccessConfirm from "./access-confirm.svelte";
    import PasskeyList from "./passkey-list.svelte";
    import Other_2fa from "./other-2fa.svelte";

  let specialAccessWatcher = interval(() => {
    return get_token(TokenType.SpecialAccess);
  }, 1000);
  let specialAccessValid = $state(false);
  $effect(() => {
    specialAccessValid = specialAccessWatcher.value !== undefined;
  });
</script>

{#if specialAccessValid}
  <div class="space-y-6">
    <div>
      <h3 class="text-xl font-medium">Security</h3>
      <p class="text-muted-foreground text-sm">Change your login settings</p>
    </div>
    <Separator />
    <div class="space-y-3">
      <h3 class="text-lg">Passkey</h3>
      <PasskeyList />
    </div>
    <div class="space-y-3">
      <h3 class="text-lg">Other 2FA Methods</h3>
      <Other_2fa />
    </div>
  </div>
{:else}
  <AccessConfirm bind:specialAccessValid />
{/if}
