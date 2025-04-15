<script lang="ts">
  import {
    user_settings,
    user_settings_update
  } from '$lib/backend/account/settings.svelte';
  import type { Settings } from '$lib/backend/account/types.svelte';
  import {
    Label,
    Separator,
    Switch,
    Tooltip
  } from 'positron-components/components/ui';
  import { onMount } from 'svelte';

  let settings: Settings | undefined = $state();

  onMount(async () => {
    settings = await user_settings();
  });

  const update = () => {
    if (settings) {
      user_settings_update(settings);
    }
  };
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-xl font-medium">OAuth / OpenID</h3>
    <p class="text-muted-foreground text-sm">
      Change your oauth / openid settings here
    </p>
  </div>
  <Separator />
  <div class="flex items-center">
    {#if settings}
      <div class="flex items-center">
        <Switch
          id="instant-redirect"
          bind:checked={settings.o_auth_instant_confirm}
          onCheckedChange={update}
        />
        <Tooltip.Provider>
          <Tooltip.Root>
            <Tooltip.Trigger class="ml-3">
              <Label for="instant-redirect">Instant Redirect</Label>
            </Tooltip.Trigger>
            <Tooltip.Content side="right">
              <p>
                Instantly redirects your browser without asking for confirmation
                when logging in with positron SSO
              </p>
            </Tooltip.Content>
          </Tooltip.Root>
        </Tooltip.Provider>
      </div>
    {/if}
  </div>
</div>
