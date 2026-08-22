<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { Copy, Check, Download } from '@lucide/svelte';
  import { SUBGROUPS } from '../lib/types';

  // 查子分类 label(面包屑用)
  const subgroupMeta = $derived.by(() => {
    const id = appState.selectedSubgroup;
    if (!id) return null;
    return SUBGROUPS.find((s) => s.id === id) ?? null;
  });

  function downloadSvg() {
    if (!appState.output) return;
    const blob = new Blob([appState.output], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'nextool-output.svg';
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

{#if appState.error}
  <pre class="error">{appState.error}</pre>
{/if}

{#if appState.isMultiOutput && Object.keys(appState.multiOutputs).length > 0}
  <!-- 多结果并出(hash 类:每算法一行) -->
  <div class="multi-output">
    {#each Object.entries(appState.multiOutputs) as [id, result] (id)}
      <div class="result-row">
        <span class="result-label">{id}</span>
        <code class="result-value">{result || appState.t('(空)', '(empty)')}</code>
        {#if result}
          <button class="copy-mini" onclick={() => navigator.clipboard.writeText(result)} title={appState.t('复制', 'Copy')}>
            <Copy size={12} />
          </button>
        {/if}
      </div>
    {/each}
  </div>
{:else if appState.output}
  {#if appState.isSvgOutput}
    <!-- SVG 预览分栏 -->
    <div class="svg-preview">
      <div class="svg-canvas">{@html appState.output}</div>
      <button class="download-btn" onclick={downloadSvg}>
        <Download size={14} />{appState.t('下载 SVG', 'Download SVG')}
      </button>
    </div>
  {:else}
    <!-- 文本/高亮输出 -->
    <div class="output-box">
      <div class="output-header">
        <span class="output-lang">{appState.selectedTool?.output_kind ?? 'text'}</span>
        {#if appState.output}
          <button class="copy-btn" onclick={() => appState.copyOutput()}>
            {#if appState.copied}<Check size={14} />{:else}<Copy size={14} />{/if}
          </button>
        {/if}
      </div>
      <pre class="output"><code class="hljs">{@html appState.highlightedOutput}</code></pre>
    </div>
  {/if}
{/if}

<style>
  .error {
    background: var(--ntx-danger-soft);
    border: 1px solid var(--ntx-danger);
    color: var(--ntx-danger);
    padding: var(--ntx-space-3);
    border-radius: var(--ntx-radius-base);
    font-family: var(--ntx-font-mono);
    white-space: pre-wrap;
  }
  .multi-output {
    display: flex;
    flex-direction: column;
    gap: var(--ntx-space-2);
  }
  .result-row {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-2);
    background: var(--ntx-bg);
    border: 1px solid var(--ntx-border);
    border-radius: var(--ntx-radius-sm);
    padding: var(--ntx-space-2) var(--ntx-space-3);
  }
  .result-label {
    font-size: 11px;
    color: var(--ntx-fg-subtle);
    min-width: 80px;
    text-transform: uppercase;
  }
  .result-value {
    flex: 1;
    font-family: var(--ntx-font-mono);
    font-size: 13px;
    color: var(--ntx-fg);
    word-break: break-all;
  }
  .copy-mini {
    display: inline-flex;
    background: transparent;
    border: none;
    color: var(--ntx-fg-subtle);
    cursor: pointer;
    padding: 2px;
    border-radius: var(--ntx-radius-sm);
  }
  .copy-mini:hover {
    color: var(--ntx-fg);
  }
  .svg-preview {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--ntx-space-3);
    background: #fff;
    border: 1px solid var(--ntx-border);
    border-radius: var(--ntx-radius-base);
    padding: var(--ntx-space-4);
  }
  .svg-canvas :global(svg) {
    width: 240px;
    height: 240px;
  }
  .download-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--ntx-space-1);
    background: var(--ntx-surface-3);
    border: 1px solid var(--ntx-border);
    color: var(--ntx-fg);
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: 13px;
  }
  .output-box {
    display: flex;
    flex-direction: column;
    background: var(--ntx-bg);
    border: 1px solid var(--ntx-border);
    border-radius: var(--ntx-radius-base);
    overflow: hidden;
  }
  .output-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-bottom: 1px solid var(--ntx-border);
  }
  .output-lang {
    font-size: 11px;
    color: var(--ntx-fg-subtle);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .copy-btn {
    display: inline-flex;
    background: transparent;
    border: none;
    color: var(--ntx-fg-subtle);
    cursor: pointer;
    padding: 2px;
    border-radius: var(--ntx-radius-sm);
  }
  .copy-btn:hover {
    color: var(--ntx-fg);
  }
  .output {
    margin: 0;
    padding: var(--ntx-space-3);
    overflow: auto;
    font-family: var(--ntx-font-mono);
    font-size: 13px;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 400px;
  }
</style>
