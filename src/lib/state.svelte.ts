// 共享响应状态(Svelte5 $state class 单例)
// 组件直接 import { appState },避免 6 组件间 prop drilling。
// Set 突变用整体重赋值,确保响应触发。

import { invoke } from '../bindings';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import hljs from 'highlight.js';
import type { ToolMetaDto, EngineStatusDto } from './types';
import { escapeHtml } from './format';

const FAV_KEY = 'nextoolkit-favorites';
const RECENT_KEY = 'nextoolkit-recent';
const COLLAPSED_KEY = 'nextoolkit-collapsed-groups';
const RECENT_MAX = 5;

function loadFavorites(): Set<string> {
  try {
    const raw = localStorage.getItem(FAV_KEY);
    return new Set(raw ? JSON.parse(raw) : []);
  } catch {
    return new Set();
  }
}

function loadRecent(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function loadCollapsed(): Set<string> {
  try {
    const raw = localStorage.getItem(COLLAPSED_KEY);
    return new Set(raw ? JSON.parse(raw) : []);
  } catch {
    return new Set();
  }
}

class AppState {
  // ── 基础 ──────────────────────────────────────────
  tools = $state<ToolMetaDto[]>([]);
  textIds = $state<Set<string>>(new Set());
  selectedTool = $state<ToolMetaDto | null>(null);
  lang = $state<'zh' | 'en'>('zh');
  query = $state('');

  // ── 输入/输出 ─────────────────────────────────────
  mainInput = $state('');
  params = $state<Record<string, string>>({});
  files = $state<Record<string, string[]>>({});
  output = $state('');
  error = $state('');
  loading = $state(false);
  copied = $state(false);

  // ── 命令面板 ──────────────────────────────────────
  paletteOpen = $state(false);
  paletteQuery = $state('');

  // ── 侧栏持久状态 ─────────────────────────────────
  favorites = $state<Set<string>>(loadFavorites());
  recent = $state<string[]>(loadRecent());
  collapsedGroups = $state<Set<string>>(loadCollapsed());

  // ── 引擎状态(运行时探测)──────────────────────────
  engines = $state<EngineStatusDto[]>([]);

  // ── 派生 ──────────────────────────────────────────
  filteredTools = $derived.by(() => {
    const q = this.query.trim().toLowerCase();
    if (!q) return this.tools;
    return this.tools.filter(
      (tool) => tool.name.toLowerCase().includes(q) || tool.desc.toLowerCase().includes(q),
    );
  });

  paletteTools = $derived.by(() => {
    const q = this.paletteQuery.trim().toLowerCase();
    if (!q) return this.tools;
    return this.tools.filter(
      (tool) => tool.name.toLowerCase().includes(q) || tool.desc.toLowerCase().includes(q),
    );
  });

  favoriteTools = $derived(this.tools.filter((tool) => this.favorites.has(tool.id)));

  /** 最近使用工具(按 id 解析回 ToolMetaDto,过滤已删除) */
  recentTools = $derived.by(() => {
    const byId = new Map(this.tools.map((tool) => [tool.id, tool]));
    return this.recent.map((id) => byId.get(id)).filter((tool): tool is ToolMetaDto => !!tool);
  });

  highlightedOutput = $derived.by(() => {
    if (!this.output || !this.selectedTool) return '';
    const kind = this.selectedTool.output_kind;
    if (kind === 'text' || kind === 'svg') return escapeHtml(this.output);
    try {
      return hljs.highlight(this.output, { language: kind }).value;
    } catch {
      return escapeHtml(this.output);
    }
  });

  isSvgOutput = $derived(this.selectedTool?.output_kind === 'svg');

  // ── 数据加载(由 App.svelte 的 onMount 调用)────────
  async loadTools() {
    try {
      const [textTools, fileTools] = await Promise.all([
        invoke<ToolMetaDto[]>('list_tools'),
        invoke<ToolMetaDto[]>('list_file_tools'),
      ]);
      this.textIds = new Set(textTools.map((tool) => tool.id));
      this.tools = [...textTools, ...fileTools];
      if (this.tools.length > 0 && !this.selectedTool) this.selectTool(this.tools[0]);
      this.loadEngines();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  /** 探测引擎可用性(失败静默,引擎状态非关键路径) */
  async loadEngines() {
    try {
      this.engines = await invoke<EngineStatusDto[]>('list_engines');
    } catch {
      this.engines = [];
    }
  }

  // ── 工具选择 ──────────────────────────────────────
  /** 纯选择:设当前工具 + 初始化参数默认值 + 清空输出(不计入最近) */
  selectTool(tool: ToolMetaDto) {
    this.selectedTool = tool;
    const defaults: Record<string, string> = {};
    for (const p of tool.params) defaults[p.key] = p.default ?? '';
    this.params = defaults;
    this.files = {};
    this.output = '';
    this.error = '';
    this.paletteOpen = false;
  }

  /** 用户点击打开:选择 + 计入最近(侧栏/面板点击用)。
   *  点已选中工具不清空输出/参数(只关闭命令面板),避免误操作丢失结果。 */
  openTool(tool: ToolMetaDto) {
    if (this.selectedTool?.id === tool.id) {
      this.paletteOpen = false;
      return;
    }
    this.selectTool(tool);
    this.addRecent(tool.id);
  }

  // ── 收藏 ──────────────────────────────────────────
  toggleFavorite(id: string) {
    const next = new Set(this.favorites);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    this.favorites = next;
    localStorage.setItem(FAV_KEY, JSON.stringify([...this.favorites]));
  }

  // ── 最近使用 ──────────────────────────────────────
  /** 追加到最近列表:已存在则不重排(保持首次使用顺序),新工具末尾追加,超上限挤掉最旧。
   *  不用 MRU 重排——重复点击同工具不应让 Recent 顺序来回跳变。 */
  addRecent(id: string) {
    if (this.recent.includes(id)) return;
    const next = [...this.recent, id].slice(-RECENT_MAX);
    this.recent = next;
    localStorage.setItem(RECENT_KEY, JSON.stringify(this.recent));
  }

  // ── 侧栏折叠 ──────────────────────────────────────
  toggleGroup(group: string) {
    const next = new Set(this.collapsedGroups);
    if (next.has(group)) next.delete(group);
    else next.add(group);
    this.collapsedGroups = next;
    localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...this.collapsedGroups]));
  }

  /** 分类是否展开:搜索时强制展开有匹配的;否则折叠状态优先(用户手动折叠的即使含选中工具也收起) */
  isGroupExpanded(group: string, toolsInGroup: ToolMetaDto[]): boolean {
    if (this.query.trim()) return toolsInGroup.length > 0;
    if (this.collapsedGroups.has(group)) return false;
    return true;
  }

  // ── 文件选择 ──────────────────────────────────────
  async pickFile(key: string, multiple: boolean) {
    const sel = await openDialog({ multiple });
    this.files = { ...this.files, [key]: sel ? (Array.isArray(sel) ? sel : [sel]) : [] };
  }

  // ── 执行 ──────────────────────────────────────────
  async run() {
    const tool = this.selectedTool;
    if (!tool) return;
    this.loading = true;
    this.error = '';
    this.output = '';
    try {
      let result: unknown;
      if (this.textIds.has(tool.id)) {
        // 文本工具:args 为 Vec<(String,String)>,前端传 [[key,value],...]
        const args = tool.params
          .filter((p) => p.kind !== 'file')
          .map((p) => [p.key, String(this.params[p.key] ?? '')] as [string, string]);
        result = await invoke<string>('run_tool', { id: tool.id, input: this.mainInput, args });
      } else {
        // 文件工具:args 对象,number 转 Number,file 取 files
        const args: Record<string, unknown> = {};
        for (const p of tool.params) {
          if (p.kind === 'file') {
            args[p.key] = p.multiple ? (this.files[p.key] ?? []) : (this.files[p.key]?.[0] ?? '');
          } else if (p.kind === 'number') {
            args[p.key] = Number(this.params[p.key] ?? 0);
          } else {
            args[p.key] = this.params[p.key] ?? '';
          }
        }
        result = await invoke<unknown>(tool.id, args);
      }
      this.output = Array.isArray(result) ? result.join('\n') : String(result);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  async copyOutput() {
    if (!this.output) return;
    await navigator.clipboard.writeText(this.output);
    this.copied = true;
    setTimeout(() => (this.copied = false), 1500);
  }

  // ── 命令面板 ──────────────────────────────────────
  onKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      this.paletteOpen = !this.paletteOpen;
      this.paletteQuery = '';
    } else if (e.key === 'Escape' && this.paletteOpen) {
      this.paletteOpen = false;
    }
  }

  togglePalette() {
    this.paletteOpen = !this.paletteOpen;
    this.paletteQuery = '';
  }

  // ── i18n ──────────────────────────────────────────
  t(zh: string, en: string): string {
    return this.lang === 'zh' ? zh : en;
  }

  toggleLang() {
    this.lang = this.lang === 'zh' ? 'en' : 'zh';
  }
}

export const appState = new AppState();
