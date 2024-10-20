<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import { MenubarMenu, MenubarTrigger, Menubar, MenubarContent, MenubarItem } from "$lib/components/ui/menubar";
</script>

<Menubar>
  <MenubarMenu>
    <MenubarTrigger>File</MenubarTrigger>
    <MenubarContent>
      <MenubarItem>1234</MenubarItem>
    </MenubarContent>
  </MenubarMenu>
</Menubar>

<Button>Test</Button>
<a href="/login">Login</a>