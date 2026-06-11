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
      content: `
        <h1>Hello Svelte! 🌍️ </h1>
        <p>This editor is running in Svelte.</p>
        <p>Select some text to see the bubble menu popping up.</p>
      `,
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
  <div class="bg-card relative w-full overflow-hidden border pb-[60px] sm:pb-0">
    {/* @ts-ignore */ null}
    <EditorToolbar editor={editorState.editor} />
    <EditorContent
      editor={editorState.editor}
      class="min-h-[600px] w-full min-w-full cursor-text sm:p-6"
    />
  </div>
{/if}
