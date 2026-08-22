<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import type { ToolMetaDto } from '../lib/types';

  let { tools }: { tools: ToolMetaDto[] } = $props();
</script>

{#if tools.length > 1}
  <div class="tool-tabs">
    {#each tools as tool (tool.id)}
      <button
        class:active={appState.selectedTool?.id === tool.id}
        onclick={() => appState.switchTool(tool)}
        title={tool.desc}
      >
        {tool.name}
      </button>
    {/each}
  </div>
{/if}

<style>
  .tool-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ntx-space-1);
    border-bottom: 1px solid var(--ntx-border);
    padding-bottom: var(--ntx-space-2);
  }
  .tool-tabs button {
    background: transparent;
    border: 1px solid transparent;
    color: var(--ntx-fg-muted);
    padding: var(--ntx-space-1) var(--ntx-space-3);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: 13px;
    transition: background var(--ntx-transition), color var(--ntx-transition);
  }
  .tool-tabs button:hover {
    background: var(--ntx-surface-2);
    color: var(--ntx-fg);
  }
  .tool-tabs button.active {
    background: var(--ntx-primary-soft);
    color: var(--ntx-primary);
    border-color: var(--ntx-primary);
    font-weight: 500;
  }
</style>
