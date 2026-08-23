<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { Check, Download, ExternalLink, X, Loader2 } from '@lucide/svelte';
</script>

<section class="engine-manager">
  <div class="header">
    <div>
      <h2>{appState.t('引擎管理', 'Engine Manager')}</h2>
      <p class="desc">
        {appState.t(
          '外部引擎在运行时探测系统已装路径。便携版(ffmpeg/pandoc)可自动下载安装,安装包引擎需手动下载安装。',
          'Engines are detected at runtime. Portable engines (ffmpeg/pandoc) can be auto-installed; installer engines need manual download.',
        )}
      </p>
    </div>
    <button class="close" onclick={() => appState.toggleEngineManager()} title={appState.t('返回工具', 'Back to tools')}>
      <X size={18} />
    </button>
  </div>

  <div class="engine-list">
    {#each appState.engineInstallInfos as eng (eng.binary)}
      <div class="engine-card" class:available={eng.available}>
        <div class="status-col">
          {#if appState.installingEngine === eng.binary}
            <Loader2 size={20} style="color: var(--ntx-primary); animation: ntx-spin 0.7s linear infinite" />
          {:else if eng.available}
            <Check size={20} style="color: var(--ntx-success)" />
          {:else}
            <span class="miss-dot"></span>
          {/if}
        </div>
        <div class="info-col">
          <div class="engine-name">
            <code>{eng.binary}</code>
            <span class="engine-desc">{eng.desc}</span>
            {#if eng.is_portable}
              <span class="tag portable">{appState.t('便携版', 'Portable')}</span>
            {:else}
              <span class="tag installer">{appState.t('安装包', 'Installer')}</span>
            {/if}
          </div>
          {#if eng.available && eng.install_path}
            <div class="engine-path">{eng.install_path}</div>
          {/if}
        </div>
        <div class="action-col">
          {#if eng.available}
            <span class="installed-tag">{appState.t('已安装', 'Installed')}</span>
          {:else if eng.is_portable}
            <button
              class="install-btn"
              onclick={() => appState.installEngine(eng.binary)}
              disabled={appState.installingEngine === eng.binary}
            >
              {#if appState.installingEngine === eng.binary}
                {appState.t('安装中…', 'Installing…')}
              {:else}
                <Download size={14} />
                {appState.t('安装', 'Install')}
              {/if}
            </button>
          {:else}
            <a href={eng.download_url} target="_blank" rel="noopener noreferrer" class="download-link">
              <Download size={14} />
              {appState.t('下载', 'Download')}
              <ExternalLink size={10} />
            </a>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <div class="footer-note">
    {appState.t(
      '便携版自动解压到 %APPDATA%/NexToolkit/engines/,安装后立即可用。安装包引擎需手动安装,安装后重启应用。',
      'Portable engines are extracted to %APPDATA%/NexToolkit/engines/. Installer engines need manual setup; restart app after installation.',
    )}
  </div>
</section>

<style>
  .engine-manager {
    overflow-y: auto;
    padding: var(--ntx-space-5);
    display: flex;
    flex-direction: column;
    gap: var(--ntx-space-4);
    height: 100%;
  }
  .header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--ntx-space-4);
  }
  .header h2 {
    margin: 0 0 var(--ntx-space-1);
    font-size: var(--ntx-text-lg);
    color: var(--ntx-fg);
  }
  .desc {
    margin: 0;
    font-size: var(--ntx-text-sm);
    color: var(--ntx-fg-muted);
    max-width: 500px;
  }
  .close {
    display: inline-flex;
    background: var(--ntx-surface-3);
    border: 1px solid var(--ntx-border);
    color: var(--ntx-fg-muted);
    cursor: pointer;
    padding: var(--ntx-space-2);
    border-radius: var(--ntx-radius-sm);
  }
  .close:hover {
    background: var(--ntx-surface-2);
    color: var(--ntx-fg);
  }
  .engine-list {
    display: flex;
    flex-direction: column;
    gap: var(--ntx-space-3);
  }
  .engine-card {
    display: flex;
    align-items: flex-start;
    gap: var(--ntx-space-4);
    padding: var(--ntx-space-4);
    border: 1px solid var(--ntx-border);
    border-radius: var(--ntx-radius-base);
    background: var(--ntx-surface);
  }
  .engine-card.available {
    border-color: color-mix(in oklch, var(--ntx-success) 30%, var(--ntx-border));
  }
  .status-col {
    flex-shrink: 0;
    padding-top: 2px;
  }
  .miss-dot {
    display: inline-block;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--ntx-danger);
  }
  .info-col {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--ntx-space-1);
  }
  .engine-name {
    display: flex;
    align-items: baseline;
    gap: var(--ntx-space-2);
    flex-wrap: wrap;
  }
  .engine-name code {
    font-family: var(--ntx-font-mono);
    font-size: var(--ntx-text-md);
    color: var(--ntx-fg);
  }
  .engine-desc {
    font-size: var(--ntx-text-sm);
    color: var(--ntx-fg-muted);
  }
  .tag {
    font-size: var(--ntx-text-xs);
    padding: 1px var(--ntx-space-1);
    border-radius: var(--ntx-radius-sm);
  }
  .tag.portable {
    background: color-mix(in oklch, var(--ntx-info) 15%, var(--ntx-surface));
    color: var(--ntx-info);
  }
  .tag.installer {
    background: var(--ntx-surface-3);
    color: var(--ntx-fg-subtle);
  }
  .engine-path {
    font-size: var(--ntx-text-xs);
    color: var(--ntx-fg-subtle);
    word-break: break-all;
  }
  .action-col {
    flex-shrink: 0;
  }
  .install-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--ntx-space-1);
    background: var(--ntx-primary);
    color: var(--ntx-primary-fg);
    border: none;
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: var(--ntx-text-sm);
  }
  .install-btn:hover:not(:disabled) {
    background: var(--ntx-primary-hover);
  }
  .install-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .download-link {
    display: inline-flex;
    align-items: center;
    gap: var(--ntx-space-1);
    background: var(--ntx-primary-soft);
    color: var(--ntx-primary);
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-radius: var(--ntx-radius-sm);
    text-decoration: none;
    font-size: var(--ntx-text-sm);
  }
  .download-link:hover {
    background: color-mix(in oklch, var(--ntx-primary) 20%, var(--ntx-surface));
  }
  .installed-tag {
    font-size: var(--ntx-text-sm);
    color: var(--ntx-success);
    padding: var(--ntx-space-2) var(--ntx-space-3);
  }
  .footer-note {
    font-size: var(--ntx-text-xs);
    color: var(--ntx-fg-subtle);
    padding: var(--ntx-space-3);
    background: var(--ntx-surface-2);
    border-radius: var(--ntx-radius-sm);
    line-height: 1.5;
  }
  @keyframes ntx-spin {
    to { transform: rotate(360deg); }
  }
</style>
