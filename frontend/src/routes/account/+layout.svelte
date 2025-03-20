<script lang="ts">
  import { Separator } from 'positron-components/components/ui';
  import { SimpleSidebar } from 'positron-components/components/nav';

  const { children } = $props();

  const routes = [
    {
      title: 'Profile',
      href: '/account/profile'
    },
    {
      title: 'Security',
      href: '/account/security'
    },
    {
      title: 'Email',
      href: '/account/email'
    }
  ];
</script>

<div class="w-full space-y-6 p-10">
  <div class="space-y-0.5">
    <h2 class="text-2xl font-bold tracking-tight">Settings</h2>
    <p class="text-muted-foreground">Manage your account</p>
  </div>
  <Separator class="my-6" />
  <div class="flex flex-col space-y-8 lg:flex-row lg:space-y-0 lg:space-x-12">
    <aside class="lg:w-52 lg:max-w-1/5">
      <SimpleSidebar items={routes} class="" />
    </aside>
    <div class="flex-1">
      {@render children()}
    </div>
  </div>
</div>
