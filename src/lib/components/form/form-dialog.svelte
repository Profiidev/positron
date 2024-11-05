<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import type { Snippet } from "svelte";
  import { Button, type ButtonSize, type ButtonVariant } from "../ui/button";
  import { LoaderCircle } from "lucide-svelte";

  interface Props {
    title: string;
    description?: string;
    confirm: string;
    confirmVariant?: ButtonVariant;
    trigger?: { text?: string; variant?: ButtonVariant; class?: string, size?: ButtonSize };
    onopen?: () => boolean | Promise<boolean>;
    onsubmit: () => string | undefined | Promise<string | undefined>;
    children?: Snippet;
    triggerInner?: Snippet;
  }

  let {
    title,
    description = "",
    confirm,
    confirmVariant = "default",
    trigger,
    onopen = () => true,
    onsubmit,
    children,
    triggerInner,
  }: Props = $props();

  let error = $state("");
  let isLoading = $state(false);
  let open = $state(false);

  export const openFn = async () => {
    if (await onopen()) {
      error = "";
      open = true;
    }
  };

  const submit = async () => {
    error = "";
    isLoading = true;

    let ret = await onsubmit();

    isLoading = false;
    if (!ret) {
      open = false;
    } else {
      error = ret;
    }
  };
</script>

{#if trigger}
  <Button variant={trigger.variant} onclick={openFn} class={trigger.class} size={trigger.size}>
    {trigger.text}
    {@render triggerInner?.()}
  </Button>
{/if}
<Dialog.Root bind:open>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
      <Dialog.Description>{description}</Dialog.Description>
    </Dialog.Header>
    <form onsubmit={submit} class="grid gap-2">
      {@render children?.()}
      {#if error}
        <span class="text-destructive truncate text-sm">{error}</span>
      {/if}
      <Dialog.Footer class="mt-4">
        <Button type="submit" disabled={isLoading} variant={confirmVariant}>
          {#if isLoading}
            <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
          {/if}
          {confirm}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
