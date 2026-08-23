<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { GROUP_LABEL, SUBGROUPS } from '../lib/types';
  import { Star, Play, ArrowLeftRight, Trash2 } from '@lucide/svelte';
  import ModeTabs from './ModeTabs.svelte';
  import ParamBar from './ParamBar.svelte';
  import SharedInput from './SharedInput.svelte';
  import OutputPanel from './OutputPanel.svelte';

  // 面包屑:group label / subgroup label
  const crumb = $derived.by(() => {
    const sg = appState.selectedSubgroup;
    if (!sg || !appState.selectedTool) return null;
    const sub = SUBGROUPS.find((s) => s.id === sg);
    if (!sub) return null;
    const glabel = appState.lang === 'zh' ? GROUP_LABEL[sub.parent].zh : GROUP_LABEL[sub.parent].en;
    const slabel = appState.lang === 'zh' ? sub.label.zh : sub.label.en;
    return { group: sub.parent, glabel, slabel };
  });
</script>

<section class="workspace">
  {#if appState.selectedTool}
    {@const tool = appState.selectedTool}
    <!-- 面包屑 -->
    {#if crumb}
      <div class="crumb">
        <span class="crumb-group">{crumb.glabel}</span>
        <span class="crumb-sep">/</span>
        <span class="crumb-sub">{crumb.slabel}</span>
      </div>
    {/if}

    <div class="tool-header">
      <div class="title-block">
        <h2>{tool.name}</h2>
        <p class="desc">{tool.desc}</p>
      </div>
      <button
        class="fav-btn"
        class:active={appState.favorites.has(tool.id)}
        onclick={() => appState.toggleFavorite(tool.id)}
        title={appState.t('收藏', 'Favorite')}
      >
        <Star size={18} />
      </button>
    </div>

    <div class="mode-row">
      <ModeTabs />
    </div>

    {#if tool.params.length > 0}
      <ParamBar {tool} />
    {/if}

    {#if tool.needs_main_input}
      <div class="io-grid">
        <SharedInput />
        <OutputPanel />
      </div>
    {:else}
      <!-- 文件工具:无主输入,只显输出 -->
      <OutputPanel />
    {/if}

    <div class="actions">
      <button class="run-btn" onclick={() => appState.run()} disabled={appState.loading}>
        {#if appState.loading}
          <span class="spinner"></span>
        {:else}
          <Play size={14} />
        {/if}
        {appState.loading ? appState.t('运行中…', 'Running…') : appState.t('运行', 'Run')}
      </button>
      {#if appState.isBidirectional && appState.output}
        <button class="swap-btn" onclick={() => appState.swap()} title={appState.t('互换输入输出', 'Swap I/O')}>
          <ArrowLeftRight size={14} />{appState.t('互换', 'Swap')}
        </button>
      {/if}
      {#if appState.mainInput || appState.output}
        <button class="clear-btn" onclick={() => appState.clear()}>
          <Trash2 size={14} />{appState.t('清空', 'Clear')}
        </button>
      {/if}
    </div>
  {:else}
    <p class="desc">{appState.t('选择左侧工具开始', 'Select a tool from the left to start')}</p>
  {/if}
</section>

<style>
  .workspace {
    overflow-y: auto;
    padding: var(--ntx-space-4) var(--ntx-space-5);
    display: flex;
    flex-direction: column;
    gap: var(--ntx-space-3);
    height: 100%;
  }
  .crumb {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-1);
    font-size: 12px;
    color: var(--ntx-fg-subtle);
  }
  .crumb-group {
    color: var(--ntx-fg-muted);
  }
  .crumb-sep {
    color: var(--ntx-fg-subtle);
  }
  .crumb-sub {
    color: var(--ntx-fg);
  }
  .tool-header {
    display: flex;
    align-items: flex-start;
    gap: var(--ntx-space-3);
  }
  .title-block {
    flex: 1;
  }
  .tool-header h2 {
    margin: 0;
    font-size: 18px;
    color: var(--ntx-fg);
  }
  .desc {
    margin: var(--ntx-space-1) 0 0;
    color: var(--ntx-fg-muted);
    font-size: 13px;
  }
  .fav-btn {
    display: inline-flex;
    background: transparent;
    border: none;
    color: var(--ntx-fg-subtle);
    cursor: pointer;
    padding: var(--ntx-space-1);
    border-radius: var(--ntx-radius-sm);
  }
  .fav-btn :global(svg) {
    fill: none;
    transition: fill var(--ntx-transition);
  }
  .fav-btn:hover {
    background: var(--ntx-surface-2);
  }
  .fav-btn.active {
    color: var(--ntx-star);
  }
  .fav-btn.active :global(svg) {
    fill: var(--ntx-star);
  }
  .mode-row {
    display: flex;
    gap: var(--ntx-space-2);
  }
  .io-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr;
    gap: var(--ntx-space-3);
    flex: 1;
    min-height: 0;
  }
  .actions {
    display: flex;
    gap: var(--ntx-space-2);
    margin-top: var(--ntx-space-2);
    position: sticky;
    bottom: 0;
    padding: var(--ntx-space-2) 0;
    background: color-mix(in oklch, var(--ntx-bg) 85%, transparent);
    backdrop-filter: blur(8px);
  }
  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--ntx-primary-fg);
    border-top-color: transparent;
    border-radius: 50%;
    animation: ntx-spin 0.7s linear infinite;
    display: inline-block;
  }
  @keyframes ntx-spin {
    to { transform: rotate(360deg); }
  }
  .run-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--ntx-space-1);
    background: var(--ntx-primary);
    color: var(--ntx-primary-fg);
    border: none;
    padding: var(--ntx-space-2) var(--ntx-space-4);
    border-radius: var(--ntx-radius-base);
    cursor: pointer;
    font-size: 14px;
  }
  .run-btn:hover:not(:disabled) {
    background: var(--ntx-primary-hover);
  }
  .run-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .swap-btn,
  .clear-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--ntx-space-1);
    background: var(--ntx-surface-3);
    border: 1px solid var(--ntx-border);
    color: var(--ntx-fg);
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-radius: var(--ntx-radius-base);
    cursor: pointer;
    font-size: 14px;
  }
  .swap-btn:hover,
  .clear-btn:hover {
    background: var(--ntx-surface-2);
  }
</style>
