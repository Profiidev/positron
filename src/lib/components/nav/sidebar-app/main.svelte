<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import { ChevronRight, Users } from "lucide-svelte";
  import { page } from "$app/stores";
  import { Permission } from "$lib/backend/management/types.svelte";
  import { getPermissions } from "$lib/backend/account/info.svelte";

  const allItems = [
    {
      title: "Management",
      items: [
        {
          title: "User Management",
          icon: Users,
          isActive: true,
          items: [
            {
              title: "Users",
              url: "/management/users",
              permission: Permission.UserList,
            },
            {
              title: "Groups",
              url: "/management/groups",
              permission: Permission.GroupList,
            },
          ],
        },
      ],
    },
  ];

  let permissions = $derived(getPermissions());
  let items = $derived.by(() => {
    return permissions
      ? allItems
          .map((g) => {
            g.items = g.items
              .map((sg) => {
                sg.items = sg.items.filter((i) => {
                  return permissions.includes(i.permission);
                });
                return sg;
              })
              .filter((sg) => {
                return sg.items.length > 0;
              });
            return g;
          })
          .filter((g) => {
            return g.items.length > 0;
          })
      : [];
  });
</script>

{#each items as group (group.title)}
  <Sidebar.Group>
    <Sidebar.GroupLabel>{group.title}</Sidebar.GroupLabel>
    <Sidebar.GroupContent>
      <Sidebar.Menu>
        {#each group.items as mainItem (mainItem.title)}
          <Collapsible.Root open={mainItem.isActive} class="group/collapsible">
            {#snippet child({ props })}
              <Sidebar.MenuItem {...props}>
                <Collapsible.Trigger>
                  {#snippet child({ props })}
                    <Sidebar.MenuButton class="h-9" {...props}>
                      {#snippet tooltipContent()}
                        {mainItem.title}
                      {/snippet}
                      {#if mainItem.icon}
                        <mainItem.icon />
                      {/if}
                      <span class="text-base">{mainItem.title}</span>
                      <ChevronRight
                        class="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90"
                      />
                    </Sidebar.MenuButton>
                  {/snippet}
                </Collapsible.Trigger>
                <Collapsible.Content>
                  {#if mainItem.items}
                    <Sidebar.MenuSub>
                      {#each mainItem.items as subItem (subItem.title)}
                        <Sidebar.MenuSubItem>
                          <Sidebar.MenuSubButton
                            isActive={$page.url.pathname === subItem.url}
                            class="h-8"
                          >
                            {#snippet child({ props })}
                              <a href={subItem.url} {...props}>
                                <span>{subItem.title}</span>
                              </a>
                            {/snippet}
                          </Sidebar.MenuSubButton>
                        </Sidebar.MenuSubItem>
                      {/each}
                    </Sidebar.MenuSub>
                  {/if}
                </Collapsible.Content>
              </Sidebar.MenuItem>
            {/snippet}
          </Collapsible.Root>
        {/each}
      </Sidebar.Menu>
    </Sidebar.GroupContent>
  </Sidebar.Group>
{/each}
