<script lang="ts">
  import './tiptap.css';
  import { onDestroy, onMount } from 'svelte';
  import { EditorContent, type Editor } from 'svelte-tiptap';
  import { ScrollArea } from '@profidev/pleiades/components/ui/scroll-area';
  import { cn } from '@profidev/pleiades/utils';
  import type { Doc as YDoc, applyUpdate as ApplyUpdate } from 'yjs';

  let {
    data
  }: {
    data: Uint8Array;
  } = $props();

  let editorState = $state<{ editor: Editor | null }>({ editor: null });
  let doc: YDoc;
  let applyUpdateFn: typeof ApplyUpdate | undefined = undefined;

  $effect(() => {
    if (data && data.length > 0 && applyUpdateFn) {
      applyUpdateFn(doc, data);
    }
  });

  onMount(async () => {
    const { Doc, applyUpdate } = await import('yjs');
    doc = new Doc();
    applyUpdateFn = applyUpdate;

    if (data && data.length > 0) {
      applyUpdate(doc, data);
    }

    const extensions = (await import('./config')).extensions;
    const Collaboration = (await import('@tiptap/extension-collaboration'))
      .default;
    const Editor = (await import('svelte-tiptap')).Editor;

    editorState.editor = new Editor({
      extensions: [
        ...extensions,
        Collaboration.configure({
          document: doc
        })
      ] as any,
      editable: false,
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
    editorState.editor?.destroy();
    editorState = { editor: null };
  };

  onDestroy(cleanup);
</script>

<svelte:window onbeforeunload={cleanup} />

{#if editorState.editor}
  <div
    class="bg-card relative mt-2 flex h-full w-full flex-col overflow-hidden rounded-md border"
  >
    <ScrollArea class="min-h-0 grow">
      <EditorContent
        editor={editorState.editor}
        class={cn('flex min-h-full w-full min-w-full cursor-default')}
      />
    </ScrollArea>
  </div>
{/if}
