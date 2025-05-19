<script lang="ts">
  import { type Data, Network, type Options } from 'vis-network';

  let {
    options = {},
    data = $bindable({}),
    ...props
  }: { class?: string; data?: Data; options?: Options } = $props();

  let element: HTMLDivElement;

  $effect(() => {
    const network = new Network(element, data, options);
    return () => {
      network.destroy();
    };
  });
</script>

<div class={props.class} bind:this={element}></div>
