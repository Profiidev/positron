<script lang="ts">
  import './tiptap.css';
  import { onDestroy, onMount } from 'svelte';
  import { getRandomColor } from './color';
  import EditorToolbar from './toolbar/EditorToolbar.svelte';
  import { EditorContent, type Editor } from 'svelte-tiptap';
  import type { WebsocketProvider } from 'y-websocket';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import type { NoteActiveEditor } from '$lib/components/notes/types';

  type AwarenessUser = {
    id?: string;
    name?: string;
    color?: string;
  };

  let {
    id,
    username,
    userId,
    activeEditors = $bindable()
  }: {
    id: string;
    username?: string;
    userId?: string;
    activeEditors?: NoteActiveEditor[];
  } = $props();

  let editorState = $state<{ editor: Editor | null }>({ editor: null });
  let provider: WebsocketProvider | undefined = undefined;
  const userColor = getRandomColor();

  const setLocalAwarenessUser = () => {
    provider?.awareness.setLocalStateField('user', {
      id: userId,
      name: username ?? 'Unknown',
      color: userColor
    });
  };

  const syncActiveEditors = () => {
    if (!provider) return;

    const localClientId = provider.awareness.clientID;
    const editors: NoteActiveEditor[] = [];

    provider.awareness.getStates().forEach((state, clientId) => {
      if (clientId === localClientId) return;

      const user = state?.user as AwarenessUser | undefined;
      if (!user?.name) return;

      editors.push({
        clientId,
        id: user.id,
        name: user.name,
        color: user.color
      });
    });

    activeEditors = editors;
  };

  $effect(() => {
    username;
    userId;
    setLocalAwarenessUser();
  });

  onMount(async () => {
    const Doc = (await import('yjs')).Doc;
    const doc = new Doc();

    const { WebsocketProvider } = await import('y-websocket');

    provider = new WebsocketProvider('/api/notes/websocket', id, doc, {
      disableBc: true
    });
    provider.awareness.on('change', syncActiveEditors);

    setLocalAwarenessUser();
    syncActiveEditors();

    const extensions = (await import('./config')).extensions;
    const Collaboration = (await import('@tiptap/extension-collaboration'))
      .default;
    const CollaborationCaret = (
      await import('@tiptap/extension-collaboration-caret')
    ).default;
    const Editor = (await import('svelte-tiptap')).Editor;

    editorState.editor = new Editor({
      extensions: [
        ...extensions,
        Collaboration.configure({
          document: doc,
          provider
        }),
        CollaborationCaret.configure({
          provider,
          user: {
            name: username ?? 'Unknown',
            color: userColor
          }
        })
      ] as any,
      editorProps: {
        attributes: {
          class: 'max-w-full focus:outline-none'
        }
      },
      onTransaction: ({ editor }) => {
        editorState = { editor: editor as Editor };
      },
      autofocus: false
    });
  });

  const cleanup = () => {
    provider?.awareness.off('change', syncActiveEditors);
    editorState.editor?.destroy();
    provider?.destroy();
    provider = undefined;
    activeEditors = [];
  };

  onDestroy(cleanup);
</script>

<svelte:window onbeforeunload={cleanup} />

{#if editorState.editor}
  <div
    class="bg-card relative mt-2 flex h-full w-full flex-col overflow-hidden rounded-md border pb-[60px] sm:pb-0"
  >
    {/* @ts-ignore */ null}
    <EditorToolbar editor={editorState.editor} />
    <ScrollArea class="min-h-0 grow">
      <EditorContent
        editor={editorState.editor}
        class="flex min-h-full w-full min-w-full cursor-text"
      />
    </ScrollArea>
  </div>
{/if}
