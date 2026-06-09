<script lang="ts">
  import './tiptap.css';
  import { Editor } from '@tiptap/core';
  import { onDestroy, onMount } from 'svelte';
  import { extensions } from './config';

  let element: HTMLElement | undefined = $state();
  let editorState = $state<{
    editor: Editor | null;
  }>({ editor: null });

  onMount(() => {
    editorState.editor = new Editor({
      element,
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
      onTransaction: ({ editor }) => {
        editorState = { editor };
      },
      autofocus: false
    });
  });

  onDestroy(() => {
    editorState.editor?.destroy();
  });
</script>

<div style="position: relative">
  {#if editorState.editor}
    <div class="fixed-menu">
      <button
        onclick={() =>
          editorState.editor?.chain().focus().toggleHeading({ level: 1 }).run()}
        class:active={editorState.editor.isActive('heading', { level: 1 })}
      >
        H1
      </button>
      <button
        onclick={() =>
          editorState.editor?.chain().focus().toggleHeading({ level: 2 }).run()}
        class:active={editorState.editor.isActive('heading', { level: 2 })}
      >
        H2
      </button>
      <button
        onclick={() => editorState.editor?.chain().focus().setParagraph().run()}
        class:active={editorState.editor.isActive('paragraph')}
      >
        P
      </button>
    </div>
  {/if}

  <div bind:this={element}></div>
</div>

<style>
  button.active {
    background: black;
    color: white;
  }
</style>
