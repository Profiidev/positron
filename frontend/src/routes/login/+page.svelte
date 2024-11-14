<script lang="ts">
  import Login from "./login-form.svelte";
</script>

<div
  class="container relative h-full flex-col items-center justify-center grid lg:max-w-none lg:grid-cols-2 px-0"
>
  <div
    class="bg-muted relative hidden h-full flex-col p-10 text-white lg:flex dark:border-r"
  >
    <div class="absolute inset-0 bg-cover background-img bg-center"></div>
    <div class="relative z-20 flex items-center text-2xl">Positron</div>
  </div>
  <div class="lg:p-8">
    <div
      class="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]"
    >
      <div class="flex flex-col space-y-2 text-center">
        <h1 class="text-2xl font-semibold tracking-tight">Login</h1>
        <p class="text-muted-foreground text-sm">
          Enter your login details below
        </p>
      </div>
      <Login />
    </div>
  </div>
</div>

<style>
  .background-img {
    background-image: url($lib/images/login.png?enhanced);
  }
</style>
