<script lang="ts">
  import '../editor.scss';
  import { commands, events, type TokenTree } from '$lib/bindings';
  import { initializePest } from '$lib/monaco-pest';
  import Editor from '$lib/Editor.svelte';
  import Network from '$lib/Network.svelte';
  import * as monaco from 'monaco-editor';
  import debounce from 'lodash.debounce';
  import { type Edge, type Data as VisData, type Node } from 'vis-network';
  import { Pane, Splitpanes } from 'svelte-splitpanes';
  import ArrowDropdown from '$lib/ArrowDropdown.svelte';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { trace } from '@tauri-apps/plugin-log';
  import { load } from '@tauri-apps/plugin-store';

  if (monaco.languages.getEncodedLanguageId('pest-rs') === 0) {
    initializePest();

    const saveGrammar = debounce(commands.saveGrammarContent, 250);
    const saveInput = debounce(commands.saveGrammarInput, 250);

    monaco.editor.addEditorAction({
      id: 'save',
      label: 'Save file',
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS],
      run: async (editor) => {
        if (leftPanel?.getId() === editor.getId()) {
          trace('Saving grammar');
          await saveGrammar();
        } else if (rightPanel?.getId() === editor.getId()) {
          trace('Saving input');
          await saveInput();
        }
      },
    });
  }

  let leftPanel: monaco.editor.IStandaloneCodeEditor | undefined = $state(undefined);
  let rightPanel: monaco.editor.IStandaloneCodeEditor | undefined = $state(undefined);
  let networkData: VisData = $state({});

  let rules: string[] = $state([]);
  let selectedRule = $state('');
  let grammar = $state('');
  let input = $state('');
  let grammarUpdate = $state(0);

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
        ++grammarUpdate;
      }
    });
    commands.updateGrammarContent(grammar);
  }, 250);

  const parseInput = debounce((input: string, rule: string, _: number) => {
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
        } else {
          networkData = {};
        }
      }
    });
  }, 250);

  const updateInput = debounce(commands.updateGrammarInput, 250);

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
        background: '#1E1E1E',
        color: '#FFFFFF',
      },
    };
    data.nodes.push(node);

    const children = tree.children.map((c) => createData(c, labelCount));
    for (const [childNode, child] of children) {
      data.edges = data.edges.concat(child.edges);
      data.nodes = data.nodes.concat(child.nodes);

      data.edges.push({
        from: node.id,
        to: childNode.id,
      });
    }
    return [node, data];
  }

  $effect(() => {
    updateGrammar(grammar);
  });

  $effect(() => {
    updateInput(input);
  });

  $effect(() => {
    if (!selectedRule) {
      return;
    }

    parseInput(input, selectedRule, grammarUpdate);
  });

  $effect(() => {
    let window = getCurrentWebviewWindow();
    const listener = events.changeFileEvent(window).listen((e) => {
      leftPanel?.setValue(e.payload.grammar);
      rightPanel?.setValue(e.payload.input);
    });

    return async () => {
      trace('Detaching change file listener');
      (await listener)();
    };
  });

  $effect(() => {
    (async () => {
      const store = await load('state.json', { autoSave: false });
  
      const lastFile = await store.get('last-file');
      if (typeof lastFile === 'string') {
        commands.changeFile(lastFile);
      }
    })();
  });
</script>

<main class="w-screen h-screen flex flex-col">
  <Splitpanes horizontal={true} theme="vs-dark">
    <Pane>
      <Splitpanes theme="vs-dark">
        <Pane>
          <Editor
            bind:editor={leftPanel}
            bind:content={grammar}
            class="w-full h-full"
            settings={{
              automaticLayout: true,
              theme: 'vs-dark',
              language: 'pest-rs',
            }}
          />
        </Pane>
        <Pane>
          <Editor
            bind:editor={rightPanel}
            bind:content={input}
            class="w-full h-full"
            settings={{
              automaticLayout: true,
              theme: 'vs-dark',
            }}
          />
        </Pane>
      </Splitpanes>
    </Pane>
    <Pane class="relative panel-bg">
      <ArrowDropdown class="absolute m-1 z-2 pointer-events-none left-30" />
      <select
        bind:value={selectedRule}
        class="w-36 z-1 m-1 px-1 text-white bg-[#1e1e1e] appearance-none select-border rounded-sm absolute"
      >
        {#each rules as rule}
          <option value={rule}>{rule}</option>
        {/each}
      </select>

      <Network
        class="w-full h-full bg-[#1e1e1e]"
        bind:data={networkData}
        options={{
          layout: {
            hierarchical: {
              direction: 'UD',
              sortMethod: 'directed',
              shakeTowards: 'roots',
            },
          },

          physics: {
            hierarchicalRepulsion: {
              nodeDistance: 200,
            },
          },
        }}
      />
    </Pane>
  </Splitpanes>
</main>

<style>
  .select-border {
    border: 1px solid #37373d;
  }
</style>
