<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { SearchX } from '@lucide/svelte';
</script>

<div class="empty">
  <SearchX size={32} />
  <p class="empty-title">{appState.t('未找到工具', 'No tools found')}</p>
  <p class="empty-desc">{appState.t('试试其他关键词,或清除搜索', 'Try another keyword or clear search')}</p>
  <button class="empty-clear" onclick={() => (appState.query = '')}>
    {appState.t('清除搜索', 'Clear search')}
  </button>
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--ntx-space-6);
    color: var(--ntx-fg-subtle);
    text-align: center;
  }
  .empty-title {
    margin: var(--ntx-space-3) 0 var(--ntx-space-1);
    font-size: 15px;
    color: var(--ntx-fg-muted);
  }
  .empty-desc {
    margin: 0 0 var(--ntx-space-3);
    font-size: 13px;
  }
  .empty-clear {
    background: var(--ntx-surface-3);
    border: 1px solid var(--ntx-border);
    color: var(--ntx-fg);
    padding: var(--ntx-space-2) var(--ntx-space-4);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: 13px;
  }
  .empty-clear:hover {
    background: var(--ntx-surface-2);
  }
</style>
