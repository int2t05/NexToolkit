<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { GROUPS, GROUP_LABEL, GROUP_CAT_VAR } from '../lib/types';
  import { Search, ChevronDown, ChevronRight, Star, Clock, X } from '@lucide/svelte';

  function clearQuery() {
    appState.query = '';
  }
</script>

<nav class="sidebar">
  <div class="search-wrap">
    <Search size={15} />
    <input class="search" placeholder={appState.t('搜索工具…', 'Search tools…')} bind:value={appState.query} />
    {#if appState.query}
      <button class="clear" onclick={clearQuery} title={appState.t('清除', 'Clear')}>
        <X size={14} />
      </button>
    {/if}
  </div>

  <div class="tree">
    {#if appState.favoriteTools.length > 0}
      <div class="section-label">
        <Star size={12} fill="currentColor" />
        {appState.t('收藏', 'Favorites')}
      </div>
      {#each appState.favoriteTools as tool (tool.id)}
        <button
          class="tool-row pinned"
          class:active={appState.selectedTool?.id === tool.id}
          onclick={() => appState.openTool(tool)}
        >
          <span class="tool-name">{tool.name}</span>
        </button>
      {/each}
    {/if}

    {#if appState.recentTools.length > 0}
      <div class="section-label">
        <Clock size={12} />
        {appState.t('最近', 'Recent')}
      </div>
      {#each appState.recentTools as tool (tool.id)}
        <button
          class="tool-row recent"
          class:active={appState.selectedTool?.id === tool.id}
          onclick={() => appState.openTool(tool)}
        >
          <span class="tool-name">{tool.name}</span>
        </button>
      {/each}
    {/if}

    {#each GROUPS as g (g)}
      {@const toolsInGroup = appState.filteredTools.filter((t) => t.group === g)}
      {#if toolsInGroup.length > 0}
        {@const expanded = appState.isGroupExpanded(g, toolsInGroup)}
        <button class="group-header" onclick={() => appState.toggleGroup(g)}>
          <span class="cat-dot" style={`background: var(${GROUP_CAT_VAR[g]})`}></span>
          <span class="group-label">{appState.lang === 'zh' ? GROUP_LABEL[g].zh : GROUP_LABEL[g].en}</span>
          <span class="group-count">{toolsInGroup.length}</span>
          <span class="chevron">
            {#if expanded}<ChevronDown size={14} />{:else}<ChevronRight size={14} />{/if}
          </span>
        </button>
        {#if expanded}
          {#each toolsInGroup as tool (tool.id)}
            <button
              class="tool-row"
              class:active={appState.selectedTool?.id === tool.id}
              onclick={() => appState.openTool(tool)}
            >
              <span class="tool-name">{tool.name}</span>
            </button>
          {/each}
        {/if}
      {/if}
    {/each}
  </div>

  <div class="sidebar-footer">
    <span class="ver">v0.3.0</span>
    <span class="local-badge">100% {appState.t('本地', 'Local')}</span>
  </div>
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--ntx-border);
    background: var(--ntx-surface);
    min-height: 0;
  }
  .search-wrap {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-2);
    padding: var(--ntx-space-2) var(--ntx-space-3);
    color: var(--ntx-fg-subtle);
    border-bottom: 1px solid var(--ntx-border);
  }
  .search {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--ntx-fg);
    font-size: 13px;
    outline: none;
  }
  .search::placeholder {
    color: var(--ntx-fg-subtle);
  }
  .clear {
    display: inline-flex;
    background: transparent;
    border: none;
    color: var(--ntx-fg-subtle);
    cursor: pointer;
    padding: 2px;
    border-radius: var(--ntx-radius-sm);
  }
  .clear:hover {
    color: var(--ntx-fg);
  }
  .tree {
    flex: 1;
    overflow-y: auto;
    padding: var(--ntx-space-2);
    min-height: 0;
  }
  .section-label {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-1);
    font-size: 11px;
    color: var(--ntx-fg-subtle);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: var(--ntx-space-3) var(--ntx-space-2) var(--ntx-space-1);
  }
  .group-header {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-2);
    width: 100%;
    background: transparent;
    border: none;
    color: var(--ntx-fg-muted);
    padding: var(--ntx-space-2) var(--ntx-space-2);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-top: var(--ntx-space-2);
  }
  .group-header:hover {
    background: var(--ntx-surface-2);
  }
  .cat-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .group-label {
    flex: 1;
    text-align: left;
  }
  .group-count {
    font-size: 10px;
    color: var(--ntx-fg-subtle);
  }
  .chevron {
    display: inline-flex;
    color: var(--ntx-fg-subtle);
  }
  .tool-row {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    color: var(--ntx-fg-muted);
    padding: var(--ntx-space-2) var(--ntx-space-2) var(--ntx-space-2) var(--ntx-space-5);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: 13px;
    transition: background var(--ntx-transition), color var(--ntx-transition);
  }
  .tool-row:hover {
    background: var(--ntx-surface-2);
    color: var(--ntx-fg);
  }
  .tool-row.active {
    background: var(--ntx-primary-soft);
    color: var(--ntx-primary);
    font-weight: 500;
  }
  .sidebar-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-top: 1px solid var(--ntx-border);
    font-size: 11px;
    color: var(--ntx-fg-subtle);
  }
  .local-badge {
    color: var(--ntx-success);
  }
</style>
