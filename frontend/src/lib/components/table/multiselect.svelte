<script lang="ts">
  import * as Popover from "$lib/components/ui/popover";
  import * as Command from "$lib/components/ui/command";
  import { Button } from "../ui/button";
  import { cn } from "$lib/utils";
  import { Check } from "lucide-svelte";
  import { ScrollArea } from "../ui/scroll-area";

  type T = $$Generic;

  interface Group {
    label: string;
    items: Item[];
  }

  const isGroups = (object: any[]): object is Group[] => {
    return (
      object.length > 0 &&
      typeof object[0] === "object" &&
      object[0] !== null &&
      "items" in object[0]
    );
  };

  interface Item {
    label: string;
    value: T;
  }

  interface Props {
    data: Group[] | Item[];
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

  let filtered = $derived.by(() => {
    if (isGroups(data)) {
      return data
        .map((g) => {
          g.items = g.items.filter(filter);
          return g;
        })
        .filter((g) => g.items.length > 0);
    } else {
      return [
        {
          label: "",
          items: data.filter(filter),
        },
      ];
    }
  });
</script>

<Popover.Root>
  <Popover.Trigger>
    {#snippet child({ props })}
      <Button
        variant={disabled ? "ghost" : "outline"}
        {...props}
        role="combobox"
        class="h-full text-wrap !opacity-100"
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
      <ScrollArea orientation="vertical" class="h-full w-full">
        <Command.List class="overflow-visible">
          <Command.Empty>No permissions found</Command.Empty>
          {#each filtered as group}
            <Command.Group heading={group.label}>
              {#each group.items as item}
                <Command.Item
                  value={item.label}
                  onSelect={() => select(item.value)}
                >
                  <Check
                    class={cn(
                      "mr-2 size-4",
                      !selected.includes(item.value) && "text-transparent",
                    )}
                  />
                  {item.label}
                </Command.Item>
              {/each}
            </Command.Group>
          {/each}
        </Command.List>
      </ScrollArea>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
