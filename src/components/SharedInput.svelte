<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { Trash2 } from '@lucide/svelte';
</script>

<div class="shared-input">
  <div class="input-header">
    <span class="input-label">{appState.t('输入', 'Input')}</span>
    {#if appState.mainInput}
      <button class="clear-btn" onclick={() => (appState.mainInput = '')} title={appState.t('清空', 'Clear')}>
        <Trash2 size={14} />
      </button>
    {/if}
  </div>
  <textarea
    class="input-area"
    rows="10"
    placeholder={appState.t('输入内容…', 'Enter content…')}
    bind:value={appState.mainInput}
    spellcheck="false"
  ></textarea>
</div>

<style>
  .shared-input {
    display: flex;
    flex-direction: column;
    background: var(--ntx-bg);
    border: 1px solid var(--ntx-border);
    border-radius: var(--ntx-radius-base);
    overflow: hidden;
  }
  .input-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-bottom: 1px solid var(--ntx-border);
  }
  .input-label {
    font-size: 12px;
    color: var(--ntx-fg-subtle);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .clear-btn {
    display: inline-flex;
    background: transparent;
    border: none;
    color: var(--ntx-fg-subtle);
    cursor: pointer;
    padding: 2px;
    border-radius: var(--ntx-radius-sm);
  }
  .clear-btn:hover {
    color: var(--ntx-fg);
  }
  .input-area {
    flex: 1;
    width: 100%;
    background: transparent;
    border: none;
    color: var(--ntx-fg);
    padding: var(--ntx-space-3);
    font-family: var(--ntx-font-mono);
    font-size: 13px;
    resize: vertical;
    outline: none;
    min-height: 200px;
  }
  .input-area::placeholder {
    color: var(--ntx-fg-subtle);
  }
</style>
