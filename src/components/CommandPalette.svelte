<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { Search } from '@lucide/svelte';
</script>

{#if appState.paletteOpen}
  <div
    class="palette-overlay"
    role="button"
    tabindex="-1"
    aria-label={appState.t('关闭面板', 'Close palette')}
    onclick={() => (appState.paletteOpen = false)}
    onkeydown={(e) => e.key === 'Enter' && (appState.paletteOpen = false)}
  >
    <div class="palette" role="presentation" onclick={(e) => e.stopPropagation()}>
      <div class="palette-input-wrap">
        <Search size={16} />
        <input class="palette-input" placeholder={appState.t('搜索工具…', 'Search tools…')} bind:value={appState.paletteQuery} />
      </div>
      <div class="palette-list">
        {#each appState.paletteTools as tool (tool.id)}
          <button class="palette-item" onclick={() => appState.openTool(tool)}>
            <span>{tool.name}</span>
            <span class="palette-desc">{tool.desc}</span>
          </button>
        {/each}
        {#if appState.paletteTools.length === 0}
          <div class="palette-empty">{appState.t('无匹配工具', 'No matching tools')}</div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .palette-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 12vh;
    z-index: 100;
  }
  .palette {
    background: var(--ntx-surface);
    border: 1px solid var(--ntx-border);
    border-radius: var(--ntx-radius-lg);
    width: 480px;
    max-width: 90vw;
    overflow: hidden;
    box-shadow: var(--ntx-shadow-lg);
  }
  .palette-input-wrap {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-2);
    padding: var(--ntx-space-3) var(--ntx-space-4);
    color: var(--ntx-fg-subtle);
    border-bottom: 1px solid var(--ntx-border);
  }
  .palette-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--ntx-fg);
    font-size: 14px;
    outline: none;
  }
  .palette-list {
    max-height: 320px;
    overflow-y: auto;
    padding: var(--ntx-space-1);
  }
  .palette-item {
    display: flex;
    justify-content: space-between;
    gap: var(--ntx-space-3);
    width: 100%;
    background: transparent;
    border: none;
    color: var(--ntx-fg);
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: 13px;
    text-align: left;
  }
  .palette-item:hover {
    background: var(--ntx-surface-2);
  }
  .palette-desc {
    color: var(--ntx-fg-subtle);
    font-size: 11px;
  }
  .palette-empty {
    color: var(--ntx-fg-subtle);
    padding: var(--ntx-space-4);
    text-align: center;
    font-size: 13px;
  }
</style>
