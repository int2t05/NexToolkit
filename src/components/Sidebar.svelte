<script lang="ts">
  import { appState } from '../lib/state.svelte';
  import { GROUPS, GROUP_LABEL, GROUP_CAT_VAR, SUBGROUPS } from '../lib/types';
  import type { SubCategory } from '../lib/types';
  import { Search, ChevronDown, ChevronRight, Star, Clock, X, ChevronsDownUp, ChevronsUpDown, Braces, ArrowRightLeft, AlignLeft, Sparkles, Type, Shield, Globe, Network, FileBox } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import { GROUP_ICON } from '../lib/types';

  const ICON_MAP: Record<string, Component> = {
    Braces, ArrowRightLeft, AlignLeft, Sparkles, Type, Shield, Globe, Network, FileBox, Clock,
  };

  function clearQuery() {
    appState.query = '';
  }

  /** group 下的子分类(按 SUBGROUPS 顺序) */
  function subgroupsOf(group: string): SubCategory[] {
    return SUBGROUPS.filter((s) => s.parent === group);
  }

  /** 子分类是否选中 */
  function subgroupActive(subgroupId: string): boolean {
    return appState.selectedSubgroup === subgroupId;
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
    <button
      class="expand-btn"
      onclick={() => appState.toggleAllSubgroups()}
      title={appState.allSubgroupsExpanded ? appState.t('收起全部', 'Collapse all') : appState.t('展开全部', 'Expand all')}
    >
      {#if appState.allSubgroupsExpanded}<ChevronsDownUp size={14} />{:else}<ChevronsUpDown size={14} />{/if}
    </button>
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
          {#if ICON_MAP[GROUP_ICON[g]]}
            {@const IconComp = ICON_MAP[GROUP_ICON[g]]}
            <span class="group-icon" style={`color: var(${GROUP_CAT_VAR[g]})`}>
              <IconComp size={14} />
            </span>
          {/if}
          <span class="group-label">{appState.lang === 'zh' ? GROUP_LABEL[g].zh : GROUP_LABEL[g].en}</span>
          <span class="group-count">{toolsInGroup.length}</span>
          <span class="chevron">
            {#if expanded}<ChevronDown size={14} />{:else}<ChevronRight size={14} />{/if}
          </span>
        </button>
        {#if expanded}
          {@const subs = subgroupsOf(g)}
          {#if subs.length > 0}
            {#each subs as sub (sub.id)}
              {@const subTools = toolsInGroup.filter((t) => appState.subgroupOf(t) === sub.id)}
              {#if subTools.length > 0}
                {@const subExpanded = appState.isSubgroupExpanded(sub.id, subTools.length > 0)}
                <button
                  class="sub-header"
                  class:active={subgroupActive(sub.id)}
                  onclick={() => appState.toggleSubgroup(sub.id)}
                >
                  <span class="sub-chevron">
                    {#if subExpanded}<ChevronDown size={12} />{:else}<ChevronRight size={12} />{/if}
                  </span>
                  <span class="sub-label">{appState.lang === 'zh' ? sub.label.zh : sub.label.en}</span>
                  <span class="sub-count">{subTools.length}</span>
                </button>
                {#if subExpanded}
                  {#each subTools as tool (tool.id)}
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
            <!-- 未归入子分类的工具(兜底) -->
            {@const ungrouped = toolsInGroup.filter((t) => !appState.subgroupOf(t))}
            {#if ungrouped.length > 0}
              {#each ungrouped as tool (tool.id)}
                <button
                  class="tool-row"
                  class:active={appState.selectedTool?.id === tool.id}
                  onclick={() => appState.openTool(tool)}
                >
                  <span class="tool-name">{tool.name}</span>
                </button>
              {/each}
            {/if}
          {:else}
            <!-- 无子分类的 group 直接列工具 -->
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
      {/if}
    {/each}
  </div>

  {#if appState.engines.length > 0}
    <div class="engines-bar" title={appState.t('引擎可用性(绿=已装)', 'Engine availability (green=installed)')}>
      <span class="engines-label">{appState.t('引擎', 'Engines')}</span>
      {#each appState.engines as eng (eng.binary)}
        <span
          class="engine-dot"
          class:available={eng.available}
          title={`${eng.binary} — ${eng.desc} (${eng.available ? '✓' : '✗'})`}
        ></span>
      {/each}
    </div>
  {/if}

  <div class="sidebar-footer">
    <span class="ver">v0.4.0</span>
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
    font-size: var(--ntx-text-base);
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
  .expand-btn {
    display: inline-flex;
    background: transparent;
    border: none;
    color: var(--ntx-fg-subtle);
    cursor: pointer;
    padding: 2px;
    border-radius: var(--ntx-radius-sm);
  }
  .expand-btn:hover {
    color: var(--ntx-fg);
    background: var(--ntx-surface-2);
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
    font-size: var(--ntx-text-xs);
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
    font-size: var(--ntx-text-sm);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-top: var(--ntx-space-2);
  }
  .group-header:hover {
    background: var(--ntx-surface-2);
  }
  .sub-header {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-1);
    width: 100%;
    background: transparent;
    border: none;
    color: var(--ntx-fg-muted);
    padding: var(--ntx-space-1) var(--ntx-space-2) var(--ntx-space-1) var(--ntx-space-4);
    border-radius: var(--ntx-radius-sm);
    cursor: pointer;
    font-size: var(--ntx-text-base);
    transition: background var(--ntx-transition), color var(--ntx-transition);
  }
  .sub-header:hover {
    background: var(--ntx-surface-2);
    color: var(--ntx-fg);
  }
  .sub-header.active {
    color: var(--ntx-fg);
    font-weight: 500;
  }
  .sub-chevron {
    display: inline-flex;
    color: var(--ntx-fg-subtle);
    flex-shrink: 0;
  }
  .sub-label {
    flex: 1;
    text-align: left;
  }
  .sub-count {
    font-size: var(--ntx-text-xs);
    color: var(--ntx-fg-subtle);
  }
  .group-icon {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }
  .group-label {
    flex: 1;
    text-align: left;
  }
  .group-count {
    font-size: var(--ntx-text-xs);
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
    font-size: var(--ntx-text-base);
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
  .engines-bar {
    display: flex;
    align-items: center;
    gap: var(--ntx-space-1);
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-top: 1px solid var(--ntx-border);
  }
  .engines-label {
    font-size: var(--ntx-text-xs);
    color: var(--ntx-fg-subtle);
    margin-right: var(--ntx-space-1);
  }
  .engine-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ntx-danger);
    flex-shrink: 0;
    cursor: help;
  }
  .engine-dot.available {
    background: var(--ntx-success);
  }
  .sidebar-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--ntx-space-2) var(--ntx-space-3);
    border-top: 1px solid var(--ntx-border);
    font-size: var(--ntx-text-xs);
    color: var(--ntx-fg-subtle);
  }
</style>
