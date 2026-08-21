<script lang="ts">
  import { TOOLS, GROUP_LABEL, type Group, type Tool, type ToolParam } from './tools';
  import { invoke } from './bindings';

  let lang: 'zh' | 'en' = $state('zh');
  let query = $state('');
  let selectedTool = $state<Tool>(TOOLS[0]);
  let mainInput = $state('');
  let params = $state<Record<string, string>>({});
  let output = $state('');
  let error = $state('');
  let loading = $state(false);
  let copied = $state(false);

  const groups: Group[] = ['encode', 'convert', 'format', 'generate', 'text', 'crypto', 'nettime'];

  // 按搜索词过滤工具
  const filteredTools = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return TOOLS;
    return TOOLS.filter((t) => t.name.toLowerCase().includes(q) || t.desc.toLowerCase().includes(q));
  });

  // 选中工具时,初始化参数默认值
  function selectTool(tool: Tool) {
    selectedTool = tool;
    const defaults: Record<string, string> = {};
    for (const p of tool.params) {
      defaults[p.key] = p.default ?? '';
    }
    params = defaults;
    output = '';
    error = '';
  }

  async function run() {
    loading = true;
    error = '';
    output = '';
    try {
      const args: Record<string, unknown> = {};
      for (const p of selectedTool.params) {
        args[p.key] = p.kind === 'number' ? Number(params[p.key] ?? 0) : params[p.key] ?? '';
      }
      // diff 工具的"对比文本"参数用 other;主输入作 input
      if (selectedTool.needsMainInput) {
        args['input'] = mainInput;
      }
      output = await invoke(selectedTool.id, args);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function copyOutput() {
    if (!output) return;
    await navigator.clipboard.writeText(output);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }

  function isSvgOutput(): boolean {
    return selectedTool.id === 'qr_svg';
  }

  function t(zh: string, en: string): string {
    return lang === 'zh' ? zh : en;
  }
</script>

<header class="topbar">
  <div class="brand">NexToolkit</div>
  <div class="badge">100% {t('本地', 'Local')} · {t('文件不离本机', 'Files never leave')}</div>
  <input class="search" placeholder={t('搜索工具…', 'Search tools…')} bind:value={query} />
  <button class="lang" onclick={() => (lang = lang === 'zh' ? 'en' : 'zh')}>
    {lang === 'zh' ? 'EN' : '中'}
  </button>
</header>

<main class="layout">
  <nav class="sidebar">
    {#each groups as g}
      {@const tools = filteredTools.filter((t) => t.group === g)}
      {#if tools.length > 0}
        <div class="group-label">{lang === 'zh' ? GROUP_LABEL[g].zh : GROUP_LABEL[g].en}</div>
        {#each tools as tool}
          <button
            class="tool-btn"
            class:active={selectedTool.id === tool.id}
            onclick={() => selectTool(tool)}
          >
            {tool.name}
          </button>
        {/each}
      {/if}
    {/each}
  </nav>

  <section class="panel">
    <h2>{selectedTool.name}</h2>
    <p class="desc">{selectedTool.desc}</p>

    {#if selectedTool.params.length > 0}
      <div class="params">
        {#each selectedTool.params as p}
          <label class="param">
            <span>{p.label}</span>
            {#if p.kind === 'textarea'}
              <textarea rows="3" bind:value={params[p.key]} placeholder={p.placeholder ?? ''}></textarea>
            {:else if p.kind === 'select'}
              <select bind:value={params[p.key]}>
                {#each p.options ?? [] as opt}
                  <option value={opt}>{opt}</option>
                {/each}
              </select>
            {:else if p.kind === 'password'}
              <input type="password" bind:value={params[p.key]} placeholder={p.placeholder ?? ''} />
            {:else if p.kind === 'number'}
              <input type="number" bind:value={params[p.key]} />
            {:else}
              <input type="text" bind:value={params[p.key]} placeholder={p.placeholder ?? ''} />
            {/if}
          </label>
        {/each}
      </div>
    {/if}

    {#if selectedTool.needsMainInput}
      <textarea
        class="main-input"
        rows="8"
        placeholder={t('输入…', 'Input…')}
        bind:value={mainInput}
      ></textarea>
    {/if}

    <div class="actions">
      <button class="run" onclick={run} disabled={loading}>
        {loading ? t('运行中…', 'Running…') : t('运行', 'Run')}
      </button>
      {#if output}
        <button class="copy" onclick={copyOutput}>
          {copied ? '✓' : t('复制', 'Copy')}
        </button>
      {/if}
    </div>

    {#if error}
      <pre class="error">{error}</pre>
    {/if}

    {#if output}
      {#if isSvgOutput()}
        <div class="svg-out">{@html output}</div>
      {:else}
        <pre class="output">{output}</pre>
      {/if}
    {/if}
  </section>
</main>

<style>
  :global(*) { box-sizing: border-box; }
  :global(body) { margin: 0; font-family: system-ui, -apple-system, 'Segoe UI', sans-serif; background: #0f1115; color: #e4e6eb; }
  .topbar { display: flex; align-items: center; gap: 12px; padding: 10px 16px; background: #16181d; border-bottom: 1px solid #23262e; }
  .brand { font-weight: 700; font-size: 18px; }
  .badge { font-size: 12px; color: #7ce0a6; background: #1a2b22; padding: 3px 8px; border-radius: 4px; }
  .search { flex: 1; background: #0f1115; border: 1px solid #2a2d35; color: #e4e6eb; padding: 6px 10px; border-radius: 6px; }
  .lang { background: #23262e; border: 1px solid #2a2d35; color: #e4e6eb; padding: 5px 12px; border-radius: 6px; cursor: pointer; }
  .layout { display: grid; grid-template-columns: 220px 1fr; height: calc(100vh - 49px); }
  .sidebar { overflow-y: auto; padding: 8px; border-right: 1px solid #23262e; }
  .group-label { font-size: 11px; color: #6b7280; text-transform: uppercase; margin: 12px 4px 4px; letter-spacing: 0.5px; }
  .tool-btn { display: block; width: 100%; text-align: left; background: transparent; border: none; color: #c9ccd3; padding: 6px 10px; border-radius: 6px; cursor: pointer; font-size: 13px; }
  .tool-btn:hover { background: #1c1f25; }
  .tool-btn.active { background: #2563eb; color: #fff; }
  .panel { overflow-y: auto; padding: 20px 24px; }
  .panel h2 { margin: 0 0 4px; font-size: 18px; }
  .desc { color: #8b8f99; font-size: 13px; margin: 0 0 16px; }
  .params { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 10px; margin-bottom: 12px; }
  .param { display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: #8b8f99; }
  .param input, .param select, .param textarea { background: #0f1115; border: 1px solid #2a2d35; color: #e4e6eb; padding: 6px 8px; border-radius: 6px; font-size: 13px; }
  .main-input { width: 100%; background: #0f1115; border: 1px solid #2a2d35; color: #e4e6eb; padding: 10px; border-radius: 6px; font-family: 'Cascadia Code', Consolas, monospace; font-size: 13px; resize: vertical; }
  .actions { display: flex; gap: 8px; margin: 12px 0; }
  .run { background: #2563eb; color: #fff; border: none; padding: 8px 20px; border-radius: 6px; cursor: pointer; font-size: 14px; }
  .run:disabled { opacity: 0.5; }
  .copy { background: #23262e; border: 1px solid #2a2d35; color: #e4e6eb; padding: 8px 16px; border-radius: 6px; cursor: pointer; }
  .output { background: #0a0c10; border: 1px solid #23262e; padding: 12px; border-radius: 6px; overflow: auto; font-family: 'Cascadia Code', Consolas, monospace; font-size: 13px; white-space: pre-wrap; word-break: break-all; }
  .error { background: #2a1414; border: 1px solid #5c2020; color: #ffadad; padding: 10px; border-radius: 6px; font-family: monospace; white-space: pre-wrap; }
  .svg-out { background: #fff; border-radius: 6px; padding: 16px; display: flex; justify-content: center; }
  .svg-out :global(svg) { width: 240px; height: 240px; }
</style>
