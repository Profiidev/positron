<script lang="ts">
  import { Separator } from '@profidev/pleiades/components/ui/separator';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import ArrowLeft from '@lucide/svelte/icons/arrow-left';
  import Trash from '@lucide/svelte/icons/trash';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import { Permission } from '$lib/permissions.svelte';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import { z } from 'zod';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { goto } from '$app/navigation';
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';
  import Save from '@lucide/svelte/icons/save';
  import { Spinner } from '@profidev/pleiades/components/ui/spinner';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import {
    deleteOAuthPolicy,
    editOAuthPolicy,
    type OAuthPolicyContent,
    type OAuthPolicyInfo,
    type SimpleGroupInfo,
    type UserInfo
  } from '$lib/client';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { policySettings } from './schema.svelte.js';
  import GroupMapping from './GroupMapping.svelte';

  const { data } = $props();

  let deleteOpen = $state(false);
  let isLoading = $state(false);
  let user: UserInfo | undefined = $state();
  let policy: OAuthPolicyInfo | undefined = $state();
  let form: BaseForm<typeof policySettings> | undefined = $state();
  let mappings: OAuthPolicyContent[] = $state([]);

  let groups: SimpleGroupInfo[] | undefined = $state();

  let readonly = $derived(
    !user?.permissions.includes(Permission.OAUTH_POLICY_EDIT)
  );

  $effect(() => {
    data.policyRes.then((res) => {
      if (!res.data) {
        if (res.response?.status === 404) {
          goto('/oauth-policy?error=not_found');
        } else {
          goto('/oauth-policy?error=other');
        }
        return;
      }

      policy = res.data;
      form?.setValue(policy);
      mappings = policy.content;
    });
  });

  $effect(() => {
    data.user.then((d) => {
      user = d;
    });
  });

  $effect(() => {
    data.groupsPromise.then((d) => {
      groups = d;
    });
  });

  const deleteItemConfirm = async () => {
    if (!policy) return;
    isLoading = true;
    let ret = await deleteOAuthPolicy({
      body: { uuid: policy.uuid }
    });
    isLoading = false;

    if (ret.error) {
      return { error: 'Failed to delete policy' };
    } else {
      toast.success(`Policy ${policy.name} deleted successfully`);
      setTimeout(() => {
        goto('/oauth-policy');
      });
    }
  };

  const onsubmit = async (form: FormValue<typeof policySettings>) => {
    if (!policy) return;
    let res = await editOAuthPolicy({
      body: {
        ...form,
        uuid: policy.uuid,
        content: mappings
      }
    });

    if (res.error) {
      if (res.response?.status === 409) {
        return {
          error: 'This policy name is already in use',
          field: 'name'
        } as const;
      } else {
        return { error: 'Failed to update policy' };
      }
    } else {
      toast.success(`Policy ${policy.name} updated successfully`);
      // do not trigger form reset
      return { error: '' };
    }
  };
</script>

<div class="flex h-full max-h-screen min-h-0 w-full flex-col space-y-6 p-4">
  <div class="mt-1! mb-0 ml-7 flex items-center md:m-0">
    <Button size="icon" variant="ghost" href="/oauth-policy" class="mr-2">
      <ArrowLeft class="size-5" />
    </Button>
    <h3 class="flex text-xl font-medium">
      Policy:
      {#if !policy}
        <Skeleton class="ml-2 h-7 w-20" />
      {:else}
        {policy.name}
      {/if}
    </h3>
    <Button
      class="ml-auto cursor-pointer"
      onclick={() => (deleteOpen = true)}
      variant="destructive"
      disabled={readonly}
    >
      <Trash />
      Delete
    </Button>
  </div>
  <Separator class="my-4" />
  <div class="flex min-h-0 grow flex-col space-y-4 lg:space-y-0 lg:space-x-6">
    <h4 class="mb-2">Settings</h4>
    <BaseForm
      class="flex min-h-0 grow flex-col"
      schema={policySettings}
      {onsubmit}
      bind:this={form}
    >
      {#snippet children({ props })}
        <ScrollArea class="mt-2 min-h-0">
          <div
            class="grid min-h-0 grow grid-cols-1 gap-4 lg:grid-cols-[1fr_auto_1fr]"
          >
            <div>
              <FormInput
                {...props}
                key="name"
                label="Policy Name"
                placeholder="Enter policy name"
                disabled={readonly}
              />
              <FormInput
                {...props}
                key="claim"
                label="Claim Name"
                placeholder="Enter claim name"
                disabled={readonly}
              />
              <FormInput
                {...props}
                key="default"
                label="Default Value"
                placeholder="Enter default value"
                disabled={readonly}
              />
              <GroupMapping {groups} bind:mappings />
            </div>
          </div>
        </ScrollArea>
      {/snippet}
      {#snippet footer({
        isLoading,
        isError
      }: {
        isLoading: boolean;
        isError: boolean;
      })}
        <div class="mt-4 grid w-full grid-cols-1 gap-8 lg:grid-cols-2">
          <Button
            class="ml-auto cursor-pointer"
            type="submit"
            disabled={isLoading}
            variant={isError ? 'destructive' : undefined}
          >
            {#if isLoading}
              <Spinner />
            {:else if isError}
              <RotateCcw />
            {:else}
              <Save />
            {/if}
            {isError ? 'Retry' : 'Save Changes'}</Button
          >
        </div>
      {/snippet}
    </BaseForm>
  </div>
</div>
<FormDialog
  title={`Delete Policy`}
  description={`Do you really want to delete the policy ${policy?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteItemConfirm}
  bind:open={deleteOpen}
  bind:isLoading
  schema={z.object({})}
/>
