<script lang="ts">
  import {
    getPermissions,
    getPriority,
    updatePermissions,
  } from "$lib/backend/account/info.svelte";
  import type { Permission, User } from "$lib/backend/management/types.svelte";
  import {
    list,
    update_permissions,
  } from "$lib/backend/management/user.svelte";
  import { createTable } from "$lib/components/table/helpers.svelte";
  import Table from "$lib/components/table/table.svelte";
  import { toast } from "svelte-sonner";
  import { columns } from "./table.svelte";

  let users: User[] | undefined = $state();
  list().then((user) => (users = user));
  let table = $state(createTable([], columns([], Number.MAX_SAFE_INTEGER)));
  let allowed_permissions = $derived(getPermissions());
  let priority = $derived(getPriority());

  $effect(() => {
    table = createTable(
      users || [],
      columns(
        allowed_permissions || [],
        priority ?? Number.MAX_SAFE_INTEGER,
        permissionSelect,
      ),
    );
  });

  const permissionSelect = (user: string, value: Permission, add: boolean) => {
    update_permissions(user, value, add).then((ret) => {
      if (ret !== null) {
        toast.error("Error while updating");
      }
    });
    updatePermissions();
  };
</script>

<div class="space-y-3 m-4">
  <div class="ml-7 md:m-0">
    <h3 class="text-xl font-medium">Users</h3>
    <p class="text-muted-foreground text-sm">
      Modify, create, delete users and manage their permissions here
    </p>
  </div>
  <Table {table} />
</div>
