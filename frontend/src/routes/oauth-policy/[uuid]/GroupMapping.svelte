<script lang="ts">
  import Multiselect from '@profidev/pleiades/components/table/multiselect.svelte';
  import { Button } from '@profidev/pleiades/components/ui/form';
  import Trash from '@lucide/svelte/icons/trash';
  import Plus from '@lucide/svelte/icons/plus';
  import ChevronUp from '@lucide/svelte/icons/chevron-up';
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import { type OAuthPolicyContent, type SimpleGroupInfo } from '$lib/client';
  import { Input } from '@profidev/pleiades/components/ui/input';

  interface Props {
    groups?: SimpleGroupInfo[];
    mappings: OAuthPolicyContent[];
    disabled?: boolean;
  }

  let { groups, mappings = $bindable(), disabled }: Props = $props();

  let remainingGroups = $derived(
    groups?.filter(
      (group) => !mappings.some((mapping) => mapping.group_id === group.uuid)
    ) || []
  );

  let sortedMappings = $derived(mappings.toSorted((a, b) => a.index - b.index));
</script>

<h5>Group Mapping</h5>
<div class="my-2 flex grow flex-col">
  <div class="flex flex-col gap-2">
    {#each sortedMappings as mapping, i}
      <div class="grid grid-cols-[2rem_2rem_1fr_1fr_2rem] gap-2">
        <Button
          size="icon"
          type="button"
          variant="outline"
          class="min-w-8 cursor-pointer"
          disabled={i === 0 || disabled}
          onclick={() => {
            mappings = sortedMappings.map((m, j) => {
              if (j === i) {
                const newIndex = m.index - 1;
                if (newIndex < 0) return m; // Can't move up
                return { ...m, index: newIndex };
              } else if (j === i - 1) {
                const newIndex = m.index + 1;
                if (newIndex >= mappings.length) return m; // Can't move down
                return { ...m, index: newIndex };
              }
              return m;
            });
          }}
        >
          <ChevronUp />
        </Button>
        <Button
          size="icon"
          type="button"
          variant="outline"
          class="min-w-8 cursor-pointer"
          disabled={i === mappings.length - 1 || disabled}
          onclick={() => {
            mappings = sortedMappings.map((m, j) => {
              if (j === i) {
                const newIndex = m.index + 1;
                if (newIndex >= mappings.length) return m; // Can't move down
                return { ...m, index: newIndex };
              } else if (j === i + 1) {
                const newIndex = m.index - 1;
                if (newIndex < 0) return m; // Can't move up
                return { ...m, index: newIndex };
              }
              return m;
            });
          }}
        >
          <ChevronDown />
        </Button>
        <Multiselect
          data={[
            {
              name: mapping.group_name,
              uuid: mapping.group_id
            },
            ...remainingGroups
          ].map((group) => ({
            label: group.name,
            value: group.uuid
          }))}
          label="Group"
          single
          selected={[mapping.group_id]}
          {disabled}
          onSelectChange={(selected) => {
            if (selected.length > 0) {
              mappings = sortedMappings.map((m, j) =>
                j === i
                  ? {
                      ...m,
                      group_id: selected[0],
                      group_name:
                        groups?.find((g) => g.uuid === selected[0])?.name || ''
                    }
                  : m
              );
            }
          }}
        />
        <Input
          value={mapping.content}
          placeholder="Claim value"
          onchange={(e) => {
            const value = (e.target as HTMLInputElement).value;
            mappings = sortedMappings.map((m, j) =>
              j === i ? { ...m, content: value } : m
            );
          }}
          {disabled}
          required
        />
        <Button
          size="icon"
          type="button"
          variant="destructive"
          class="min-w-8 cursor-pointer"
          {disabled}
          onclick={() => {
            mappings = sortedMappings.filter(
              (m) => m.group_id !== mapping.group_id
            );
          }}
        >
          <Trash />
        </Button>
      </div>
    {/each}
  </div>
  {#if remainingGroups.length > 0}
    <Button
      type="button"
      size="icon"
      class="mt-2 cursor-pointer"
      {disabled}
      onclick={() => {
        mappings = [
          ...mappings,
          {
            group_id: remainingGroups[0].uuid,
            group_name: remainingGroups[0].name,
            content: '',
            index: mappings.length
          }
        ];
      }}
    >
      <Plus />
    </Button>
  {/if}
</div>
