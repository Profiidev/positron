<script lang="ts" module>
  import type { Snippet } from "svelte";
  import { get } from "svelte/store";
  import {
    setError,
    superForm,
    type SuperForm,
    type SuperValidated,
  } from "sveltekit-superforms";
  import { zodClient } from "sveltekit-superforms/adapters";
  import type { ZodEffects, ZodObject, ZodRawShape } from "zod";
  import { FormButton } from "../ui/form";
  import { LoaderCircle } from "lucide-svelte";
  import type { ButtonVariant } from "../ui/button";
  import { cn } from "$lib/utils";

  export interface FormSchema<T extends ZodRawShape> {
    schema: ZodObject<T> | ZodEffects<ZodObject<T>>;
    form: SuperValidated<T, any, T>;
  }

  export interface Error {
    field?: string;
    error: string;
  }
</script>

<script lang="ts" generics="T extends ZodRawShape">
  interface Props {
    form: FormSchema<T>;
    onsubmit: (
      form: SuperValidated<T>,
    ) => Error | undefined | Promise<Error | undefined>;
    children?: Snippet<
      [{ props: { formData: SuperForm<T>; disabled: boolean } }]
    >;
    footer: Snippet<[{ children: Snippet<[{ className?: string }?]> }]>;
    isLoading: boolean;
    confirmVariant?: ButtonVariant;
    confirm: string;
    error?: string;
    class?: string;
  }

  let {
    form: formInfo,
    onsubmit,
    children,
    footer,
    isLoading = $bindable(false),
    confirmVariant = "default",
    confirm,
    error = $bindable(""),
    class: className,
  }: Props = $props();

  let form = superForm(formInfo.form, {
    validators: zodClient(formInfo.schema),
    SPA: true,
    onUpdate: async ({ form, cancel }) => {
      if (!form.valid) return;

      error = "";
      isLoading = true;

      let ret = await onsubmit(form);

      isLoading = false;
      if (ret) {
        if (ret.field) {
          setError(form, ret.field as "", ret.error, undefined);
        } else {
          if (ret.error !== "") error = ret.error;
          cancel();
        }
      }
    },
  });

  let { enhance } = form;

  export const setValue = (value: T) => {
    let old = get(form.form);

    let newValue: T = {} as any;
    for (const key in old) {
      newValue[key] = value[key] ?? old[key];
    }

    form.form.set(newValue);
  };
</script>

<form method="POST" class={cn("grid gap-3", className)} use:enhance>
  {@render children?.({ props: { formData: form, disabled: isLoading } })}
  {#if error}
    <span class="text-destructive truncate text-sm">{error}</span>
  {/if}
  {@render footer({ children: formButton })}
</form>

{#snippet formButton(props: { className?: string } | undefined)}
  {@const prop = { ...props }}
  <FormButton
    class={prop.className}
    type="submit"
    disabled={isLoading}
    variant={confirmVariant}
  >
    {#if isLoading}
      <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
    {/if}
    {confirm}
  </FormButton>
{/snippet}
