<script lang="ts">
  import Multiselect from '@profidev/pleiades/components/table/multiselect.svelte';
  import { Button } from '@profidev/pleiades/components/ui/form';
  import Trash from '@lucide/svelte/icons/trash';
  import Plus from '@lucide/svelte/icons/plus';
  import GripVertical from '@lucide/svelte/icons/grip-vertical';
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

  // pointer-based drag so the row only moves vertically
  let drag: { index: number; startY: number; dy: number; step: number } | null =
    $state(null);

  const move = (from: number, to: number) => {
    const next = [...sortedMappings];
    next.splice(to, 0, ...next.splice(from, 1));
    mappings = next.map((m, index) => ({ ...m, index }));
  };

  const onpointermove = (e: PointerEvent) => {
    if (!drag) return;
    drag.dy = e.clientY - drag.startY;
    // swap with neighbour once dragged past half of its slot
    if (drag.dy > drag.step / 2 && drag.index < sortedMappings.length - 1) {
      move(drag.index, ++drag.index);
      drag.startY += drag.step;
      drag.dy -= drag.step;
    } else if (drag.dy < -drag.step / 2 && drag.index > 0) {
      move(drag.index, --drag.index);
      drag.startY -= drag.step;
      drag.dy += drag.step;
    }
  };
</script>

<!-- window, not pointer capture: reordering moves the row node and drops capture -->
<svelte:window
  {onpointermove}
  onpointerup={() => (drag = null)}
  onpointercancel={() => (drag = null)}
/>

<h5>Group Mapping</h5>
<div class="my-2 flex grow flex-col">
  <div class="flex flex-col gap-2" role="list">
    {#each sortedMappings as mapping, i (mapping.group_id)}
      <div
        class="bg-background grid grid-cols-[2rem_1fr_1fr_2rem] gap-2 rounded-md border p-2 {drag?.index ===
        i
          ? 'relative z-10 shadow-lg'
          : ''}"
        style:transform={drag?.index === i
          ? `translateY(${drag.dy}px)`
          : undefined}
        role="listitem"
      >
        <div
          role="button"
          tabindex={disabled ? -1 : 0}
          aria-label="Drag to reorder, or use arrow keys"
          class="flex touch-none items-center justify-center {disabled
            ? 'opacity-50'
            : drag
              ? 'cursor-grabbing'
              : 'cursor-grab'}"
          onpointerdown={(e) => {
            if (disabled || e.button !== 0) return;
            // no text selection while dragging
            e.preventDefault();
            const handle = e.currentTarget as HTMLElement;
            // row height + list gap
            const row = handle.parentElement!;
            const step =
              row.getBoundingClientRect().height +
              parseFloat(getComputedStyle(row.parentElement!).rowGap);
            drag = { index: i, startY: e.clientY, dy: 0, step };
          }}
          onkeydown={(e) => {
            if (disabled) return;
            if (e.key === 'ArrowUp' && i > 0) move(i, i - 1);
            else if (e.key === 'ArrowDown' && i < sortedMappings.length - 1)
              move(i, i + 1);
            else return;
            e.preventDefault();
          }}
        >
          <GripVertical class="size-4" />
        </div>
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
