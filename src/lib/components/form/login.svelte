<script lang="ts">
  import { cn } from "$lib/utils";
  import { Key, LoaderCircle } from "lucide-svelte";
  import { Button } from "../ui/button/index";
  import { Input } from "../ui/input/index";
  import { Label } from "../ui/label/index";

  let className: string | undefined | null = undefined;
  export { className as class };

  let enterEmail = true;
  let isLoading = false;
  let email = "";
  let password = "";

  const reset = () => {
    enterEmail = true;
    isLoading = false;
    email = "";
    password = "";
  }

  const onSubmit = () => {
    if(enterEmail) {
      enterEmail = false;
      return;
    }

    isLoading = true;
    setTimeout(() => {
      isLoading = false;
    }, 3000);
  }
</script>

<div class={cn("grid gap-6", className)}>
  <form on:submit|preventDefault={onSubmit}>
    <div class="grid gap-2">
      <div class="grid gap-1">
        {#if enterEmail}
          <Label class="sr-only" for="email">Email</Label>
          <Input id="email" placeholder="name@example.com" type="email" autocapitalize="none" autocomplete="email" autocorrect="off" disabled={isLoading} required bind:value={email} />
        {:else}
          <div class="flex mb-3 w-full items-center">
            <span class="text-foreground truncate max-w-36 sm:max-w-72">{email}</span>
            <Button variant="link" type="button" disabled={isLoading} class="ml-auto p-0" on:click={reset}>Change</Button>
          </div>
          <Label class="sr-only" for="password">Password</Label>
          <Input id="password" placeholder="Password" type="password" autocapitalize="none" autocomplete="current-password" autocorrect="off" disabled={isLoading} required bind:value={password} />
        {/if}
      </div>
      <Button type="submit" disabled={isLoading}>
        {#if isLoading}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        {#if enterEmail}
          Sign In with Email
        {:else}
          Sign In
        {/if}
      </Button>
    </div>
  </form>
  <div class="relative">
    <div class="absolute inset-0 flex items-center">
      <span class="w-full border-t" />
    </div>
    <div class="relative flex justify-center text-xs uppercase">
      <span class="bg-background text-muted-foreground px-2">Or continue with </span>
    </div>
  </div>
  <Button variant="outline" type="button" disabled={isLoading}>
    {#if isLoading}
      <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
    {:else}
      <Key class="mr-2 h-4 w-4" />
    {/if}
    Passkey
  </Button>
</div>