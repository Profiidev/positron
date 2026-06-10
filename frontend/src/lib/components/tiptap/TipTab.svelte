<script lang="ts">
  import './tiptap.css';
  import { onDestroy, onMount } from 'svelte';
  import { extensions } from './config';
  import EditorToolbar from './toolbar/EditorToolbar.svelte';
  import { EditorContent, Editor } from 'svelte-tiptap';

  let editorState = $state<{ editor: Editor | null }>({ editor: null });

  onMount(() => {
    editorState.editor = new Editor({
      extensions,
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
