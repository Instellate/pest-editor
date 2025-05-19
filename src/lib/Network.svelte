<script lang="ts">
  import { type Data, Network, type Options } from 'vis-network';

  let {
    options = {},
    data = $bindable({}),
    ...props
  }: { class?: string; data?: Data; options?: Options } = $props();

  let element: HTMLDivElement | undefined = undefined;
  let network: Network | undefined = undefined;

  $effect(() => {
    if (!element) {
       return;
    }

    if (network) {
      network.destroy();
    }

    network = new Network(element, data, options);
  });
</script>

<div class={props.class} bind:this={element}></div>
