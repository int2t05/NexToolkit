<script lang="ts">
  import { appState } from '../lib/state.svelte';
</script>

{#if appState.error}
  <pre class="error">{appState.error}</pre>
{/if}

{#if appState.output}
  {#if appState.isSvgOutput}
    <div class="svg-out">{@html appState.output}</div>
  {:else}
    <pre class="output"><code class="hljs">{@html appState.highlightedOutput}</code></pre>
  {/if}
{/if}

<style>
  .output {
    background: var(--ntx-bg);
    border: 1px solid var(--ntx-border);
    padding: var(--ntx-space-4);
    border-radius: var(--ntx-radius-base);
    overflow: auto;
    font-family: var(--ntx-font-mono);
    font-size: 13px;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .error {
    background: var(--ntx-danger-soft);
    border: 1px solid var(--ntx-danger);
    color: var(--ntx-danger);
    padding: var(--ntx-space-3);
    border-radius: var(--ntx-radius-base);
    font-family: var(--ntx-font-mono);
    white-space: pre-wrap;
  }
  .svg-out {
    background: #fff;
    border-radius: var(--ntx-radius-base);
    padding: var(--ntx-space-4);
    display: flex;
    justify-content: center;
  }
  .svg-out :global(svg) {
    width: 240px;
    height: 240px;
  }
</style>
