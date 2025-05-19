<script lang="ts">
  import * as monaco from 'monaco-editor';

  let {
    settings = {},
    editor = $bindable(),
    ...props
  }: {
    settings?: monaco.editor.IStandaloneEditorConstructionOptions;
    editor?: monaco.editor.IStandaloneCodeEditor;
    class?: string
  } = $props();

  let element: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (!element) {
      return;
    }

    editor = monaco.editor.create(element, settings);
  });
</script>

<div bind:this={element} class={props.class ?? "w-full h-full"}></div>
