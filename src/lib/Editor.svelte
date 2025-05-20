<script lang="ts">
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import * as monaco from 'monaco-editor';

  let {
    settings = {},
    editor = $bindable(),
    content = $bindable(''),
    ...props
  }: {
    settings?: monaco.editor.IStandaloneEditorConstructionOptions;
    editor?: monaco.editor.IStandaloneCodeEditor;
    content?: string;
    class?: string;
  } = $props();

  let element: HTMLDivElement | undefined = $state();
  const initialContent = content;

  $effect(() => {
    if (!element) {
      return;
    }

    editor = monaco.editor.create(element, {
      value: initialContent,
      ...settings,
    });

    return () => {
      editor?.dispose();
    };
  });

  $effect(() => {
    if (!editor) {
      return;
    }

    const didPaste = editor.onDidPaste(async (e) => {
      const clipboardContent = await readText();
      editor?.executeEdits('pest-rs', [
        {
          range: e.range,
          text: clipboardContent,
        },
      ]);
    });

    const model = editor.getModel();
    const didContentChange = model?.onDidChangeContent(() => {
      content = model?.getValue() ?? '';
    });

    return () => {
      didPaste.dispose();
      didContentChange?.dispose();
    };
  });
</script>

<div bind:this={element} class={props.class ?? 'w-full h-full'}></div>
