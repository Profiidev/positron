<script lang="ts">
  import * as Sidebar from 'positron-components/components/ui/sidebar';
  import * as Collapsible from 'positron-components/components/ui/collapsible';
  import ChevronRight from '@lucide/svelte/icons/chevron-right';
  import Image from '@lucide/svelte/icons/image';
  import KeyRound from '@lucide/svelte/icons/key-round';
  import Users from '@lucide/svelte/icons/users';
  import { page } from '$app/state';
  import { Permission } from '$lib/backend/management/types.svelte';
  import { userData } from '$lib/backend/account/info.svelte';

  const allItems = [
    {
      title: 'Services',
      items: [
        {
          title: 'Image',
          icon: Image,
          isActive: true,
          items: [
            {
              title: 'Apod',
              url: '/services/apod',
              permission: Permission.ApodList
            }
          ]
        }
      ]
    },
    {
      title: 'Management',
      items: [
        {
          title: 'User Management',
          icon: Users,
          isActive: true,
          items: [
            {
              title: 'Users',
              url: '/management/users',
              permission: Permission.UserList
            },
            {
              title: 'Groups',
              url: '/management/groups',
              permission: Permission.GroupList
            }
          ]
        },
        {
          title: 'Auth Provider',
          icon: KeyRound,
          isActive: true,
          items: [
            {
              title: 'OAuth / OpenID Clients',
              url: '/management/oauth_client',
              permission: Permission.OAuthClientList
            },
            {
              title: 'OAuth Scopes',
              url: '/management/oauth_scope',
              permission: Permission.OAuthClientList
            },
            {
              title: 'OAuth Policies',
              url: '/management/oauth_policy',
              permission: Permission.OAuthClientList
            }
          ]
        }
      ]
    }
  ];

  let permissions = $derived(userData.value?.[0].permissions);
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
      : undefined;
  });
</script>

{#if !items}
  <Sidebar.Group>
    <Sidebar.Menu>
      <Sidebar.MenuSkeleton />
      <Sidebar.MenuSkeleton />
      <Sidebar.MenuSkeleton />
      <Sidebar.MenuSkeleton />
    </Sidebar.Menu>
  </Sidebar.Group>
{:else}
  {#each items as group (group.title)}
    <Sidebar.Group>
      <Sidebar.GroupLabel>{group.title}</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each group.items as mainItem (mainItem.title)}
            <Collapsible.Root
              open={mainItem.isActive}
              class="group/collapsible"
            >
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
                              isActive={page.url.pathname === subItem.url}
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
{/if}
