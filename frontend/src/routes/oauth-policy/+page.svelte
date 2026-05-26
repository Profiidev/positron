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
    deleteOAuthPolicy,
    type OAuthPolicyInfo,
    type UserInfo
  } from '$lib/client';

  const { data } = $props();

  let selected: OAuthPolicyInfo | undefined = $state();
  let deleteOpen = $state(false);
  let isLoading = $state(false);
  let user: UserInfo | undefined = $state();

  let canCreate = $derived(
    user?.permissions.includes(Permission.OAUTH_POLICY_EDIT) ?? false
  );

  $effect(() => {
    data.user.then((d) => {
      user = d;
    });
  });

  $effect(() => {
    if (data.error) {
      if (data.error === 'not_found') {
        toast.error('Policy not found');
      } else if (data.error === 'other') {
        toast.error('Failed to load policy');
      }

      const url = new URL(window.location.href);
      url.searchParams.delete('error');
      window.history.replaceState({}, '', url);
    }
  });

  const deleteItemConfirm = async () => {
    if (!selected) return;

    isLoading = true;
    let ret = await deleteOAuthPolicy({
      body: { uuid: selected.uuid }
    });
    isLoading = false;

    if (ret.error) {
      return { error: 'Failed to delete policy' };
    } else {
      toast.success(`Policy ${selected.name} deleted successfully`);
      invalidate((url) =>
        url.pathname.startsWith('/api/oauth_management/policy')
      );
    }
  };

  const startDeletePolicy = (item: OAuthPolicyInfo) => {
    selected = item;
    deleteOpen = true;
  };
</script>

<div class="flex max-h-screen flex-col p-4">
  <div class="ml-7 flex items-center md:m-0">
    <h3 class="text-xl font-medium">OAuth / Oidc Policies</h3>
    <Button
      class="ml-auto cursor-pointer"
      href="/oauth-policy/create"
      disabled={!canCreate}
    >
      <Plus />
      Create
    </Button>
  </div>
  <Table
    data={data.policies}
    {columns}
    class="mt-4 min-h-0 grow"
    columnData={{
      deletePolicy: startDeletePolicy,
      user
    }}
  />
</div>
<FormDialog
  title={`Delete Policy`}
  description={`Do you really want to delete the policy ${selected?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteItemConfirm}
  bind:open={deleteOpen}
  bind:isLoading
  schema={z.object({})}
/>
