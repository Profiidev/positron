<script lang="ts">
  import './tiptap.css';
  import { onDestroy, onMount } from 'svelte';
  import { extensions } from './config';
  import EditorToolbar from './toolbar/EditorToolbar.svelte';
  import { EditorContent, Editor } from 'svelte-tiptap';
  import Collaboration from '@tiptap/extension-collaboration';
  import * as Y from 'yjs';
  import { WebsocketProvider } from 'y-websocket';

  let editorState = $state<{ editor: Editor | null }>({ editor: null });
  const doc = new Y.Doc();

  onMount(() => {
    const provider = new WebsocketProvider('/api/notes/ws', 'test-room', doc, {
      disableBc: true
    });
    const undoManager = new Y.UndoManager(doc);
    provider.awareness.setLocalStateField('user', {
      name: 'Anonymous',
      color: '#ffff00',
      colorLight: '#00ff00'
    });

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
        })
      ],
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
  });
</script>

{#if editorState.editor}
  <div class="bg-card relative w-full overflow-hidden border pb-[60px] sm:pb-0">
    <EditorToolbar editor={editorState.editor} />
    <EditorContent
      editor={editorState.editor}
      class="min-h-[600px] w-full min-w-full cursor-text sm:p-6"
    />
  </div>
{/if}
