<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from './bindings';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import hljs from 'highlight.js';

  // 工具元数据 DTO(对齐 Rust ToolMetaDto/ParamSpecDto,snake_case 字段经 serde 直传)
  interface ParamSpecDto {
    key: string;
    kind: string; // text|textarea|select|number|password|bool|file
    label: string;
    default: string | null;
    options: string[];
    placeholder: string | null;
    multiple: boolean;
  }

  interface ToolMetaDto {
    id: string;
    name: string;
    desc: string;
    group: string;
    params: ParamSpecDto[];
    needs_main_input: boolean;
    output_kind: string; // text|svg|<highlight.js 语言名>
  }

  // 分组标签(原 tools.ts 内联,现随动态渲染迁移至此)
  const GROUP_LABEL: Record<string, { zh: string; en: string }> = {
    encode: { zh: '编解码', en: 'Encoders' },
    convert: { zh: '转换', en: 'Converters' },
    format: { zh: '格式化', en: 'Formatters' },
    generate: { zh: '生成器', en: 'Generators' },
    text: { zh: '文本', en: 'Text' },
    crypto: { zh: '加密', en: 'Crypto' },
    nettime: { zh: '网络/时间', en: 'Net/Time' },
    fileconv: { zh: '文件转换', en: 'Files' },
  };
  const groups: string[] = ['encode', 'convert', 'format', 'generate', 'text', 'crypto', 'nettime', 'fileconv'];

  let lang: 'zh' | 'en' = $state('zh');
  let query = $state('');
  let allTools = $state<ToolMetaDto[]>([]);
  let textIds = $state<Set<string>>(new Set());
  let selectedTool = $state<ToolMetaDto | null>(null);
  let mainInput = $state('');
  let params = $state<Record<string, string>>({});
  let files = $state<Record<string, string[]>>({});
  let output = $state('');
  let error = $state('');
  let loading = $state(false);
  let copied = $state(false);

  // Ctrl+K 命令面板
  let paletteOpen = $state(false);
  let paletteQuery = $state('');

  // 收藏工具(localStorage 持久化)
  const FAV_KEY = 'nextoolkit-favorites';
  let favorites = $state<Set<string>>(loadFavorites());

  onMount(async () => {
    try {
      const [textTools, fileTools] = await Promise.all([
        invoke<ToolMetaDto[]>('list_tools'),
        invoke<ToolMetaDto[]>('list_file_tools'),
      ]);
      textIds = new Set(textTools.map((t) => t.id));
      allTools = [...textTools, ...fileTools];
      if (allTools.length > 0) selectTool(allTools[0]);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });

  // 按搜索词过滤工具
  const filteredTools = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return allTools;
    return allTools.filter((t) => t.name.toLowerCase().includes(q) || t.desc.toLowerCase().includes(q));
  });

  // 命令面板过滤结果
  const paletteTools = $derived.by(() => {
    const q = paletteQuery.trim().toLowerCase();
    if (!q) return allTools;
    return allTools.filter((t) => t.name.toLowerCase().includes(q) || t.desc.toLowerCase().includes(q));
  });

  // 收藏工具列表(置顶显示)
  const favoriteTools = $derived(allTools.filter((t) => favorites.has(t.id)));

  // 输出语法高亮:按工具 output_kind 决定语言(text/svg 不高亮)
  const highlightedOutput = $derived.by(() => {
    if (!output || !selectedTool) return '';
    const kind = selectedTool.output_kind;
    if (kind === 'text' || kind === 'svg') return escapeHtml(output);
    try {
      return hljs.highlight(output, { language: kind }).value;
    } catch {
      return escapeHtml(output);
    }
  });

  function loadFavorites(): Set<string> {
    try {
      const raw = localStorage.getItem(FAV_KEY);
      return new Set(raw ? JSON.parse(raw) : []);
    } catch {
      return new Set();
    }
  }

  function saveFavorites() {
    localStorage.setItem(FAV_KEY, JSON.stringify([...favorites]));
  }

  function toggleFavorite(id: string) {
    const next = new Set(favorites);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    favorites = next;
    saveFavorites();
  }

  function escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  // Ctrl+K 切换命令面板
  function onKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      paletteOpen = !paletteOpen;
      paletteQuery = '';
    } else if (e.key === 'Escape' && paletteOpen) {
      paletteOpen = false;
    }
  }

  // 选中工具时,初始化参数默认值
  function selectTool(tool: ToolMetaDto) {
    selectedTool = tool;
    const defaults: Record<string, string> = {};
    for (const p of tool.params) {
      defaults[p.key] = p.default ?? '';
    }
    params = defaults;
    files = {};
    output = '';
    error = '';
    paletteOpen = false;
  }

  // 文件参数:调系统对话框选择,单选存单元素数组,多选存数组
  async function pickFile(key: string, multiple: boolean) {
    const sel = await openDialog({ multiple });
    files = { ...files, [key]: sel ? (Array.isArray(sel) ? sel : [sel]) : [] };
  }

  // 文件名展示(去目录,只留文件名,多个逗号分隔)
  function fileDisplay(key: string): string {
    const list = files[key] ?? [];
    return list.map((p) => p.split(/[\\/]/).pop() ?? p).join(', ');
  }

  // 执行:文本工具经 run_tool(id,input,args) 通用入口;文件工具经 invoke(id, args 对象)
  async function run() {
    if (!selectedTool) return;
    loading = true;
    error = '';
    output = '';
    try {
      let result: unknown;
      if (textIds.has(selectedTool.id)) {
        // 文本工具:args 为 Vec<(String,String)>,前端传 [[key,value],...]
        const args = selectedTool.params
          .filter((p) => p.kind !== 'file')
          .map((p) => [p.key, String(params[p.key] ?? '')] as [string, string]);
        result = await invoke<string>('run_tool', {
          id: selectedTool.id,
          input: mainInput,
          args,
        });
      } else {
        // 文件工具:args 对象,number 转 Number,file 取 files
        const args: Record<string, unknown> = {};
        for (const p of selectedTool.params) {
          if (p.kind === 'file') {
            args[p.key] = p.multiple ? (files[p.key] ?? []) : (files[p.key]?.[0] ?? '');
          } else if (p.kind === 'number') {
            args[p.key] = Number(params[p.key] ?? 0);
          } else {
            args[p.key] = params[p.key] ?? '';
          }
        }
        result = await invoke<unknown>(selectedTool.id, args);
      }
      output = Array.isArray(result) ? result.join('\n') : String(result);
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
    return selectedTool?.output_kind === 'svg';
  }

  function t(zh: string, en: string): string {
    return lang === 'zh' ? zh : en;
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if paletteOpen}
  <div class="palette-overlay" role="button" tabindex="-1" aria-label={t('关闭面板', 'Close palette')} onclick={() => (paletteOpen = false)} onkeydown={(e) => e.key === 'Enter' && (paletteOpen = false)}>
    <div class="palette" role="presentation" onclick={(e) => e.stopPropagation()}>
      <input class="palette-input" placeholder={t('搜索工具…', 'Search tools…')} bind:value={paletteQuery} />
      <div class="palette-list">
        {#each paletteTools as tool}
          <button class="palette-item" onclick={() => selectTool(tool)}>
            <span>{tool.name}</span>
            <span class="palette-desc">{tool.desc}</span>
          </button>
        {/each}
        {#if paletteTools.length === 0}
          <div class="palette-empty">{t('无匹配工具', 'No matching tools')}</div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<header class="topbar">
  <div class="brand">NexToolkit</div>
  <div class="badge">100% {t('本地', 'Local')} · {t('文件不离本机', 'Files never leave')}</div>
  <input class="search" placeholder={t('搜索工具… (Ctrl+K)', 'Search tools… (Ctrl+K)')} bind:value={query} />
  <button class="lang" onclick={() => (lang = lang === 'zh' ? 'en' : 'zh')}>
    {lang === 'zh' ? 'EN' : '中'}
  </button>
</header>

<main class="layout">
  <nav class="sidebar">
    {#if favoriteTools.length > 0}
      <div class="group-label">{t('收藏', 'Favorites')}</div>
      {#each favoriteTools as tool}
        <button
          class="tool-btn"
          class:active={selectedTool?.id === tool.id}
          onclick={() => selectTool(tool)}
        >
          {tool.name}
        </button>
      {/each}
    {/if}
    {#each groups as g}
      {@const tools = filteredTools.filter((t) => t.group === g)}
      {#if tools.length > 0}
        <div class="group-label">{lang === 'zh' ? GROUP_LABEL[g].zh : GROUP_LABEL[g].en}</div>
        {#each tools as tool}
          <button
            class="tool-btn"
            class:active={selectedTool?.id === tool.id}
            onclick={() => selectTool(tool)}
          >
            {tool.name}
          </button>
        {/each}
      {/if}
    {/each}
  </nav>

  <section class="panel">
    {#if selectedTool}
      <div class="tool-header">
        <h2>{selectedTool.name}</h2>
        <button class="fav-btn" class:active={favorites.has(selectedTool.id)} onclick={() => toggleFavorite(selectedTool.id)} title={t('收藏', 'Favorite')}>
          {favorites.has(selectedTool.id) ? '★' : '☆'}
        </button>
      </div>
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
                  {#each p.options as opt}
                    <option value={opt}>{opt}</option>
                  {/each}
                </select>
              {:else if p.kind === 'file'}
                <button class="file-pick" onclick={() => pickFile(p.key, p.multiple)}>
                  {t('选择文件', 'Choose file')}{p.multiple ? ` (${t('多选', 'multi')})` : ''}
                </button>
                {#if fileDisplay(p.key)}
                  <span class="file-name">{fileDisplay(p.key)}</span>
                {/if}
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

      {#if selectedTool.needs_main_input}
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
          <pre class="output"><code class="hljs">{@html highlightedOutput}</code></pre>
        {/if}
      {/if}
    {:else}
      <p class="desc">{t('加载工具中…', 'Loading tools…')}</p>
    {/if}
  </section>
</main>

<style>
  :global(*) { box-sizing: border-box; }

  /* highlight.js 暗色主题 */
  :global(.hljs) { color: #e4e6eb; }
  :global(.hljs-keyword) { color: #c678dd; }
  :global(.hljs-string) { color: #98c379; }
  :global(.hljs-number) { color: #d19a66; }
  :global(.hljs-comment) { color: #7f848e; font-style: italic; }
  :global(.hljs-attr) { color: #61afef; }
  :global(.hljs-tag) { color: #e06c75; }
  :global(.hljs-built_in) { color: #56b6c2; }
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
  .file-pick { background: #23262e; border: 1px solid #2a2d35; color: #e4e6eb; padding: 6px 10px; border-radius: 6px; cursor: pointer; font-size: 13px; }
  .file-name { color: #7ce0a6; font-size: 12px; word-break: break-all; }
  .main-input { width: 100%; background: #0f1115; border: 1px solid #2a2d35; color: #e4e6eb; padding: 10px; border-radius: 6px; font-family: 'Cascadia Code', Consolas, monospace; font-size: 13px; resize: vertical; }
  .actions { display: flex; gap: 8px; margin: 12px 0; }
  .run { background: #2563eb; color: #fff; border: none; padding: 8px 20px; border-radius: 6px; cursor: pointer; font-size: 14px; }
  .run:disabled { opacity: 0.5; }
  .copy { background: #23262e; border: 1px solid #2a2d35; color: #e4e6eb; padding: 8px 16px; border-radius: 6px; cursor: pointer; }
  .output { background: #0a0c10; border: 1px solid #23262e; padding: 12px; border-radius: 6px; overflow: auto; font-family: 'Cascadia Code', Consolas, monospace; font-size: 13px; white-space: pre-wrap; word-break: break-all; }
  .error { background: #2a1414; border: 1px solid #5c2020; color: #ffadad; padding: 10px; border-radius: 6px; font-family: monospace; white-space: pre-wrap; }
  .svg-out { background: #fff; border-radius: 6px; padding: 16px; display: flex; justify-content: center; }
  .svg-out :global(svg) { width: 240px; height: 240px; }
  .tool-header { display: flex; align-items: center; gap: 10px; }
  .fav-btn { background: transparent; border: none; color: #8b8f99; cursor: pointer; font-size: 18px; padding: 2px 6px; border-radius: 4px; }
  .fav-btn:hover { background: #1c1f25; }
  .fav-btn.active { color: #f5c518; }
  .palette-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: flex-start; justify-content: center; padding-top: 12vh; z-index: 100; }
  .palette { background: #16181d; border: 1px solid #2a2d35; border-radius: 8px; width: 480px; max-width: 90vw; overflow: hidden; }
  .palette-input { width: 100%; background: #0f1115; border: none; border-bottom: 1px solid #2a2d35; color: #e4e6eb; padding: 12px 14px; font-size: 14px; outline: none; }
  .palette-list { max-height: 320px; overflow-y: auto; padding: 6px; }
  .palette-item { display: flex; justify-content: space-between; gap: 12px; width: 100%; background: transparent; border: none; color: #c9ccd3; padding: 8px 12px; border-radius: 6px; cursor: pointer; font-size: 13px; text-align: left; }
  .palette-item:hover { background: #1c1f25; }
  .palette-desc { color: #6b7280; font-size: 11px; }
  .palette-empty { color: #6b7280; padding: 16px; text-align: center; font-size: 13px; }
</style>
