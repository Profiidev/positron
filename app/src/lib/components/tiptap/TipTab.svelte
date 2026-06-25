<script lang="ts">
  import './tiptap.css';
  import { onDestroy, onMount } from 'svelte';
  import { getRandomColor } from './color';
  import EditorToolbar from './toolbar/EditorToolbar.svelte';
  import { EditorContent, Editor } from 'svelte-tiptap';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import { cn } from '@profidev/pleiades/utils';
  import type { NoteActiveEditor } from '../notes/types';
  import {
    noteContent,
    TauriWebsocketProvider
  } from '$lib/commands/notes.svelte';
  import { extensions } from './config';
  import Collaboration from '@tiptap/extension-collaboration';
  import CollaborationCaret from '@tiptap/extension-collaboration-caret';
  import { Doc, applyUpdate } from 'yjs';

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
    activeEditors = $bindable()
  }: {
    id: string;
    username?: string;
    userId?: string;
    editable?: boolean;
    activeEditors?: NoteActiveEditor[];
  } = $props();

  let editorState = $state<{ editor: Editor | null }>({ editor: null });
  let provider: TauriWebsocketProvider | undefined = undefined;
  let providerReady = $state(false);
  let lastEditable = $state<boolean | undefined>(undefined);
  const userColor = getRandomColor();
  const doc = new Doc();

  // svelte-ignore state_referenced_locally
  noteContent(id).then((content) => {
    if (!content) return;
    doc.transact(() => applyUpdate(doc, content));
  });

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
    provider = new TauriWebsocketProvider(id, doc);
    provider.awareness.on('change', onAwarenessChange);

    lastEditable = editable;
    providerReady = true;
    setLocalAwarenessState();
    syncActiveEditors();

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
