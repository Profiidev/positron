<script lang="ts">
  import { change_image, info, update_profile } from "$lib/account/general.svelte";
  import type { UserInfo } from "$lib/account/types.svelte";
  import { get_uuid } from "$lib/auth/token.svelte";
  import FormDialog from "$lib/components/form/form-dialog.svelte";
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

  let image: string | undefined = $state();
  let name: string | undefined = $state();
  let isLoading = $state(false);

  const updatePreview = async (e: Event) => {
    let input = e.target as HTMLInputElement;
    let file = input.files?.[0];
    if (file) {
      image = arrayBufferToBase64(await file.arrayBuffer());
    }
  };

  const startImageUpload = () => {
    image = undefined;
    return true;
  };

  const uploadImage = async () => {
    if (!image) {
      return "No image provided";
    }

    let ret = await change_image(image);

    if (ret !== null) {
      return "Error while uploading Image";
    }

    info(uuid).then((info) => (infoData = info));
    return "";
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
  <div class="flex">
    <div class="space-y-3">
      {#if infoData}
        <div class="relative">
          <img
            src={`data:image/png;base64, ${infoData.image}`}
            alt="Profile"
            class="size-52 rounded-full"
          />
          <FormDialog
            title="Change Profile Picture"
            description="Upload a image here to make it your profile picture"
            confirm="Upload"
            trigger={{
              variant: "ghost",
              class:
                "group absolute hover:backdrop-blur-sm size-52 rounded-full inset-0 flex items-center justify-center hover:bg-transparent",
            }}
            onopen={startImageUpload}
            onsubmit={uploadImage}
          >
            {#snippet triggerInner()}
              <Upload class="!size-12 hidden group-hover:block" />
            {/snippet}
            <Label class="sr-only" for="picture">Picture</Label>
            <Input
              type="file"
              id="picture"
              accept="image/png, image/jpeg"
              onchange={updatePreview}
            />
            <div class="flex justify-center">
              {#if image}
                <img
                  src={`data:image/png;base64, ${image}`}
                  alt="Profile"
                  class="size-52 rounded-full border object-cover"
                />
              {:else}
                <div
                  class="size-52 rounded-full border-2 flex justify-center items-center"
                >
                  <p>No Preview available</p>
                </div>
              {/if}
            </div>
          </FormDialog>
        </div>
      {:else}
        <Skeleton class="size-52 rounded-full" />
      {/if}
    </div>
    <form class="space-y-3 pl-10" onsubmit={updateProfile}>
      <h3 class="text-lg">Username</h3>
      <Input autocomplete="off" placeholder="Username" required bind:value={name} />
      <Button type="submit" class="!mt-8" disabled={isLoading}>
        {#if isLoading}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        Update Profile
      </Button>
    </form>
  </div>
</div>
