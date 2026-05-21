<script lang="ts">
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { Badge } from '@profidev/pleiades/components/ui/badge';
  import { toast } from '@profidev/pleiades/components/util/general';
  import Totp_6 from '@profidev/pleiades/components/form/totp-6.svelte';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import { Clock9 } from '@lucide/svelte';
  import {
    totpAdd as totpAddSchema,
    totpRemove as totpRemoveSchema
  } from './schema.svelte';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import {
    totpFinishSetup,
    totpRemove,
    totpStartSetup,
    type UserInfo
  } from '$lib/client';

  interface Props {
    valid: boolean;
    requestAccess: () => Promise<boolean>;
    userInfo?: UserInfo;
  }

  let { valid, requestAccess, userInfo }: Props = $props();

  let totpQr = $state('');
  let totpCode = $state('');

  const startRemoveTotp = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }
    return true;
  };

  const removeTotp = async () => {
    let { response } = await totpRemove();

    if (response?.status !== 200) {
      return { error: 'Error while removing TOTP' };
    } else {
      toast.success('Remove successful', {
        description: 'TOTP was removed successfully from your account'
      });
    }
  };

  const startAddTotp = async () => {
    if (!valid) {
      if (!(await requestAccess())) {
        return false;
      }
    }

    totpQr = '';

    const fetch = async () => {
      let { data } = await totpStartSetup();

      if (data) {
        totpQr = data.qr;
        totpCode = data.code;
      }
    };

    fetch();
    return true;
  };

  const addTotp = async (form: FormValue<typeof totpAddSchema>) => {
    let { response } = await totpFinishSetup({
      body: {
        code: form.code
      }
    });

    if (response?.status !== 200) {
      if (response?.status === 401) {
        return { error: 'TOTP code invalid', field: 'code' };
      } else {
        return { error: 'Error while adding TOTP' };
      }
    } else {
      toast.success('Addition successful', {
        description: 'TOTP was added successfully to your account'
      });
    }
  };
</script>

<div class="rounded-xl border p-1">
  <div class="flex items-center">
    <div class="flex flex-col space-y-2 p-2">
      <div class="flex space-x-2">
        <Clock9 class="size-5" />
        <h4>TOTP</h4>
        {#if userInfo}
          {#if userInfo.totp_enabled}
            <Badge>Enabled</Badge>
          {:else}
            <Badge variant="destructive">Disabled</Badge>
          {/if}
        {:else}
          <Skeleton class="h-6 w-16 rounded-full" />
        {/if}
      </div>
    </div>
    {#if userInfo}
      {#if userInfo.totp_enabled}
        <FormDialog
          title="Remove TOTP"
          description="Do you really want to remove the TOTP 2FA method"
          confirm="Remove"
          confirmVariant="destructive"
          trigger={{
            text: 'Remove',
            variant: 'destructive',
            class: 'm-2 ml-auto',
            loadIcon: true
          }}
          onopen={startRemoveTotp}
          onsubmit={removeTotp}
          schema={totpRemoveSchema}
        ></FormDialog>
      {:else}
        <FormDialog
          title="Add TOTP"
          description="Scan the QR-Code below or enter the code manually and enter the TOTP code"
          confirm="Add"
          trigger={{ text: 'Add', class: 'm-2 ml-auto', loadIcon: true }}
          onopen={startAddTotp}
          onsubmit={addTotp}
          schema={totpAddSchema}
        >
          {#snippet children({ props })}
            <div class="flex flex-col items-center space-y-2">
              {#if totpQr !== ''}
                <img
                  class="size-60"
                  src={`data:image/png;base64, ${totpQr}`}
                  alt="QR"
                />
                <p class="text-muted-foreground">Or use the code</p>
                <p class="bg-muted rounded px-1">{totpCode}</p>
              {:else}
                <Skeleton class="size-60" />
                <p class="text-muted-foreground">Or use the code</p>
                <Skeleton class="h-6 w-80" />
              {/if}
              <Totp_6
                label="Confirm Code"
                key="code"
                {...props}
                class="flex justify-center"
              />
            </div>
          {/snippet}
        </FormDialog>
      {/if}
    {:else}
      <Skeleton class="m-2 ml-auto h-10 w-20" />
    {/if}
  </div>
</div>
