<script lang="ts">
  import './tiptap.css';
  import { onDestroy, onMount } from 'svelte';
  import { extensions, getRandomColor } from './config';
  import EditorToolbar from './toolbar/EditorToolbar.svelte';
  import { EditorContent, Editor } from 'svelte-tiptap';
  import Collaboration from '@tiptap/extension-collaboration';
  import CollaborationCaret from '@tiptap/extension-collaboration-caret';
  import * as Y from 'yjs';
  import { WebsocketProvider } from 'y-websocket';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';

  const {
    id
  }: {
    id: string;
  } = $props();

  let editorState = $state<{ editor: Editor | null }>({ editor: null });
  const doc = new Y.Doc();
  let provider: WebsocketProvider | undefined = undefined;
  let undoManager: Y.UndoManager | undefined = undefined;

  onMount(() => {
    provider = new WebsocketProvider('/api/notes/websocket', id, doc, {
      disableBc: true
    });
    undoManager = new Y.UndoManager(doc);

    provider.on('status', (e) => {
      console.log(e);
    });

    editorState.editor = new Editor({
      extensions: [
        ...extensions,
        Collaboration.configure({
          document: doc,
          provider,
          yUndoOptions: {
            undoManager
          }
        }),
        CollaborationCaret.configure({
          provider,
          user: {
            name: 'Anonymous',
            color: getRandomColor()
          }
        })
      ] as any,
      editorProps: {
        attributes: {
          class: 'max-w-full focus:outline-none'
        }
      },
      onUpdate: ({ editor }) => {
        console.log(editor.getText());
      },
      onTransaction: ({ editor }) => {
        editorState = { editor: editor as Editor };
      },
      autofocus: false
    });
  });

  onDestroy(() => {
    editorState.editor?.destroy();
    undoManager?.destroy();
    provider?.destroy();
  });
</script>

{#if editorState.editor}
  <div
    class="bg-card relative mt-2 flex h-full w-full flex-col overflow-hidden rounded-md border pb-[60px] sm:pb-0"
  >
    {/* @ts-ignore */ null}
    <EditorToolbar editor={editorState.editor} />
    <ScrollArea class="min-h-0 grow">
      <EditorContent
        editor={editorState.editor}
        class="flex w-full min-w-full cursor-text min-h-full"
      />
    </ScrollArea>
  </div>
{/if}
