<script lang="ts">
  import { getUserInfo } from "$lib/backend/account/info.svelte";
  import {
    create_group,
    edit_group_permissions,
    edit_group_users,
    list_groups,
    list_groups_user,
  } from "$lib/backend/management/group.svelte";
  import {
    Permission,
    type Group,
    type UserInfo,
  } from "$lib/backend/management/types.svelte";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { createTable } from "$lib/components/table/helpers.svelte";
  import Table from "$lib/components/table/table.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { toast } from "svelte-sonner";
  import { columns } from "./table.svelte";
  import type { Row } from "@tanstack/table-core";
  import { RequestError } from "$lib/backend/types.svelte";

  let isLoading = $state(false);
  let groups: Group[] | undefined = $state();
  let users: UserInfo[] | undefined = $state();
  list_groups().then((group) => (groups = group));
  list_groups_user().then((user) => (users = user));
  let userInfo = $derived(getUserInfo());

  let name = $state("");
  let access_level = $state("");

  const updateGroups = async () => {
    await list_groups().then((group) => (groups = group));
  };

  const filterFn = (row: Row<Group>, id: string, filterValues: any) => {
    const info = [row.original.name, row.original.uuid]
      .filter(Boolean)
      .join(" ");

    let searchTerms = Array.isArray(filterValues)
      ? filterValues
      : [filterValues];
    return searchTerms.some((term) => info.includes(term.toLowerCase()));
  };

  let table = $state(
    createTable(
      [],
      columns([], [], Number.MAX_SAFE_INTEGER, updateGroups),
      filterFn,
    ),
  );

  $effect(() => {
    table = createTable(
      groups || [],
      columns(
        userInfo?.permissions || [],
        users || [],
        userInfo?.access_level ?? Number.MAX_SAFE_INTEGER,
        updateGroups,
        permissionSelect,
        userSelect,
      ),
      filterFn,
    );
  });

  const createGroup = async () => {
    let ret = await create_group(name, access_level);
    if (ret) {
      if (ret === RequestError.Unauthorized) {
        return "You can only use access levels above yours";
      } else if (ret === RequestError.Conflict) {
        return "Name already taken";
      } else {
        return "Error while creating group";
      }
    } else {
      await list_groups().then((group) => (groups = group));
      toast.success("Created Group");
      name = "";
      access_level = "";
    }
  };

  const permissionSelect = (user: string, value: Permission, add: boolean) => {
    edit_group_permissions(user, value, add).then((ret) => {
      if (ret) {
        toast.error("Error while updating");
      }
    });
  };

  const userSelect = (user: string, value: UserInfo, add: boolean) => {
    edit_group_users(user, value.uuid, add).then((ret) => {
      if (ret) {
        toast.error("Error while updating");
      }
    });
  };
</script>

<div class="space-y-3 m-4">
  <div class="ml-7 md:m-0">
    <h3 class="text-xl font-medium">Groups</h3>
    <p class="text-muted-foreground text-sm">
      Modify, create, delete groups and manage their permissions and members
      here
    </p>
  </div>
  <Table filterColumn="name" {table}>
    {#if userInfo?.permissions.includes(Permission.GroupCreate)}
      <FormDialog
        title="Create Group"
        description="Enter the details for the new group below"
        confirm="Create"
        trigger={{
          text: "Create Group",
          variant: "secondary",
          class: "ml-2",
        }}
        onsubmit={createGroup}
      >
        <Label for="name" class="sr-only">Name</Label>
        <Input
          id="name"
          placeholder="Name"
          required
          disabled={isLoading}
          bind:value={name}
        />
        <Label for="access_level" class="sr-only">Email</Label>
        <Input
          id="access_level"
          placeholder="Access Level"
          type="number"
          required
          disabled={isLoading}
          bind:value={access_level}
        />
      </FormDialog>
    {/if}
  </Table>
</div>
