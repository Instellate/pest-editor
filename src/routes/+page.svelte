<script lang="ts">
  import { commands, type TokenTree } from '$lib/bindings';
  import { initializePest } from '$lib/monaco-pest';
  import Editor from '$lib/Editor.svelte';
  import * as monaco from 'monaco-editor';
  import debounce from 'lodash.debounce';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import { Network, type Edge, type Data as VisData, type Node } from 'vis-network';

  monaco.languages.register({ id: 'pest-rs' });
  initializePest();

  let leftPanel: monaco.editor.IStandaloneCodeEditor | undefined = $state(undefined);
  let rightPanel: monaco.editor.IStandaloneCodeEditor | undefined = $state(undefined);

  let network: Network | undefined = $state();
  let networkElement: HTMLDivElement | undefined = $state();
  let networkData: VisData = $state({});

  let rules: string[] = $state([]);
  let selectedRule = $state('');
  let grammar = $state('');
  let input = $state('');

  const updateGrammar = debounce((grammar: string) => {
    commands.updatePestGrammar(grammar).then((v) => {
      if (v.status === 'error') {
        const errors: monaco.editor.IMarkerData[] = v.error.map((e) => {
          return {
            startLineNumber: e.location.start_line,
            startColumn: e.location.start_col,
            endLineNumber: e.location.end_line,
            endColumn: e.location.end_col,
            message: e.message,
            severity: monaco.MarkerSeverity.Error,
          };
        });

        const model = leftPanel?.getModel();
        if (model) {
          monaco.editor.setModelMarkers(model, 'pest-rs', errors);
        }
      } else {
        rules = v.data;
        if (!rules.includes(selectedRule)) {
          selectedRule = '';
        }

        const model = leftPanel?.getModel();
        if (model) {
          monaco.editor.setModelMarkers(model, 'pest-rs', []);
        }
      }
    });
  }, 500);

  const parseInput = debounce((input: string, rule: string) => {
    commands.parseInput(input, rule).then((v) => {
      if (v.status === 'error') {
        const model = rightPanel?.getModel();
        if (model) {
          monaco.editor.setModelMarkers(model, 'pest-rs', [
            {
              startLineNumber: v.error.location.start_line,
              startColumn: v.error.location.start_col,
              endLineNumber: v.error.location.end_line,
              endColumn: v.error.location.end_col,
              message: v.error.message,
              severity: monaco.MarkerSeverity.Error,
            },
          ]);
        }
      } else {
        const model = rightPanel?.getModel();
        if (model) {
          monaco.editor.setModelMarkers(model, 'pest-rs', []);
        }

        if (v.data) {
          const labelCount = new Map<string, number>();

          const [_, newData] = createData(v.data, labelCount);
          networkData = newData;
          if (network) {
            network.destroy();
          }

          network = new Network(networkElement!, networkData, {
            layout: {
              hierarchical: {
                direction: 'UD',
              },
            },
          });
        } else {
          networkData = {};
          network = new Network(networkElement!, networkData, {
            layout: {
              hierarchical: {
                direction: 'UD',
              },
            },
          });
        }
      }
    });
  }, 250);

  interface FormattedData extends VisData {
    edges: Edge[];
    nodes: Node[];
  }

  function createData(
    tree: TokenTree,
    labelCount: Map<string, number>,
  ): [Node, FormattedData] {
    const data: FormattedData = {
      edges: [],
      nodes: [],
    };

    let id;
    const count = labelCount.get(tree.label);
    if (count) {
      id = `${tree.label}-${count}`;
      labelCount.set(tree.label, count + 1);
    } else {
      id = `${tree.label}-0`;
      labelCount.set(tree.label, 1);
    }

    const node: Node = {
      id,
      label: tree.label,
      shape: 'text',
      font: {
        background: '#FFFFFF',
      },
    };

    const children = tree.children.map((c) => createData(c, labelCount));
    for (const [childNode, child] of children) {
      data.edges = data.edges.concat(child.edges);
      data.nodes = data.nodes.concat(child.nodes);

      data.edges.push({
        from: node.id,
        to: childNode.id,
      });
    }
    data.nodes.push(node);

    return [node, data];
  }

  function onPasted(editor: monaco.editor.IStandaloneCodeEditor) {
    editor.onDidPaste(async (e) => {
      const clipboardContent = await readText();
      editor.executeEdits('pest-rs', [
        {
          range: e.range,
          text: clipboardContent,
        },
      ]);
    });
  }

  $effect(() => {
    if (leftPanel) {
      leftPanel
        .getModel()
        ?.onDidChangeContent(
          () => (grammar = leftPanel?.getModel()?.getLinesContent().join('\n') ?? ''),
        );
      onPasted(leftPanel);
    }
  });

  $effect(() => {
    if (rightPanel) {
      rightPanel
        .getModel()
        ?.onDidChangeContent(
          () => (input = rightPanel?.getModel()?.getLinesContent().join('\n') ?? ''),
        );
      onPasted(rightPanel);
    }
  });

  $effect(() => {
    if (networkElement && !network) {
      network = new Network(networkElement, networkData, {
        layout: {
          hierarchical: {
            direction: 'UD',
          },
        },
      });
    }
  });

  $effect(() => {
    updateGrammar(grammar);
  });

  $effect(() => {
    if (!selectedRule) {
      return;
    }

    parseInput(input, selectedRule);
  });
</script>

<main class="w-screen h-screen flex flex-col">
  <div class="h-2/3 flex">
    <Editor
      bind:editor={leftPanel}
      settings={{
        automaticLayout: true,
        theme: 'vs-dark',
        language: 'pest-rs',
      }}
      class="w-1/2 h-full"
    />
    <Editor
      bind:editor={rightPanel}
      settings={{
        automaticLayout: true,
        theme: 'vs-dark',
      }}
      class="w-1/2 h-full"
    />
  </div>

  <div class="h-1/3 flex flex-col w-full">
    <select bind:value={selectedRule}>
      {#each rules as rule}
        <option value={rule}>{rule}</option>
      {/each}
    </select>

    <div bind:this={networkElement} class="w-full h-full"></div>
  </div>
</main>
