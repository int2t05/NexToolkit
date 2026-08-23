<script lang="ts">
  import { appState } from '../lib/state.svelte';
</script>

{#if appState.isBidirectional}
  <div class="mode-tabs">
    <button class:active={appState.mode === 'encode'} onclick={() => appState.mode !== 'encode' && appState.switchMode()}>
      {appState.t('编码', 'Encode')}
    </button>
    <button class:active={appState.mode === 'decode'} onclick={() => appState.mode !== 'decode' && appState.switchMode()}>
      {appState.t('解码', 'Decode')}
    </button>
  </div>
{/if}

<style>
  .mode-tabs {
    display: inline-flex;
    gap: var(--ntx-space-1);
    background: var(--ntx-surface-3);
    border: 1px solid var(--ntx-border);
    border-radius: var(--ntx-radius-sm);
    padding: var(--ntx-space-1);
  }
  .mode-tabs button {
    background: transparent;
    border: none;
    color: var(--ntx-fg-muted);
    padding: var(--ntx-space-1) var(--ntx-space-3);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: 12px;
    transition: background var(--ntx-transition), color var(--ntx-transition);
  }
  .mode-tabs button.active {
    background: var(--ntx-primary);
    color: var(--ntx-primary-fg);
  }
</style>
