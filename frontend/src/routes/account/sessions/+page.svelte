<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import FormDialog from '@profidev/pleiades/components/form/form-dialog.svelte';
  import { toast } from '@profidev/pleiades/components/util/general';
  import LogOut from '@lucide/svelte/icons/log-out';
  import Table from '$lib/components/table/Table.svelte';
  import { revokeSession, type SessionInfo } from '$lib/client';
  import { z } from 'zod';
  import { columns } from './table.svelte';
  import { otherSessionCount, sessionDisplayName } from './session-utils';

  const { data } = $props();

  let sessions: SessionInfo[] | undefined = $state();
  let selected: SessionInfo | undefined = $state();
  let revokeOpen = $state(false);
  let revokeAllOpen = $state(false);
  let isLoading = $state(false);

  $effect(() => {
    data.sessions.then((list) => {
      sessions = list;
    });
  });

  const otherCount = $derived(sessions ? otherSessionCount(sessions) : 0);

  const startRevoke = (session: SessionInfo) => {
    selected = session;
    revokeOpen = true;
  };

  const revokeOne = async () => {
    if (!selected) {
      return;
    }

    isLoading = true;
    const { error } = await revokeSession({
      body: { id: selected.id }
    });
    isLoading = false;

    if (error) {
      return { error: 'Failed to revoke session' };
    }
    toast.success('Session revoked');
  };

  const revokeAllOthers = async () => {
    const targets = sessions?.filter((session) => !session.current) ?? [];
    if (targets.length === 0) {
      return;
    }

    isLoading = true;
    const results = await Promise.all(
      targets.map((session) => revokeSession({ body: { id: session.id } }))
    );
    isLoading = false;

    if (results.some(({ error }) => error)) {
      return { error: 'Failed to revoke one or more sessions' };
    }
    toast.success('Sessions revoked');
  };
</script>

<div class="space-y-4">
  <div
    class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between"
  >
    <h4 class="text-lg font-medium">Sessions</h4>
    <Button
      variant="outline"
      class="border-destructive text-destructive hover:text-destructive cursor-pointer sm:shrink-0"
      disabled={!sessions || otherCount === 0}
      onclick={() => (revokeAllOpen = true)}
    >
      <LogOut class="size-4" />
      Revoke all other sessions
    </Button>
  </div>

  <Table data={sessions} {columns} columnData={{ revoke: startRevoke }} />
</div>
<FormDialog
  title="Revoke session"
  description={`This will sign out "${selected ? sessionDisplayName(selected.name, selected.is_app) : ''}" from your account.`}
  confirm="Revoke"
  confirmVariant="destructive"
  onsubmit={revokeOne}
  bind:open={revokeOpen}
  bind:isLoading
  schema={z.object({})}
/>

<FormDialog
  title="Revoke all other sessions"
  description={`This will sign out ${otherCount} other session${otherCount === 1 ? '' : 's'} from your account.`}
  confirm="Revoke all"
  confirmVariant="destructive"
  onsubmit={revokeAllOthers}
  bind:open={revokeAllOpen}
  bind:isLoading
  schema={z.object({})}
/>
