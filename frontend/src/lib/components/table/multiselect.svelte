<script lang="ts">
  import * as Popover from "$lib/components/ui/popover";
  import * as Command from "$lib/components/ui/command";
  import { Button } from "../ui/button";
  import { cn } from "$lib/utils";
  import { Check } from "lucide-svelte";

  type T = $$Generic;

  interface Item {
    label: string;
    value: T;
  }

  interface Props {
    data: Item[];
    filter?: (data: Item) => boolean;
    selected: T[];
    disabled?: boolean;
    onSelect?: (selected: T, add: boolean) => void;
  }

  let {
    data,
    selected = $bindable(),
    filter = () => true,
    disabled = true,
    onSelect = () => {},
  }: Props = $props();

  const select = (value: T) => {
    if (selected.includes(value)) {
      selected = selected.filter((e) => e !== value);
      onSelect(value, false);
    } else {
      selected.push(value);
      onSelect(value, true);
    }
  };
</script>

<Popover.Root>
  <Popover.Trigger>
    {#snippet child({ props })}
      <Button
        variant={disabled ? "ghost" : "outline"}
        {...props}
        role="combobox"
        class="w-full h-full text-wrap !opacity-100"
        {disabled}
      >
        {#if selected.length === 0}
          No permissions
        {:else}
          {selected.join(", ")}
        {/if}
      </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content>
    <Command.Root>
      <Command.Input placeholder="Search permissions..." />
      <Command.List>
        <Command.Empty>No permissions found</Command.Empty>
        <Command.Group>
          {#each data.filter(filter) as entry}
            <Command.Item
              value={entry.label}
              onSelect={() => select(entry.value)}
            >
              <Check
                class={cn(
                  "mr-2 size-4",
                  !selected.includes(entry.value) && "text-transparent",
                )}
              />
              {entry.label}
            </Command.Item>
          {/each}
        </Command.Group>
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
