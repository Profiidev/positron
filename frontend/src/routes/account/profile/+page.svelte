<script lang="ts">
  import { getProfileInfo, updateInfo } from "$lib/backend/account/info.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { arrayBufferToBase64 } from "$lib/util/convert.svelte";
  import { Upload, LoaderCircle } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import Avatar from "$lib/components/util/avatar.svelte";
  import {
    profile_change_image,
    profile_update,
  } from "$lib/backend/account/general.svelte";

  let infoData = $derived(getProfileInfo());
  $effect(() => {
    name = infoData?.name;
  });

  let name: string | undefined = $state();
  let isLoading = $state(false);
  let imageInput: undefined | HTMLElement | null = $state(null);

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
        await updateInfo();
        toast.success("Upload successful", {
          description: "Your profile profile image was updated successfully",
        });
      }
    }
  };

  const startImageUpload = () => {
    imageInput?.click();
  };

  const updateProfile = async () => {
    if (!name) {
      toast.error("Missing Inputs");
      return;
    }

    isLoading = true;

    let ret = await profile_update(name);

    isLoading = false;

    if (ret) {
      toast.error("Update Error", {
        description: "There was an error while updating your profile",
      });
    } else {
      await updateInfo();
      toast.success("Successfully Update", {
        description: "Your profile was updated successfully",
      });
    }
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
          <Avatar src={infoData.image} class="size-52 rounded-full" />
          <Button
            class="group absolute hover:backdrop-blur-sm size-52 rounded-full inset-0 flex items-center justify-center hover:bg-transparent"
            variant="ghost"
            onclick={startImageUpload}
          >
            <Upload class="!size-12 hidden group-hover:block" />
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
    <form
      class="mt-5 sm:mt-0 sm:pl-10 flex flex-col space-y-2"
      onsubmit={updateProfile}
    >
      <Label for="username">Username</Label>
      <div class="relative">
        <Input
          id="username"
          autocomplete="off"
          placeholder={infoData ? "Username" : ""}
          class="sm:max-w-72"
          required
          bind:value={name}
        />
        {#if !infoData}
          <div class="absolute inset-0 size-full flex items-center ml-2">
            <Skeleton class="h-5 w-32" />
          </div>
        {/if}
      </div>
      <Button type="submit" class="!mt-8 ml-auto" disabled={isLoading}>
        {#if isLoading}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        Update Profile
      </Button>
    </form>
  </div>
</div>
