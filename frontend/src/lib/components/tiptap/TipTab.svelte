<script lang="ts">
  import './tiptap.css';
  import { onDestroy, onMount } from 'svelte';
  import { extensions } from './config';
  import EditorToolbar from './toolbar/EditorToolbar.svelte';
  import FloatingToolbar from './extensions/FloatingToolbar.svelte';
  import { EditorContent, Editor } from 'svelte-tiptap';

  let editor = $state<Editor | null>(null);

  onMount(() => {
    editor = new Editor({
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
      autofocus: false
    });
  });

  onDestroy(() => {
    editor?.destroy();
  });
</script>

{#if editor}
  <div
    class="bg-card relative max-h-[calc(100dvh-6rem)] w-full overflow-hidden overflow-y-scroll border pb-[60px] sm:pb-0"
  >
    <EditorToolbar {editor} />
    <FloatingToolbar {editor} />
    <EditorContent
      {editor}
      class="min-h-[600px] w-full min-w-full cursor-text sm:p-6"
    />
  </div>
{/if}
