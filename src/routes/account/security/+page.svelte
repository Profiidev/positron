<script lang="ts">
  import { get_token, TokenType } from "$lib/auth/token.svelte";
  import { interval } from "$lib/util/interval.svelte";
  import Separator from "$lib/components/ui/separator/separator.svelte";
  import PasskeyList from "./passkey-list.svelte";
  import Totp_2fa from "./totp-2fa.svelte";
  import AccessConfirm from "./access-confirm.svelte";
  import Password from "./password.svelte";

  let specialAccessWatcher = interval(() => {
    return get_token(TokenType.SpecialAccess);
  }, 1000);
  let specialAccessValid = $state(false);
  $effect(() => {
    specialAccessValid = specialAccessWatcher.value !== undefined;
  });

  let cb = $state((_: boolean) => {});
  let accessOpen = $state(false);

  const requestAccess = async () => {
    return new Promise<boolean>((resolve) => {
      cb = resolve;
      accessOpen = true;
    });
  };
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-xl font-medium">Security</h3>
    <p class="text-muted-foreground text-sm">
      Change your authentication settings here
    </p>
  </div>
  <Separator />
  <div class="space-y-3">
    <h3 class="text-lg">Password</h3>
    <Password valid={specialAccessValid} {requestAccess} />
  </div>
  <div class="space-y-3">
    <h3 class="text-lg">Passkey</h3>
    <PasskeyList valid={specialAccessValid} {requestAccess} />
  </div>
  <div class="space-y-3">
    <h3 class="text-lg">Other 2FA Methods</h3>
    <div class="border p-2 rounded-xl">
      <Totp_2fa valid={specialAccessValid} {requestAccess} />
    </div>
  </div>
</div>
<AccessConfirm {cb} bind:open={accessOpen} />
