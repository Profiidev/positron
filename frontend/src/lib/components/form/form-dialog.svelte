<script lang="ts" module>
  export interface FormSchema<T extends ZodRawShape> {
    schema: ZodObject<T>;
    form: SuperValidated<T, any, T>;
  }

  export interface Error {
    field?: string;
    error: string;
  }
</script>

<script lang="ts" generics="T extends ZodRawShape">
  import * as Dialog from "$lib/components/ui/dialog";
  import type { Snippet } from "svelte";
  import { Button, type ButtonSize, type ButtonVariant } from "../ui/button";
  import { LoaderCircle } from "lucide-svelte";
  import { FormButton } from "../ui/form";
  import {
    setError,
    superForm,
    type SuperForm,
    type SuperValidated,
  } from "sveltekit-superforms";
  import { zodClient } from "sveltekit-superforms/adapters";
  import type { ZodObject, ZodRawShape } from "zod";
  import { get } from "svelte/store";

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

  let error = $state("");

  let form = superForm(formInfo.form, {
    validators: zodClient(formInfo.schema),
    SPA: true,
    onUpdate: async ({ form, cancel }) => {
      if (!form.valid) return;

      error = "";
      isLoading = true;

      let ret = await onsubmit(form);

      isLoading = false;
      if (!ret) {
        open = false;
      } else {
        if (ret.field) {
          form.errors;
          setError(form, ret.field as "", ret.error, undefined);
        } else if (ret.error !== "") {
          error = "";
          cancel();
        }
      }
    },
  });
  let { enhance } = form;

  const openFn = async () => {
    isLoading = true;
    if (await onopen()) {
      error = "";
      open = true;
    }
    isLoading = false;
  };

  export const setValue = (value: T) => {
    let old = get(form.form);

    let newValue: T = {} as any;
    for (const key in old) {
      newValue[key] = value[key] ?? old[key];
    }

    form.form.set(newValue);
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
    <form method="POST" class="grid gap-3" use:enhance>
      {@render children?.({ form })}
      {#if error}
        <span class="text-destructive truncate text-sm">{error}</span>
      {/if}
      <Dialog.Footer class="mt-4">
        <FormButton type="submit" disabled={isLoading} variant={confirmVariant}>
          {#if isLoading}
            <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
          {/if}
          {confirm}
        </FormButton>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
