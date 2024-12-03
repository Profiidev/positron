<script lang="ts" generics="T extends ZodRawShape">
  import * as Dialog from "$lib/components/ui/dialog";
  import type { Snippet, SvelteComponent } from "svelte";
  import { Button, type ButtonSize, type ButtonVariant } from "../ui/button";
  import { LoaderCircle } from "lucide-svelte";
  import { type SuperForm, type SuperValidated } from "sveltekit-superforms";
  import type { ZodRawShape } from "zod";
  import type { Error, FormSchema } from "./form.svelte";
  import Form from "./form.svelte";
  import { wait_for } from "$lib/util/interval.svelte";

  interface Props {
    title: string;
    description?: string;
    confirm: string;
    confirmVariant?: ButtonVariant;
    open?: boolean;
    class?: string;
    isLoading?: boolean;
    trigger?: {
      text?: string;
      variant?: ButtonVariant;
      class?: string;
      size?: ButtonSize;
      loadIcon?: boolean;
      disabled?: boolean;
    };
    onopen?: () => boolean | Promise<boolean>;
    onsubmit: (
      form: SuperValidated<T>,
    ) => Error | undefined | Promise<Error | undefined>;
    children?: Snippet<[{ form: SuperForm<T> }]>;
    triggerInner?: Snippet;
    form: FormSchema<T>;
  }

  let {
    title,
    description = "",
    confirm,
    confirmVariant = "default",
    open = $bindable(false),
    class: className,
    trigger,
    isLoading = $bindable(false),
    onopen = () => true,
    onsubmit,
    children,
    triggerInner,
    form: formInfo,
  }: Props = $props();

  let formComp: SvelteComponent | undefined = $state();
  let error = $state("");
  let formSetValue: undefined | ((value: T) => void) = $derived(
    formComp?.setValue,
  );

  const submit = async (form: SuperValidated<T>) => {
    let ret = await onsubmit(form);
    if (!ret) {
      open = false;
    }
    return ret;
  };

  export const openFn = async () => {
    isLoading = true;
    if (await onopen()) {
      open = true;
      error = "";
    }
    isLoading = false;
  };

  export const setValue = async (value: T) => {
    if (await wait_for(() => formSetValue !== undefined, 10, 5000)) {
      formSetValue!(value);
    }
  };
</script>

{#if trigger}
  <Button
    variant={trigger.variant}
    onclick={openFn}
    class={trigger.class}
    size={trigger.size}
    disabled={isLoading || trigger.disabled}
  >
    {#if isLoading && trigger.loadIcon}
      <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
    {/if}
    {trigger.text}
    {@render triggerInner?.()}
  </Button>
{/if}
<Dialog.Root bind:open>
  <Dialog.Content class={className}>
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
      <Dialog.Description>{description}</Dialog.Description>
    </Dialog.Header>
    <Form
      bind:this={formComp}
      bind:isLoading
      bind:error
      form={formInfo}
      {confirmVariant}
      onsubmit={submit}
      {confirm}
      {children}
    >
      {#snippet footer({ children })}
        <Dialog.Footer class="mt-4">
          {@render children()}
        </Dialog.Footer>
      {/snippet}
    </Form>
  </Dialog.Content>
</Dialog.Root>
