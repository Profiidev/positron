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
  import FormSelect from '@profidev/pleiades/components/form/form-select.svelte';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import {
    deleteOauthClient,
    editOauthClient,
    regenerateSecretOauthClient,
    type OAuthClientInfo,
    type SimpleGroupInfo,
    type SimpleOAuthScopeInfo,
    type SimpleUserInfo,
    type UserInfo
  } from '$lib/client';
  import { Skeleton } from '@profidev/pleiades/components/ui/skeleton';
  import { clientSettings, formatData } from './schema.svelte.js';
  import FormTags from '$lib/components/FormTags.svelte';
  import { Label } from '@profidev/pleiades/components/ui/label';
  import { Input } from '@profidev/pleiades/components/ui/input';
  import { onMount } from 'svelte';
  import { CopyButton } from '@profidev/pleiades/components/ui-extra/copy-button';

  const { data } = $props();

  let deleteOpen = $state(false);
  let isLoading = $state(false);
  let regenerateOpen = $state(false);
  let siteUrl: string | undefined = $state();
  let secret: string | undefined = $state();
  let user: UserInfo | undefined = $state();
  let client: OAuthClientInfo | undefined = $state();
  let form: BaseForm<typeof clientSettings> | undefined = $state();

  let users: SimpleUserInfo[] | undefined = $state();
  let groups: SimpleGroupInfo[] | undefined = $state();
  let scopes: SimpleOAuthScopeInfo[] | undefined = $state();

  let readonly = $derived(
    !user?.permissions.includes(Permission.OAUTH_CLIENT_EDIT)
  );

  let oauthUrls = $derived([
    {
      name: 'Authorization URL',
      value: `${siteUrl}api/oauth/authorize`
    },
    {
      name: 'Token URL',
      value: `${siteUrl}api/oauth/token`
    },
    {
      name: 'Userinfo URL',
      value: `${siteUrl}api/oauth/user`
    },
    {
      name: 'Logout URL',
      value: `${siteUrl}api/oauth/logout/${client?.client_id}`
    },
    {
      name: 'Revoke URL',
      value: `${siteUrl}api/oauth/revoke`
    },
    {
      name: 'JWKs URL',
      value: `${siteUrl}api/oauth/jwks`
    },
    {
      name: 'OIDC Configuration URL',
      value: `${siteUrl}api/oauth/.well-known/openid-configuration`
    }
  ]);

  onMount(() => {
    let newSecret = sessionStorage.getItem('newSecret');
    if (newSecret) {
      secret = newSecret;
      sessionStorage.removeItem('newSecret');
    }
  });

  $effect(() => {
    data.clientRes.then((res) => {
      if (!res.data) {
        if (res.response?.status === 404) {
          goto('/oauth-client?error=not_found');
        } else {
          goto('/oauth-client?error=other');
        }
        return;
      }

      client = res.data;
      form?.setValue(formatData(client));
    });
  });

  $effect(() => {
    data.user.then((d) => {
      user = d;
    });
  });

  $effect(() => {
    data.usersPromise.then((d) => {
      users = d;
    });
  });

  $effect(() => {
    data.groupsPromise.then((d) => {
      groups = d;
    });
  });

  $effect(() => {
    data.scopesPromise.then((d) => {
      scopes = d;
    });
  });

  $effect(() => {
    data.sitePromise.then((d) => {
      siteUrl = d?.url;
    });
  });

  const deleteItemConfirm = async () => {
    if (!client) return;
    isLoading = true;
    let ret = await deleteOauthClient({
      body: { client_id: client.client_id }
    });
    isLoading = false;

    if (ret.error) {
      return { error: 'Failed to delete client' };
    } else {
      toast.success(`Client ${client.name} deleted successfully`);
      setTimeout(() => {
        goto('/oauth-client');
      });
    }
  };

  const regenerateConfirm = async () => {
    if (!client) return;
    isLoading = true;
    let res = await regenerateSecretOauthClient({
      path: { uuid: client.client_id }
    });
    isLoading = false;

    if (!res.data) {
      return { error: 'Failed to regenerate client secret' };
    } else {
      toast.success(`Client Secret ${client.name} regenerated successfully`);
      secret = res.data.secret;
    }
  };

  const onsubmit = async (form: FormValue<typeof clientSettings>) => {
    if (!client) return;
    let res = await editOauthClient({
      body: {
        ...form,
        client_id: client.client_id
      }
    });

    if (res.error) {
      if (res.response?.status === 409) {
        return {
          error: 'This client name is already in use',
          field: 'name'
        } as const;
      } else {
        return { error: 'Failed to update client' };
      }
    } else {
      toast.success(`Client ${client.name} updated successfully`);
      // do not trigger form reset
      return { error: '' };
    }
  };
</script>

<div class="flex h-full max-h-screen min-h-0 w-full flex-col space-y-6 p-4">
  <div class="mt-1! mb-0 ml-7 flex items-center md:m-0">
    <Button size="icon" variant="ghost" href="/oauth-client" class="mr-2">
      <ArrowLeft class="size-5" />
    </Button>
    <h3 class="flex text-xl font-medium">
      User:
      {#if !client}
        <Skeleton class="ml-2 h-7 w-20" />
      {:else}
        {client.name}
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
      schema={clientSettings}
      {onsubmit}
      bind:this={form}
    >
      {#snippet children({ props })}
        <ScrollArea class="mt-2 min-h-0">
          <div
            class="grid min-h-0 grow grid-cols-1 gap-4 lg:grid-cols-[1fr_auto_1fr]"
          >
            <div class="flex flex-col">
              <Label>Client ID</Label>
              <CopyButton
                class="my-2 h-8 grow justify-start"
                text={client?.client_id || ''}
                variant="outline"
              >
                <span class="truncate">{client?.client_id}</span>
              </CopyButton>
              {#if client?.confidential}
                <Label
                  >Client Secret
                  {#if secret}
                    <span class="text-destructive">
                      (Can not be viewed again!)
                    </span>
                  {/if}
                </Label>
                <div class="my-2 flex gap-2">
                  {#if secret}
                    <CopyButton
                      text={secret}
                      variant="outline"
                      class="h-8 grow justify-start"
                    >
                      <span class="truncate">{secret}</span>
                    </CopyButton>
                  {:else}
                    <Input
                      value="Can not be viewed."
                      readonly
                      class="text-destructive"
                    />
                  {/if}
                  <Button
                    variant="destructive"
                    class="cursor-pointer"
                    onclick={() => (regenerateOpen = true)}
                  >
                    <RotateCcw />
                    Regenerate
                  </Button>
                </div>
              {/if}
              <FormInput
                {...props}
                key="name"
                label="Client Name"
                placeholder="Enter client name"
                disabled={readonly}
              />
              <FormInput
                {...props}
                key="redirect_uri"
                label="Default Redirect URI"
                placeholder="https://example.com/callback"
                disabled={readonly}
              />
              <FormTags
                {...props}
                key="additional_redirect_uris"
                label="Additional Redirect URIs"
                validate={(val) => {
                  let res = z.url().safeParse(val);
                  if (!res.success) return undefined;
                  return val;
                }}
                disabled={readonly}
                placeholder="https://example.com/callback"
              />
              <FormSelect
                {...props}
                key="scope"
                label="Scopes"
                data={scopes?.map((scopes) => ({
                  label: scopes.name,
                  value: scopes.uuid
                })) ?? []}
                disabled={readonly}
              />
              <FormSelect
                {...props}
                key="user_access"
                label="User Access"
                data={users?.map((user) => ({
                  label: user.name,
                  value: user.id
                })) || []}
                disabled={readonly}
              />
              <FormSelect
                {...props}
                key="group_access"
                label="Group Access"
                data={groups?.map((group) => ({
                  label: group.name,
                  value: group.uuid
                })) || []}
                disabled={readonly}
              />
            </div>
            <Separator orientation="vertical" class="hidden lg:block" />
            <div class="flex min-w-0 flex-col">
              {#each oauthUrls as info}
                <Label for={info.name}>{info.name}</Label>
                <CopyButton
                  class="my-2 max-h-8 grow justify-start"
                  id={info.name}
                  text={info.value}
                  variant="outline"
                >
                  <span class="truncate">{info.value}</span>
                </CopyButton>
              {/each}
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
  title={`Delete Client`}
  description={`Do you really want to delete the client ${client?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteItemConfirm}
  bind:open={deleteOpen}
  bind:isLoading
  schema={z.object({})}
/>
<FormDialog
  title={`Regenerate Client Secret`}
  description={`Do you really want to regenerate the client secret for ${client?.name}?`}
  confirm="Regenerate"
  confirmVariant="destructive"
  onsubmit={regenerateConfirm}
  bind:open={regenerateOpen}
  bind:isLoading
  schema={z.object({})}
/>
