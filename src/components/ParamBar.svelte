<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { joinFileNames } from '../lib/format';
  import { FolderOpen, Folder } from '@lucide/svelte';
  import type { ToolMetaDto } from '../lib/types';

  let { tool }: { tool: ToolMetaDto } = $props();
</script>

{#if tool.params.length > 0}
  <div class="param-bar">
    {#each tool.params as p (p.key)}
      <label class="param">
        <span class="param-label">{p.label}</span>
        {#if p.kind === 'textarea'}
          <textarea rows="2" bind:value={appState.params[p.key]} placeholder={p.placeholder ?? ''}></textarea>
        {:else if p.kind === 'select'}
          <select bind:value={appState.params[p.key]}>
            {#each p.options as opt}
              <option value={opt}>{opt}</option>
            {/each}
          </select>
        {:else if p.kind === 'file'}
          <div class="file-row">
            <button class="file-pick" onclick={() => appState.pickFile(p.key, p.multiple, false)}>
              <FolderOpen size={14} />
              {appState.t('选文件', 'Files')}{p.multiple ? ` (${appState.t('多', 'multi')})` : ''}
            </button>
            <button class="file-pick" onclick={() => appState.pickFile(p.key, true, true)} title={appState.t('选文件夹', 'Pick folder')}>
              <Folder size={14} />
              {appState.t('文件夹', 'Folder')}
            </button>
          </div>
          {#if joinFileNames(appState.files[p.key] ?? [])}
            <span class="file-name">{joinFileNames(appState.files[p.key] ?? [])}</span>
          {/if}
        {:else if p.kind === 'password'}
          <input type="password" bind:value={appState.params[p.key]} placeholder={p.placeholder ?? ''} />
        {:else if p.kind === 'number'}
          <input type="number" bind:value={appState.params[p.key]} />
        {:else}
          <input type="text" bind:value={appState.params[p.key]} placeholder={p.placeholder ?? ''} />
        {/if}
      </label>
    {/each}
  </div>
{/if}

<style>
  .param-bar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ntx-space-3);
    align-items: flex-end;
  }
  .param {
    display: flex;
    flex-direction: column;
    gap: var(--ntx-space-1);
  }
  .param-label {
    font-size: 11px;
    color: var(--ntx-fg-subtle);
  }
  .param input,
  .param select,
  .param textarea {
    background: var(--ntx-bg);
    border: 1px solid var(--ntx-border);
    color: var(--ntx-fg);
    padding: var(--ntx-space-1) var(--ntx-space-2);
    border-radius: var(--ntx-radius-sm);
    font-size: 13px;
    font-family: inherit;
    min-width: 120px;
  }
  .param input:focus-visible,
  .param select:focus-visible,
  .param textarea:focus-visible {
    border-color: var(--ntx-primary);
  }
  .file-row {
    display: flex;
    gap: var(--ntx-space-1);
  }
  .file-pick {
    display: inline-flex;
    align-items: center;
    gap: var(--ntx-space-1);
    background: var(--ntx-surface-3);
    border: 1px solid var(--ntx-border);
    color: var(--ntx-fg);
    padding: var(--ntx-space-1) var(--ntx-space-2);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: 13px;
  }
  .file-pick:hover {
    background: var(--ntx-surface-2);
  }
  .file-name {
    color: var(--ntx-success);
    font-size: 11px;
    word-break: break-all;
  }
</style>
