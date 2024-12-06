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
    label: string;
    compare?: (a: T, b: T) => boolean;
    disabled?: boolean;
  }

  let {
    data,
    selected = $bindable([]),
    filter = () => true,
    label,
    compare = (a, b) => a === b,
    disabled,
  }: Props = $props();

  const select = (value: T) => {
    let index = selected.findIndex((i) => compare(i, value));
    if (index !== -1) {
      selected.splice(index, 1);
    } else {
      selected.push(value);
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

  const find_element = (value: T): Item | undefined => {
    return filtered
      .map(
        (g) =>
          g.items
            .map((i) => (compare(i.value, value) ? i : undefined))
            .filter((i) => i !== undefined)[0],
      )
      .filter((i) => i !== undefined)[0];
  };
  $inspect(disabled).with(console.log)
</script>

<Popover.Root>
  <Popover.Trigger>
    {#snippet child({ props })}
      <Button
        variant="outline"
        {...props}
        role="combobox"
        class="h-10 text-wrap !opacity-100"
        {disabled}
      >
        {#if selected.length === 0}
          No {label}
        {:else}
          {selected.map((s) => find_element(s)?.label).join(", ")}
        {/if}
      </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content>
    <Command.Root>
      <Command.Input placeholder={`Search ${label.toLowerCase()}...`} />
      <ScrollArea orientation="vertical" class="h-full w-full">
        <Command.List class="overflow-visible">
          <Command.Empty>No {label} found</Command.Empty>
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
                      !selected.some((i) => compare(i, item.value)) &&
                        "text-transparent",
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
