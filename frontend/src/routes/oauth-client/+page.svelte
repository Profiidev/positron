<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import Plus from '@lucide/svelte/icons/plus';
  import Table from '$lib/components/table/Table.svelte';
  import { columns } from './table.svelte';
  import { z } from 'zod';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { invalidate } from '$app/navigation';
  import { Permission } from '$lib/permissions.svelte';
  import {
    deleteOauthClient,
    type OAuthClientInfo,
    type UserInfo
  } from '$lib/client';

  const { data } = $props();

  let selected: OAuthClientInfo | undefined = $state();
  let deleteOpen = $state(false);
  let isLoading = $state(false);
  let user: UserInfo | undefined = $state();

  let canCreate = $derived(
    user?.permissions.includes(Permission.OAUTH_CLIENT_EDIT) ?? false
  );

  $effect(() => {
    data.user.then((d) => {
      user = d;
    });
  });

  $effect(() => {
    if (data.error) {
      if (data.error === 'not_found') {
        toast.error('Client not found');
      } else if (data.error === 'other') {
        toast.error('Failed to load client');
      }

      const url = new URL(window.location.href);
      url.searchParams.delete('error');
      window.history.replaceState({}, '', url);
    }
  });

  const deleteItemConfirm = async () => {
    if (!selected) return;

    isLoading = true;
    let ret = await deleteOauthClient({
      body: { client_id: selected.client_id }
    });
    isLoading = false;

    if (ret.error) {
      return { error: 'Failed to delete client' };
    } else {
      toast.success(`Client ${selected.name} deleted successfully`);
      invalidate((url) =>
        url.pathname.startsWith('/api/oauth_management/client')
      );
    }
  };

  const startDeleteClient = (item: OAuthClientInfo) => {
    selected = item;
    deleteOpen = true;
  };
</script>

<div class="flex max-h-screen flex-col p-4">
  <div class="ml-7 flex items-center md:m-0">
    <h3 class="text-xl font-medium">OAuth / Oidc Clients</h3>
    <Button
      class="ml-auto cursor-pointer"
      href="/oauth-client/create"
      disabled={!canCreate}
    >
      <Plus />
      Create
    </Button>
  </div>
  <Table
    data={data.clients}
    {columns}
    class="mt-4 min-h-0 grow"
    columnData={{
      deleteClient: startDeleteClient,
      user
    }}
    searchColumns={[
      'additional_redirect_uris',
      'client_id',
      'default_scope',
      'group_access',
      'name',
      'redirect_uri',
      'user_access'
    ]}
  />
</div>
<FormDialog
  title={`Delete Client`}
  description={`Do you really want to delete the client ${selected?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteItemConfirm}
  bind:open={deleteOpen}
  bind:isLoading
  schema={z.object({})}
/>
