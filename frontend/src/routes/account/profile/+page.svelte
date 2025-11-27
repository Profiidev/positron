<script lang="ts">
  import { Button } from 'positron-components/components/ui/button';
  import { Input } from 'positron-components/components/ui/input';
  import { Label } from 'positron-components/components/ui/label';
  import { Separator } from 'positron-components/components/ui/separator';
  import { Skeleton } from 'positron-components/components/ui/skeleton';
  import { toast } from 'positron-components/components/util/general';
  import BaseForm from 'positron-components/components/form/base-form.svelte';
  import FormInput from 'positron-components/components/form/form-input.svelte';
  import { arrayBufferToBase64 } from 'positron-components/util/convert.svelte';
  import Upload from '@lucide/svelte/icons/upload';
  import SimpleAvatar from 'positron-components/components/util/simple-avatar.svelte';
  import {
    profile_change_image,
    profile_update
  } from '$lib/backend/account/general.svelte';
  import { userData } from '$lib/backend/account/info.svelte';
  import { profileSchema } from './schema.svelte';
  import type { SvelteComponent } from 'svelte';
  import type { FormValue } from 'positron-components/components/form/types';

  let infoData = $derived(userData.value?.[1]);
  $effect(() => {
    if (infoData) formComp?.setValue(infoData);
  });

  let isLoading = $state(false);
  let imageInput: undefined | HTMLElement | null = $state(null);
  let formComp: SvelteComponent | undefined = $state();

  const updatePreview = async (e: Event) => {
    let input = e.target as HTMLInputElement;
    let file = input.files?.[0];
    if (file) {
      let image = arrayBufferToBase64(await file.arrayBuffer());
      let ret = await profile_change_image(image);

      if (ret) {
        toast.error('Update Error', {
          description: 'Error while uploading image'
        });
      } else {
        toast.success('Upload successful', {
          description: 'Your profile profile image was updated successfully'
        });
      }
    }
  };

  const startImageUpload = () => {
    imageInput?.click();
  };

  const updateProfile = async (form: FormValue<typeof profileSchema>) => {
    isLoading = true;

    let ret = await profile_update(form.name);

    isLoading = false;

    if (ret) {
      toast.error('Update Error', {
        description: 'There was an error while updating your profile'
      });
    } else {
      toast.success('Successfully Update', {
        description: 'Your profile was updated successfully'
      });
    }

    return undefined;
  };
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-xl font-medium">Profile</h3>
    <p class="text-muted-foreground text-sm">Change your personal info here</p>
  </div>
  <Separator />
  <div class="flex flex-col sm:flex-row">
    <div class="space-y-3">
      {#if infoData}
        <div class="relative">
          <SimpleAvatar src={infoData.image} class="size-52 rounded-full" />
          <Button
            class="group absolute inset-0 flex size-52 items-center justify-center rounded-full hover:bg-transparent hover:backdrop-blur-xs"
            variant="ghost"
            onclick={startImageUpload}
          >
            <Upload class="hidden size-12! group-hover:block" />
            <Label class="sr-only" for="picture">Picture</Label>
            <Input
              bind:ref={imageInput}
              type="file"
              id="picture"
              accept="image/png, image/jpeg"
              class="hidden"
              onchange={updatePreview}
            />
          </Button>
        </div>
      {:else}
        <Skeleton class="size-52 rounded-full" />
      {/if}
    </div>
    <BaseForm
      class="mt-5 flex flex-col space-y-2 sm:mt-0 sm:pl-10"
      onsubmit={updateProfile}
      schema={profileSchema}
      bind:isLoading
      bind:this={formComp}
    >
      {#snippet children({ props })}
        <div class="relative">
          <FormInput
            label="Username"
            placeholder={infoData ? 'Username' : ''}
            key="name"
            class="sm:max-w-72"
            {...props}
          />
          {#if !infoData}
            <div class="absolute inset-0 mt-11 ml-2 flex size-full">
              <Skeleton class="h-5 w-32" />
            </div>
          {/if}
        </div>
      {/snippet}
      {#snippet footer({ defaultBtn })}
        {@render defaultBtn({
          className: 'mt-8! ml-auto',
          content: 'Update Profile'
        })}
      {/snippet}
    </BaseForm>
  </div>
</div>
