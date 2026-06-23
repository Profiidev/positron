<script lang="ts">
  import './tiptap/tiptap.css';
  import { onDestroy, onMount } from 'svelte';
  import { getRandomColor } from './tiptap/color';
  import EditorToolbar from './tiptap/toolbar/EditorToolbar.svelte';
  import { EditorContent, type Editor } from 'svelte-tiptap';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import { cn } from '@profidev/pleiades/utils';
  import type { NoteActiveEditor } from './types';
  import { TauriWebsocketProvider } from '$lib/commands/notes.svelte';

  type AwarenessUser = {
    name?: string;
    color?: string;
  };

  type AwarenessState = {
    user?: AwarenessUser;
    canEdit?: boolean;
    userId?: string;
  };

  let {
    id,
    username,
    userId,
    editable = true,
    activeEditors = $bindable(),
    wsPath = '/api/notes/websocket'
  }: {
    id: string;
    username?: string;
    userId?: string;
    editable?: boolean;
    activeEditors?: NoteActiveEditor[];
    wsPath?: string;
  } = $props();

  let editorState = $state<{ editor: Editor | null }>({ editor: null });
  let provider: TauriWebsocketProvider | undefined = undefined;
  let providerReady = $state(false);
  let lastEditable = $state<boolean | undefined>(undefined);
  const userColor = getRandomColor();

  const setLocalAwarenessState = () => {
    if (!provider) return;
    provider.awareness.setLocalStateField('canEdit', editable);
    if (userId) {
      provider.awareness.setLocalStateField('userId', userId);
    }
    provider.awareness.setLocalStateField('user', {
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

      const awareness = state as AwarenessState | undefined;
      const user = awareness?.user;
      if (!user?.name || awareness?.canEdit !== true) return;

      editors.push({
        clientId,
        id: awareness.userId,
        name: user.name,
        color: user.color
      });
    });

    activeEditors = editors;
  };

  const onAwarenessChange = () => {
    if (provider?.awareness.getLocalState()?.canEdit !== editable) {
      setLocalAwarenessState();
    }
    syncActiveEditors();
  };

  $effect(() => {
    username;
    userId;
    editable;
    setLocalAwarenessState();
  });

  $effect(() => {
    editable;
    editorState.editor?.setEditable(editable);
  });

  $effect(() => {
    if (!providerReady || !provider) return;

    const next = editable;
    if (lastEditable === undefined || next === lastEditable) return;

    lastEditable = next;
    provider.disconnect();
    provider.connect();
    setLocalAwarenessState();
  });

  onMount(async () => {
    const Doc = (await import('yjs')).Doc;
    const doc = new Doc();

    provider = new TauriWebsocketProvider(id, doc);
    console.log(provider)
    provider.awareness.on('change', onAwarenessChange);

    lastEditable = editable;
    providerReady = true;
    setLocalAwarenessState();
    syncActiveEditors();

    const extensions = (await import('./tiptap/config')).extensions;
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
      editable,
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
    providerReady = false;
    lastEditable = undefined;
    provider?.awareness.off('change', onAwarenessChange);
    editorState.editor?.destroy();
    provider?.destroy();
    provider = undefined;
    activeEditors = [];
    editorState = { editor: null };
  };

  onDestroy(cleanup);
</script>

<svelte:window onbeforeunload={cleanup} />

{#if editorState.editor}
  <div
    class="bg-card relative mt-2 flex h-full w-full flex-col overflow-hidden rounded-md border"
  >
    {#if editable && editorState.editor}
      {/* @ts-ignore */ null}
      <EditorToolbar editor={editorState.editor} />
    {/if}
    <ScrollArea class="min-h-0 grow">
      <EditorContent
        editor={editorState.editor}
        class={cn(
          'flex min-h-full w-full min-w-full',
          editable ? 'cursor-text' : 'cursor-default'
        )}
      />
    </ScrollArea>
  </div>
{/if}
