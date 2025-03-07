<script lang="ts">
  import {
    Button,
    Input,
    Label,
    Separator,
    Skeleton,
  } from "positron-components/components/ui";
  import { BaseForm, FormInput } from "positron-components/components/form";
  import { arrayBufferToBase64 } from "positron-components/util";
  import { Upload } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import { SimpleAvatar } from "positron-components/components/util";
  import {
    profile_change_image,
    profile_update,
  } from "$lib/backend/account/general.svelte";
  import { userData } from "$lib/backend/account/info.svelte";
  import type { PageServerData } from "./$types";
  import { profileSchema } from "./schema.svelte";
  import type { SuperValidated } from "sveltekit-superforms";
  import type { SvelteComponent } from "svelte";

  interface Props {
    data: PageServerData;
  }

  let { data }: Props = $props();

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
        toast.error("Update Error", {
          description: "Error while uploading image",
        });
      } else {
        toast.success("Upload successful", {
          description: "Your profile profile image was updated successfully",
        });
      }
    }
  };

  const startImageUpload = () => {
    imageInput?.click();
  };

  const updateProfile = async (form: SuperValidated<any>) => {
    isLoading = true;

    let ret = await profile_update(form.data.name);

    isLoading = false;

    if (ret) {
      toast.error("Update Error", {
        description: "There was an error while updating your profile",
      });
    } else {
      toast.success("Successfully Update", {
        description: "Your profile was updated successfully",
      });
    }

    return undefined;
  };

  const profileForm = {
    form: data.profile,
    schema: profileSchema,
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
          <Button.Button
            class="group absolute hover:backdrop-blur-xs size-52 rounded-full inset-0 flex items-center justify-center hover:bg-transparent"
            variant="ghost"
            onclick={startImageUpload}
          >
            <Upload class="size-12! hidden group-hover:block" />
            <Label class="sr-only" for="picture">Picture</Label>
            <Input
              bind:ref={imageInput}
              type="file"
              id="picture"
              accept="image/png, image/jpeg"
              class="hidden"
              onchange={updatePreview}
            />
          </Button.Button>
        </div>
      {:else}
        <Skeleton class="size-52 rounded-full" />
      {/if}
    </div>
    <BaseForm
      class="mt-5 sm:mt-0 sm:pl-10 flex flex-col space-y-2"
      onsubmit={updateProfile}
      form={profileForm}
      bind:isLoading
      confirm="Update Profile"
      bind:this={formComp}
    >
      {#snippet children({ props })}
        <div class="relative">
          <FormInput
            label="Username"
            placeholder={infoData ? "Username" : ""}
            key="name"
            class="sm:max-w-72"
            {...props}
          />
          {#if !infoData}
            <div class="absolute inset-0 size-full flex mt-11 ml-2">
              <Skeleton class="h-5 w-32" />
            </div>
          {/if}
        </div>
      {/snippet}
      {#snippet footer({ children })}
        {@render children({ className: "mt-8! ml-auto" })}
      {/snippet}
    </BaseForm>
  </div>
</div>
