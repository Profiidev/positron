<script lang="ts">
  import { Separator } from '@profidev/pleiades/components/ui/separator';
  import { Button } from '@profidev/pleiades/components/ui/button';
  import ArrowLeft from '@lucide/svelte/icons/arrow-left';
  import Trash from '@lucide/svelte/icons/trash';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import { DEFAULT_SCOPES, Permission } from '$lib/permissions.svelte';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import { z } from 'zod';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { goto } from '$app/navigation';
  import BaseForm from '@profidev/pleiades/components/form/base-form.svelte';
  import type { FormValue } from '@profidev/pleiades/components/form/types';
  import FormInput from '@profidev/pleiades/components/form/form-input.svelte';
  import Save from '@lucide/svelte/icons/save';
  import { Spinner } from '@profidev/pleiades/components/ui/spinner';
  import FormSelect from '@profidev/pleiades/components/form/form-select.svelte';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import {
    deleteOAuthScope,
    editOAuthScope,
    type OAuthScopeInfo,
    type SimpleOAuthPolicyInfo,
    type UserInfo
  } from '$lib/client';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { scopeSettings, formatData } from './schema.svelte.js';

  const { data } = $props();

  let deleteOpen = $state(false);
  let isLoading = $state(false);
  let user: UserInfo | undefined = $state();
  let scope: OAuthScopeInfo | undefined = $state();
  let form: BaseForm<typeof scopeSettings> | undefined = $state();

  let policies: SimpleOAuthPolicyInfo[] | undefined = $state();

  let readonly = $derived(
    !user?.permissions.includes(Permission.OAUTH_SCOPE_EDIT)
  );

  $effect(() => {
    data.scopeRes.then((res) => {
      if (!res.data) {
        if (res.response?.status === 404) {
          goto('/oauth-scope?error=not_found');
        } else {
          goto('/oauth-scope?error=other');
        }
        return;
      }

      scope = res.data;
      form?.setValue(formatData(scope));
    });
  });

  $effect(() => {
    data.user.then((d) => {
      user = d;
    });
  });

  $effect(() => {
    data.policiesPromise.then((d) => {
      policies = d;
    });
  });

  const deleteItemConfirm = async () => {
    if (!scope) return;
    isLoading = true;
    let ret = await deleteOAuthScope({
      body: { uuid: scope.uuid }
    });
    isLoading = false;

    if (ret.error) {
      return { error: 'Failed to delete scope' };
    } else {
      toast.success(`Scope ${scope.name} deleted successfully`);
      setTimeout(() => {
        goto('/oauth-scope');
      });
    }
  };

  const onsubmit = async (form: FormValue<typeof scopeSettings>) => {
    if (!scope) return;
    let res = await editOAuthScope({
      body: {
        ...form,
        uuid: scope.uuid
      }
    });

    if (res.error) {
      if (res.response?.status === 409) {
        return {
          error: 'This scope name is already in use',
          field: 'name'
        } as const;
      } else if (res.response?.status === 406) {
        return {
          error: 'This scope is already used',
          field: 'scope'
        } as const;
      } else {
        return { error: 'Failed to update scope' };
      }
    } else {
      toast.success(`Scope ${scope.name} updated successfully`);
      // do not trigger form reset
      return { error: '' };
    }
  };
</script>

<div class="flex h-full max-h-screen min-h-0 w-full flex-col space-y-6 p-4">
  <div class="mt-1! mb-0 ml-7 flex items-center md:m-0">
    <Button size="icon" variant="ghost" href="/oauth-scope" class="mr-2">
      <ArrowLeft class="size-5" />
    </Button>
    <h3 class="flex text-xl font-medium">
      User:
      {#if !scope}
        <Skeleton class="ml-2 h-7 w-20" />
      {:else}
        {scope.name}
      {/if}
    </h3>
    <Button
      class="ml-auto cursor-pointer"
      onclick={() => (deleteOpen = true)}
      variant="destructive"
      disabled={readonly || DEFAULT_SCOPES.includes(scope?.scope ?? '')}
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
      schema={scopeSettings}
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
                label="Scope Name"
                placeholder="Enter scope name"
                disabled={readonly}
              />
              <FormInput
                {...props}
                key="scope"
                label="Scope"
                placeholder="Enter scope"
                disabled={readonly ||
                  DEFAULT_SCOPES.includes(scope?.scope ?? '')}
              />
              <FormSelect
                {...props}
                key="policies"
                label="Policies"
                data={policies?.map((policy) => ({
                  label: policy.name,
                  value: policy.uuid
                })) ?? []}
                disabled={readonly}
              />
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
  title={`Delete Scope`}
  description={`Do you really want to delete the scope ${scope?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteItemConfirm}
  bind:open={deleteOpen}
  bind:isLoading
  schema={z.object({})}
/>
