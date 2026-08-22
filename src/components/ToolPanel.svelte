<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { Star, Play, Copy, Check } from '@lucide/svelte';
  import ParamForm from './ParamForm.svelte';
  import OutputArea from './OutputArea.svelte';
</script>

<section class="panel">
  {#if appState.selectedTool}
    {@const tool = appState.selectedTool}
    <div class="tool-header">
      <h2>{tool.name}</h2>
      <button
        class="fav-btn"
        class:active={appState.favorites.has(tool.id)}
        onclick={() => appState.toggleFavorite(tool.id)}
        title={appState.t('收藏', 'Favorite')}
      >
        <Star size={18} />
      </button>
    </div>
    <p class="desc">{tool.desc}</p>

    <ParamForm {tool} />

    {#if tool.needs_main_input}
      <textarea
        class="main-input"
        rows="8"
        placeholder={appState.t('输入…', 'Input…')}
        bind:value={appState.mainInput}
      ></textarea>
    {/if}

    <div class="actions">
      <button class="run-btn" onclick={() => appState.run()} disabled={appState.loading}>
        <Play size={14} />
        {appState.loading ? appState.t('运行中…', 'Running…') : appState.t('运行', 'Run')}
      </button>
      {#if appState.output}
        <button class="copy-btn" onclick={() => appState.copyOutput()}>
          {#if appState.copied}
            <Check size={14} />{appState.t('已复制', 'Copied')}
          {:else}
            <Copy size={14} />{appState.t('复制', 'Copy')}
          {/if}
        </button>
      {/if}
    </div>

    <OutputArea />
  {:else}
    <p class="desc">{appState.t('加载工具中…', 'Loading tools…')}</p>
  {/if}
</section>

<style>
  .panel {
    overflow-y: auto;
    padding: var(--ntx-space-5) var(--ntx-space-6);
  }
  .tool-header {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-3);
  }
  .panel h2 {
    margin: 0;
    font-size: 18px;
    color: var(--ntx-fg);
  }
  .fav-btn {
    display: inline-flex;
    background: transparent;
    border: none;
    color: var(--ntx-fg-subtle);
    cursor: pointer;
    padding: var(--ntx-space-1);
    border-radius: var(--ntx-radius-sm);
    transition: color var(--ntx-transition), background var(--ntx-transition);
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
  .desc {
    color: var(--ntx-fg-muted);
    font-size: 13px;
    margin: var(--ntx-space-1) 0 var(--ntx-space-4);
  }
  .main-input {
    width: 100%;
    background: var(--ntx-bg);
    border: 1px solid var(--ntx-border);
    color: var(--ntx-fg);
    padding: var(--ntx-space-3);
    border-radius: var(--ntx-radius-base);
    font-family: var(--ntx-font-mono);
    font-size: 13px;
    resize: vertical;
  }
  .main-input:focus-visible {
    border-color: var(--ntx-primary);
    outline: none;
  }
  .actions {
    display: flex;
    gap: var(--ntx-space-2);
    margin: var(--ntx-space-3) 0;
  }
  .run-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--ntx-space-1);
    background: var(--ntx-primary);
    color: var(--ntx-primary-fg);
    border: none;
    padding: var(--ntx-space-2) var(--ntx-space-5);
    border-radius: var(--ntx-radius-base);
    cursor: pointer;
    font-size: 14px;
    transition: background var(--ntx-transition);
  }
  .run-btn:hover:not(:disabled) {
    background: var(--ntx-primary-hover);
  }
  .run-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .copy-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--ntx-space-1);
    background: var(--ntx-surface-3);
    border: 1px solid var(--ntx-border);
    color: var(--ntx-fg);
    padding: var(--ntx-space-2) var(--ntx-space-4);
    border-radius: var(--ntx-radius-base);
    cursor: pointer;
    font-size: 14px;
  }
  .copy-btn:hover {
    background: var(--ntx-surface-2);
  }
</style>
