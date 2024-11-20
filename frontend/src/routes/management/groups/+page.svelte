<script lang="ts">
  import { getUserInfo } from "$lib/backend/account/info.svelte";
  import {
    create_group,
    delete_group,
    edit_group,
    list_groups,
    list_groups_user,
  } from "$lib/backend/management/group.svelte";
  import {
    getPermissionGroups,
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
  import Multiselect from "$lib/components/table/multiselect.svelte";

  let isLoading = $state(false);
  let groups: Group[] | undefined = $state();
  let users: UserInfo[] | undefined = $state();
  list_groups().then((group) => (groups = group));
  list_groups_user().then((user) => (users = user));
  let userInfo = $derived(getUserInfo());

  let name = $state("");
  let access_level = $state("");
  let group: Group | undefined = $state();
  let editOpen = $state(false);
  let deleteOpen = $state(false);

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
      columns(
        [],
        0,
        () => {},
        () => {},
      ),
      filterFn,
    ),
  );

  $effect(() => {
    table = createTable(
      groups || [],
      columns(
        userInfo?.permissions || [],
        userInfo?.access_level ?? 0,
        editGroup,
        deleteGroup,
      ),
      filterFn,
    );
  });

  const createGroup = async () => {
    let ret = await create_group(name, Number(access_level));
    if (ret) {
      if (ret === RequestError.Unauthorized) {
        return "You can only use access levels below yours";
      } else if (ret === RequestError.Conflict) {
        return "Name already taken";
      } else {
        return "Error while creating group";
      }
    } else {
      updateGroups();
      toast.success("Created Group");
      name = "";
      access_level = "";
    }
  };

  const updateGroups = async () => {
    await list_groups().then((group) => (groups = group));
  };

  const editGroup = (uuid: string) => {
    group = groups?.find((group) => group.uuid === uuid);
    editOpen = true;
  };

  const deleteGroup = (uuid: string) => {
    group = groups?.find((group) => group.uuid === uuid);
    deleteOpen = true;
  };

  const editGroupConfirm = async () => {
    if (!group) {
      return;
    }

    let ret = await edit_group(group);

    if (ret) {
      if (ret === RequestError.Unauthorized) {
        return "You can only set access level below yours";
      } else {
        return "Error while updating group";
      }
    } else {
      updateGroups();
      toast.success("Group updated");
    }
  };

  const deleteGroupConfirm = async () => {
    if (!group) {
      return;
    }

    let ret = await delete_group(group.uuid);

    if (ret) {
      return "Error while deleting Group";
    } else {
      updateGroups();
      toast.success("Group deleted");
    }
  };
</script>

<FormDialog
  title="Delete Group"
  description={`Do you really want to delete the group ${group?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deleteGroupConfirm}
  bind:open={deleteOpen}
></FormDialog>
<FormDialog
  title="Edit Group"
  description={`Edit the group info for ${group?.name} below`}
  confirm="Confirm"
  onsubmit={editGroupConfirm}
  bind:open={editOpen}
>
  {#if group && userInfo}
    <Label for="name">Name</Label>
    <Input id="name" placeholder="Name" bind:value={group.name} />
    <Label for="access_level">Access Level</Label>
    <Input
      id="access_level"
      placeholder="Access Level"
      type="number"
      bind:value={group.access_level}
    />
    <Label>Permissions</Label>
    <Multiselect
      label="Permissions"
      data={getPermissionGroups()}
      filter={(i) => userInfo.permissions.includes(i.label as Permission)}
      selected={group.permissions}
    />
    <Label>Users</Label>
    <Multiselect
      label="Users"
      data={users?.map((u) => ({
        label: u.name,
        value: u,
      })) || []}
      selected={group.users}
      display={(u) => u.name}
      compare={(a, b) => a.uuid === b.uuid}
    />
  {/if}
</FormDialog>
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
        <Label for="name">Name</Label>
        <Input
          id="name"
          placeholder="Name"
          required
          disabled={isLoading}
          bind:value={name}
        />
        <Label for="access_level">Access Level</Label>
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
