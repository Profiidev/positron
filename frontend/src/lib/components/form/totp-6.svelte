<script lang="ts">
  import * as InputOtp from "$lib/components/ui/input-otp";
  import * as Form from "$lib/components/ui/form";
  import type { SuperForm } from "sveltekit-superforms";

  interface Props {
    class: string | undefined;
    key: string;
    label: string;
    formData: SuperForm<any>;
  }

  let { class: className, key, formData: form, label }: Props = $props();

  let { form: formData } = $derived(form);
</script>

<Form.Field {form} name={key}>
  <Form.Control>
    {#snippet children({ props })}
      <Form.Label class={className}>{label}</Form.Label>
      <InputOtp.Root
        maxlength={6}
        bind:value={$formData[key]}
        class={className}
        autofocus
        {...props}
      >
        {#snippet children({ cells })}
          <InputOtp.Group>
            {#each cells.slice(0, 3) as cell}
              <InputOtp.Slot {cell} />
            {/each}
          </InputOtp.Group>
          <InputOtp.Separator />
          <InputOtp.Group>
            {#each cells.slice(3, 6) as cell}
              <InputOtp.Slot {cell} />
            {/each}
          </InputOtp.Group>
        {/snippet}
      </InputOtp.Root>
    {/snippet}
  </Form.Control>
  <Form.FieldErrors class={className} />
</Form.Field>
