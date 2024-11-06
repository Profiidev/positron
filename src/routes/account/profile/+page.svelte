<script lang="ts">
  import {
    change_image,
    info,
    update_profile,
  } from "$lib/account/general.svelte";
  import type { UserInfo } from "$lib/account/types.svelte";
  import { get_uuid } from "$lib/auth/token.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { arrayBufferToBase64 } from "$lib/util/convert.svelte";
  import { Upload, LoaderCircle } from "lucide-svelte";
  import { toast } from "svelte-sonner";

  let uuid = get_uuid() || "";

  let infoData: UserInfo | undefined = $state();
  info(uuid).then((info) => {
    infoData = info;
    name = info?.name;
  });

  let name: string | undefined = $state();
  let isLoading = $state(false);
  let imageInput: undefined | HTMLElement | null = $state(null);

  const updatePreview = async (e: Event) => {
    let input = e.target as HTMLInputElement;
    let file = input.files?.[0];
    if (file) {
      let image = arrayBufferToBase64(await file.arrayBuffer());
      let ret = await change_image(image);

      if (ret !== null) {
        toast.error("Update Error", {
          description: "Error while uploading image",
        });
      } else {
        infoData = await info(uuid);
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

    let ret = await update_profile({
      name,
    });

    isLoading = false;

    if (ret !== null) {
      toast.error("Update Error", {
        description: "There was an error while updating your profile",
      });
    } else {
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
          <img
            src={`data:image/png;base64, ${infoData.image}`}
            alt="Profile"
            class="size-52 rounded-full"
          />
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
    <form class="mt-5 sm:mt-0 sm:pl-10 flex flex-col" onsubmit={updateProfile}>
      <Label for="username">Username</Label>
      <div class="relative">
        <Input
          id="username"
          autocomplete="off"
          placeholder={infoData ? "Username" : ""}
          class="sm:min-w-72"
          required
          bind:value={name}
        />
        {#if !infoData}
          <div class="absolute inset-0 size-full flex items-center ml-2">
            <Skeleton class="h-5 w-32" />
          </div>
        {/if}
      </div>
      <Button type="submit" class="mt-8 ml-auto" disabled={isLoading}>
        {#if isLoading}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        Update Profile
      </Button>
    </form>
  </div>
</div>
