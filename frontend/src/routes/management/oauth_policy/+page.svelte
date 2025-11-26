<script lang="ts">
  import {
    create_policy,
    delete_policy,
    edit_policy
  } from '$lib/backend/management/oauth_policy.svelte';
  import {
    group_info_list,
    oauth_policy_list
  } from '$lib/backend/management/stores.svelte';
  import {
    Permission,
    type GroupInfo,
    type OAuthPolicyCreate
  } from '$lib/backend/management/types.svelte';
  import SimpleTable from 'positron-components/components/table/simple-table.svelte';
  import { Label } from 'positron-components/components/ui/label';
  import { Input } from 'positron-components/components/ui/input';
  import * as Select from 'positron-components/components/ui/select';
  import { Button } from 'positron-components/components/ui/button';
  import FormInput from 'positron-components/components/form/form-input.svelte';
  import { RequestError } from 'positron-components/backend';
  import { deepCopy } from 'positron-components/util/other.svelte';
  import { Plus, Trash } from '@lucide/svelte';
  import { createSchema, deleteSchema, editSchema } from './schema.svelte';
  import { columns } from './table.svelte';
  import { userData } from '$lib/backend/account/info.svelte';
  import type { FormValue } from 'positron-components/components/form/types';

  let policies = $derived(oauth_policy_list.value);
  let groups = $derived(group_info_list.value);
  let group: [GroupInfo, string][] = $state([]);
  let userInfo = $derived(userData.value?.[0]);

  const createItemFn = async (form: FormValue<typeof createSchema>) => {
    let policy: OAuthPolicyCreate = {
      group: group,
      ...form
    };
    return await create_policy(policy);
  };
</script>

<SimpleTable
  data={policies}
  filter_keys={['name', 'claim', 'uuid']}
  {columns}
  label="Policy"
  {createItemFn}
  editItemFn={edit_policy}
  deleteItemFn={delete_policy}
  toId={(item) => item.uuid}
  display={(item) => item?.name}
  title="Policies"
  description="Modify, create, delete policies and manage their settings here"
  {createSchema}
  {editSchema}
  {deleteSchema}
  errorMappings={{
    [RequestError.Conflict]: {
      field: 'name',
      error: 'Name already taken'
    }
  }}
  createButtonDisabled={!userInfo?.permissions.includes(
    Permission.OAuthClientCreate
  )}
  columnData={userInfo?.permissions ?? []}
>
  {#snippet editDialog({ props, item })}
    {@const groups_left_edit = deepCopy(
      groups?.filter((g) => !item.group.some((p) => p[0].uuid === g.uuid)) || []
    )}
    <FormInput label="Name" placeholder="Name" key="name" {...props} />
    <FormInput label="Claim" placeholder="Claim" key="claim" {...props} />
    <FormInput
      label="Default Content"
      placeholder="content"
      key="default"
      {...props}
    />
    <Label>Group Mappings</Label>
    {#each item.group as group}
      <div class="flex space-x-2">
        <Select.Root
          type="single"
          bind:value={group[0].uuid}
          allowDeselect={false}
          disabled={props.disabled}
          onValueChange={(v) =>
            (group[0].name = groups?.find((g) => g.uuid === v)?.name || '')}
        >
          <Select.Trigger class="w-full">{group[0].name}</Select.Trigger>
          <Select.Content>
            {#each [group[0], ...groups_left_edit] as option}
              <Select.Item value={option.uuid} label={option.name} />
            {/each}
          </Select.Content>
        </Select.Root>
        <Input
          placeholder="Content"
          bind:value={group[1]}
          required
          disabled={props.disabled}
        />
        <Button
          size="icon"
          variant="destructive"
          class="min-w-10"
          disabled={props.disabled}
          onclick={() => {
            if (!item) return;
            item.group = item.group.filter((g) => g[0].uuid !== group[0].uuid);
          }}
        >
          <Trash />
        </Button>
      </div>
    {/each}
    {#if groups_left_edit.length > 0}
      <Button
        size="icon"
        disabled={props.disabled}
        onclick={() => {
          if (!item) return;
          item.group.push([groups_left_edit[0], '']);
        }}
      >
        <Plus />
      </Button>
    {/if}
  {/snippet}
  {#snippet createDialog({ props })}
    {@const groups_left_create = deepCopy(
      groups?.filter((g) => !group.some((p) => p[0].uuid === g.uuid)) || []
    )}
    <FormInput label="Name" placeholder="Name" key="name" {...props} />
    <FormInput label="Claim" placeholder="Claim" key="claim" {...props} />
    <FormInput
      label="Default Content"
      placeholder="content"
      key="default"
      {...props}
    />
    <Label>Group Mappings</Label>
    {#each group as group_item}
      <div class="flex space-x-2">
        <Select.Root
          type="single"
          bind:value={group_item[0].uuid}
          allowDeselect={false}
          disabled={props.disabled}
          onValueChange={(v) =>
            (group_item[0].name =
              groups?.find((g) => g.uuid === v)?.name || '')}
        >
          <Select.Trigger class="w-full">{group_item[0].name}</Select.Trigger>
          <Select.Content>
            {#each [group_item[0], ...groups_left_create] as option}
              <Select.Item value={option.uuid} label={option.name} />
            {/each}
          </Select.Content>
        </Select.Root>
        <Input
          placeholder="Content"
          bind:value={group_item[1]}
          required
          disabled={props.disabled}
        />
        <Button
          size="icon"
          variant="destructive"
          class="min-w-10"
          disabled={props.disabled}
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
        disabled={props.disabled}
        onclick={() => {
          group.push([groups_left_create[0], '']);
        }}
      >
        <Plus />
      </Button>
    {/if}
  {/snippet}
</SimpleTable>
