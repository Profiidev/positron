<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Option } from ".";
  import { Button } from "../ui/button";
  import { Root, Trigger, Content } from "../ui/tooltip";

  let className: string | undefined = undefined;

  export let isCollapsed: boolean;
  export let options: Option[];
  export { className as class };
</script>

<div data-collapsed={isCollapsed} class={cn("group flex flex-col gap-4 py-2 data-[collapsed=true]:py-2", className)}>
  <nav class="grid gap-1 px-2 group-[[data-collapsed=true]]:justify-center group-[[data-collapsed=true]]:px-2">
    {#each options as option}
      {#if isCollapsed}
        <Root openDelay={0}>
          <Trigger asChild let:builder>
            <Button builders={[builder]} variant={option.selected ? "default": "ghost"} size="icon" class={cn("size-9", option.selected && "dark:bg-muted dark:text-muted-foreground dark:hover:bg-muted dark:hover:text-white")}>
              <svelte:component this={option.icon} class="size-4" aria-hidden="true" />
              <span class="sr-only">{option.title}</span>
            </Button>
          </Trigger>
          <Content side="right" class="flex items-center gap-4">
            {option.title}
            {#if option.label}
              <span class="text-muted-foreground ml-auto">
                {option.label}
              </span>
            {/if}
          </Content>
        </Root>
      {:else}
        <Button variant={option.selected ? "default": "ghost"} size="sm" class={cn("justify-start", { "dark:bg-muted dark:hover:bg-muted dark:text-white dark:hover:text-white": option.selected, })}>
          <svelte:component this={option.icon} class="size-4 mr-2" aria-hidden="true" />
          {option.title}
          {#if option.label}
            <span class={cn("ml-auto", { "text-background dark:text-white": option.selected })}>
              {option.label}
            </span>
          {/if}
        </Button>
      {/if}
    {/each}
  </nav>
</div>