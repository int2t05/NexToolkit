<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { Command, Languages } from '@lucide/svelte';
</script>

<header class="topbar">
  <div class="brand">NexToolkit</div>
  <div class="badge">100% {appState.t('本地', 'Local')} · {appState.t('文件不离本机', 'Files never leave')}</div>
  <div class="spacer"></div>
  <button
    class="icon-btn"
    onclick={() => appState.togglePalette()}
    title={appState.t('命令面板 (Ctrl+K)', 'Command palette (Ctrl+K)')}
  >
    <Command size={16} />
    <kbd>Ctrl K</kbd>
  </button>
  <button class="icon-btn" onclick={() => appState.toggleLang()} title={appState.t('切换语言', 'Toggle language')}>
    <Languages size={16} />
    {appState.lang === 'zh' ? 'EN' : '中'}
  </button>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-3);
    padding: var(--ntx-space-2) var(--ntx-space-4);
    background: var(--ntx-surface);
    border-bottom: 1px solid var(--ntx-border);
  }
  .brand {
    font-weight: 700;
    font-size: 18px;
    color: var(--ntx-fg);
  }
  .badge {
    font-size: var(--ntx-text-sm);
    color: var(--ntx-fg-muted);
    padding: 3px var(--ntx-space-2);
  }
  .spacer {
    flex: 1;
  }
  .icon-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--ntx-space-1);
    background: var(--ntx-surface-3);
    border: 1px solid var(--ntx-border);
    color: var(--ntx-fg-muted);
    padding: var(--ntx-space-1) var(--ntx-space-3);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: 13px;
    transition: background var(--ntx-transition), color var(--ntx-transition);
  }
  .icon-btn:hover {
    background: var(--ntx-surface-2);
    color: var(--ntx-fg);
  }
  kbd {
    font-family: var(--ntx-font-mono);
    font-size: 10px;
    background: var(--ntx-bg);
    padding: 1px var(--ntx-space-1);
    border-radius: var(--ntx-radius-sm);
    border: 1px solid var(--ntx-border);
  }
</style>
