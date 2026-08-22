<script lang="ts">
  import { onMount } from 'svelte';
  import { appState } from './lib/state.svelte';
  import TopBar from './components/TopBar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import ToolPanel from './components/ToolPanel.svelte';
  import CommandPalette from './components/CommandPalette.svelte';

  onMount(() => {
    appState.loadTools();
  });
</script>

<svelte:window onkeydown={(e) => appState.onKeydown(e)} />

<div class="app">
  <TopBar />
  <main class="layout">
    <Sidebar />
    <ToolPanel />
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
