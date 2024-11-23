<script lang="ts">
  import { userData } from "$lib/backend/account/info.svelte";
  import {
    Permission,
    type GroupInfo,
    type OAuthPolicy,
  } from "$lib/backend/management/types.svelte";
  import { createTable } from "$lib/components/table/helpers.svelte";
  import type { Row } from "@tanstack/table-core";
  import { columns } from "./table.svelte";
  import { RequestError } from "$lib/backend/types.svelte";
  import { toast } from "svelte-sonner";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
  import { Label } from "$lib/components/ui/label";
  import { Input } from "$lib/components/ui/input";
  import Table from "$lib/components/table/table.svelte";
  import {
    create_policy,
    delete_policy,
    edit_policy,
  } from "$lib/backend/management/oauth_policy.svelte";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Plus, Trash } from "lucide-svelte";
  import { deepCopy } from "$lib/util/other.svelte";
  import {
    group_info_list,
    oauth_policy_list,
  } from "$lib/backend/management/stores.svelte";

  let isLoading = $state(false);
  let policies = $derived(oauth_policy_list.value);
  let groups = $derived(group_info_list.value);
  let userInfo = $derived(userData.value?.[0]);

  let policy: OAuthPolicy | undefined = $state();
  let editOpen = $state(false);
  let deleteOpen = $state(false);
  let name = $state("");
  let claim = $state("");
  let default_content = $state("");
  let group: [GroupInfo, string][] = $state([]);

  let groups_left_edit = $derived(
    deepCopy(
      groups?.filter(
        (g) => policy && !policy.group.some((p) => p[0].uuid === g.uuid),
      ) || [],
    ),
  );
  let groups_left_create = $derived(
    deepCopy(
      groups?.filter((g) => !group.some((p) => p[0].uuid === g.uuid)) || [],
    ),
  );

  const filterFn = (row: Row<OAuthPolicy>, id: string, filterValues: any) => {
    const info = [row.original.name, row.original.claim, row.original.uuid]
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
        () => {},
        () => {},
      ),
      filterFn,
    ),
  );

  $effect(() => {
    table = createTable(
      policies || [],
      columns(userInfo?.permissions || [], editPolicy, deletePolicy),
      filterFn,
    );
  });

  const createPolicy = async () => {
    let ret = await create_policy({
      name,
      claim,
      default: default_content,
      group,
    });
    if (ret) {
      if (ret === RequestError.Conflict) {
        return "Name already taken";
      } else {
        return "Error while creating policy";
      }
    } else {
      toast.success("Created Policy");
      name = "";
      claim = "";
      default_content = "";
      group = [];
    }
  };

  const editPolicy = (uuid: string) => {
    let p = policies?.find((policy) => policy.uuid === uuid);
    policy = deepCopy(p);
    editOpen = true;
  };

  const deletePolicy = (uuid: string) => {
    policy = policies?.find((policy) => policy.uuid === uuid);
    deleteOpen = true;
  };

  const editPolicyConfirm = async () => {
    if (!policy) {
      return;
    }

    let ret = await edit_policy(policy);

    if (ret) {
      if (ret === RequestError.Conflict) {
        return "Name already taken";
      } else {
        return "Error while updating policy";
      }
    } else {
      toast.success("Policy updated");
    }
  };

  const deletePolicyConfirm = async () => {
    if (!policy) {
      return;
    }

    let ret = await delete_policy(policy.uuid);

    if (ret) {
      return "Error while deleting policy";
    } else {
      toast.success("Policy deleted");
    }
  };
</script>

<FormDialog
  title="Delete Policy"
  description={`Do you really want to delete the policy ${policy?.name}?`}
  confirm="Delete"
  confirmVariant="destructive"
  onsubmit={deletePolicyConfirm}
  bind:open={deleteOpen}
></FormDialog>
<FormDialog
  title="Edit Policy"
  description={`Edit the policy info for ${policy?.name} below`}
  confirm="Confirm"
  onsubmit={editPolicyConfirm}
  bind:open={editOpen}
>
  {#if policy && userInfo && groups}
    <Label for="name">Name</Label>
    <Input id="name" placeholder="Name" bind:value={policy.name} />
    <Label for="claim">Claim</Label>
    <Input id="claim" placeholder="Claim" bind:value={policy.claim} />
    <Label for="default">Default Content</Label>
    <Input
      id="default"
      placeholder="Default Content"
      required
      bind:value={policy.default}
    />
    <Label>Group Mappings</Label>
    {#each policy.group as group}
      <div class="flex space-x-2">
        <Select.Root
          type="single"
          bind:value={group[0].uuid}
          allowDeselect={false}
          onValueChange={(v) =>
            (group[0].name = groups?.find((g) => g.uuid === v)?.name || "")}
        >
          <Select.Trigger>{group[0].name}</Select.Trigger>
          <Select.Content>
            {#each [group[0], ...groups_left_edit] as option}
              <Select.Item value={option.uuid} label={option.name} />
            {/each}
          </Select.Content>
        </Select.Root>
        <Input placeholder="Content" bind:value={group[1]} />
        <Button
          size="icon"
          variant="destructive"
          class="min-w-10"
          onclick={() => {
            if (!policy) return;
            policy.group = policy.group.filter(
              (g) => g[0].uuid !== group[0].uuid,
            );
          }}
        >
          <Trash />
        </Button>
      </div>
    {/each}
    {#if groups_left_edit.length > 0}
      <Button
        size="icon"
        onclick={() => {
          if (!policy) return;
          policy.group.push([groups_left_edit[0], ""]);
        }}
      >
        <Plus />
      </Button>
    {/if}
  {/if}
</FormDialog>
<div class="space-y-3 m-4">
  <div class="ml-7 md:m-0">
    <h3 class="text-xl font-medium">Policies</h3>
    <p class="text-muted-foreground text-sm">
      Modify, create, delete policies and manage their settings here
    </p>
  </div>
  <Table filterColumn="name" {table}>
    {#if userInfo?.permissions.includes(Permission.OAuthClientCreate)}
      <FormDialog
        title="Create Policy"
        description="Enter the details for the new policy below"
        confirm="Create"
        trigger={{
          text: "Create Policy",
          variant: "secondary",
          class: "ml-2",
          loadIcon: true,
        }}
        onsubmit={createPolicy}
      >
        <Label for="name">Name</Label>
        <Input
          id="name"
          placeholder="Name"
          required
          disabled={isLoading}
          bind:value={name}
        />
        <Label for="claim">Claim</Label>
        <Input
          id="claim"
          placeholder="Claim"
          required
          disabled={isLoading}
          bind:value={claim}
        />
        <Label for="default">Default Content</Label>
        <Input
          id="default"
          placeholder="Default Content"
          required
          bind:value={default_content}
        />
        <Label>Group Mappings</Label>
        {#each group as group_item}
          <div class="flex space-x-2">
            <Select.Root
              type="single"
              bind:value={group_item[0].uuid}
              allowDeselect={false}
              onValueChange={(v) =>
                (group_item[0].name =
                  groups?.find((g) => g.uuid === v)?.name || "")}
            >
              <Select.Trigger>{group_item[0].name}</Select.Trigger>
              <Select.Content>
                {#each [group_item[0], ...groups_left_create] as option}
                  <Select.Item value={option.uuid} label={option.name} />
                {/each}
              </Select.Content>
            </Select.Root>
            <Input placeholder="Content" bind:value={group_item[1]} />
            <Button
              size="icon"
              variant="destructive"
              class="min-w-10"
              onclick={() => {
                group = group.filter((g) => g[0].uuid !== group_item[0].uuid);
              }}
            >
              <Trash />
            </Button>
          </div>
        {/each}
        {#if groups_left_create.length > 0}
          <Button
            size="icon"
            onclick={() => {
              group.push([groups_left_create[0], ""]);
            }}
          >
            <Plus />
          </Button>
        {/if}
      </FormDialog>
    {/if}
  </Table>
</div>
