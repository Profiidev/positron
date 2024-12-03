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
  import type { ZodObject, ZodRawShape } from "zod";
  import { FormButton } from "../ui/form";
  import { LoaderCircle } from "lucide-svelte";
  import type { ButtonVariant } from "../ui/button";

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
  interface Props {
    form: FormSchema<T>;
    onsubmit: (
      form: SuperValidated<T>,
    ) => Error | undefined | Promise<Error | undefined>;
    children?: Snippet<[{ form: SuperForm<T> }]>;
    footer: Snippet<[{ children: Snippet }]>;
    isLoading: boolean;
    confirmVariant?: ButtonVariant;
    confirm: string;
    error?: string;
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

  export const setValue = (value: T) => {
    let old = get(form.form);

    let newValue: T = {} as any;
    for (const key in old) {
      newValue[key] = value[key] ?? old[key];
    }

    form.form.set(newValue);
  };
</script>

<form method="POST" class="grid gap-3" use:enhance>
  {@render children?.({ form })}
  {#if error}
    <span class="text-destructive truncate text-sm">{error}</span>
  {/if}
  {@render footer({ children: formButton })}
</form>

{#snippet formButton()}
  <FormButton type="submit" disabled={isLoading} variant={confirmVariant}>
    {#if isLoading}
      <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
    {/if}
    {confirm}
  </FormButton>
{/snippet}
