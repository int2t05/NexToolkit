<script lang="ts">
  import { onMount } from 'svelte';
  import { appState } from './lib/state.svelte';
  import TopBar from './components/TopBar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import ToolWorkspace from './components/ToolWorkspace.svelte';
  import EmptyState from './components/EmptyState.svelte';
  import CommandPalette from './components/CommandPalette.svelte';
  import EngineManager from './components/EngineManager.svelte';

  const showEngines = $derived(appState.selectedView === 'engines');

  onMount(() => {
    appState.applyTheme();
    appState.loadTools();
  });

  // 搜索无结果时显空状态
  const noResults = $derived(
    appState.query.trim().length > 0 && appState.filteredTools.length === 0,
  );
</script>

<svelte:window onkeydown={(e) => appState.onKeydown(e)} />

<div class="app">
  <TopBar />
  <main class="layout">
    <Sidebar />
    {#if showEngines}
      <EngineManager />
    {:else if noResults}
      <EmptyState />
    {:else}
      <ToolWorkspace />
    {/if}
  </main>
</div>

<CommandPalette />

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .layout {
    display: grid;
    grid-template-columns: 240px 1fr;
    flex: 1;
    min-height: 0;
  }
</style>
