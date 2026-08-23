// 共享响应状态(Svelte5 $state class 单例)
// 组件直接 import { appState },避免 6 组件间 prop drilling。
// Set 突变用整体重赋值,确保响应触发。

import { invoke } from '../bindings';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import hljs from 'highlight.js';
import type { ToolMetaDto, EngineStatusDto, EngineInstallInfoDto } from './types';
import { SUBGROUPS, SUBCATEGORY, BIDIRECTIONAL, MULTI_OUTPUT_SUBGROUPS, GROUPS } from './types';
import { escapeHtml } from './format';

const FAV_KEY = 'nextoolkit-favorites';
const RECENT_KEY = 'nextoolkit-recent';
const COLLAPSED_KEY = 'nextoolkit-collapsed-groups';
const EXPANDED_SUBGROUPS_KEY = 'nextoolkit-expanded-subgroups';
const THEME_KEY = 'nextoolkit-theme';
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

function loadExpandedSubgroups(): Set<string> {
  try {
    const raw = localStorage.getItem(EXPANDED_SUBGROUPS_KEY);
    return new Set(raw ? JSON.parse(raw) : []);
  } catch {
    return new Set();
  }
}

function loadTheme(): 'light' | 'dark' | 'auto' {
  try {
    const raw = localStorage.getItem(THEME_KEY);
    if (raw === 'light' || raw === 'dark' || raw === 'auto') return raw;
  } catch {}
  return 'auto';
}

class AppState {
  // ── 基础 ──────────────────────────────────────────
  tools = $state<ToolMetaDto[]>([]);
  textIds = $state<Set<string>>(new Set());
  selectedTool = $state<ToolMetaDto | null>(null);
  lang = $state<'zh' | 'en'>('zh');
  query = $state('');
  theme = $state<'light' | 'dark' | 'auto'>(loadTheme());

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

  // ── 视图切换(工具 / 引擎管理)──────────────────────
  selectedView = $state<'tools' | 'engines'>('tools');

  // ── 侧栏持久状态 ─────────────────────────────────
  favorites = $state<Set<string>>(loadFavorites());
  recent = $state<string[]>(loadRecent());
  collapsedGroups = $state<Set<string>>(loadCollapsed());
  expandedSubgroups = $state<Set<string>>(loadExpandedSubgroups());

  // ── 引擎状态(运行时探测)──────────────────────────
  engines = $state<EngineStatusDto[]>([]);
  engineInstallInfos = $state<EngineInstallInfoDto[]>([]);
  installingEngine = $state<string | null>(null);

  // ── 工作区(子分类 + tab 切工具 + 模式)──────────────
  selectedSubgroup = $state<string | null>(null);
  mode = $state<'encode' | 'decode'>('encode');
  /** 多结果并出(hash 类):每工具 id → 结果字符串 */
  multiOutputs = $state<Record<string, string>>({});

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

  /** 工具所属子分类 id(未映射则 null,显在父 group 兜底) */
  subgroupOf(tool: ToolMetaDto): string | null {
    return SUBCATEGORY[tool.id] ?? null;
  }

  /** 子分类下的工具(按 subgroup id 过滤) */
  toolsInSubgroup(subgroupId: string): ToolMetaDto[] {
    return this.tools.filter((tool) => SUBCATEGORY[tool.id] === subgroupId);
  }

  /** 当前选中工具是否双向(有 encode/decode 对) */
  isBidirectional = $derived.by(() => {
    const id = this.selectedTool?.id;
    if (!id) return false;
    return id in BIDIRECTIONAL || Object.values(BIDIRECTIONAL).includes(id);
  });

  /** 当前子分类是否多结果并出(hash 类) */
  isMultiOutput = $derived.by(() => {
    const sg = this.selectedSubgroup;
    return sg !== null && MULTI_OUTPUT_SUBGROUPS.has(sg);
  });

  /** 当前子分类的所有工具(用于多结果并出) */
  subgroupTools = $derived.by(() => {
    const sg = this.selectedSubgroup;
    if (!sg) return [];
    return this.toolsInSubgroup(sg);
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

  /** 探测引擎可用性 + 安装信息(失败静默,引擎状态非关键路径) */
  async loadEngines() {
    try {
      const [statuses, infos] = await Promise.all([
        invoke<EngineStatusDto[]>('list_engines'),
        invoke<EngineInstallInfoDto[]>('engine_install_infos'),
      ]);
      this.engines = statuses;
      this.engineInstallInfos = infos;
    } catch {
      this.engines = [];
      this.engineInstallInfos = [];
    }
  }

  /** 安装便携版引擎(ffmpeg/pandoc),完成后刷新状态 */
  async installEngine(engine: string) {
    this.installingEngine = engine;
    try {
      await invoke<string>('install_engine', { engine });
      await this.loadEngines();
    } finally {
      this.installingEngine = null;
    }
  }

  // ── 工具选择 ──────────────────────────────────────
  /** 纯选择:设当前工具 + 设 subgroup + 初始化参数默认值 + 清空输出(mainInput 保留) */
  selectTool(tool: ToolMetaDto) {
    this.selectedTool = tool;
    const sg = this.subgroupOf(tool);
    if (sg) this.selectedSubgroup = sg;
    const defaults: Record<string, string> = {};
    for (const p of tool.params) defaults[p.key] = p.default ?? '';
    this.params = defaults;
    this.files = {};
    this.output = '';
    this.error = '';
    this.multiOutputs = {};
    this.paletteOpen = false;
    // 若示例字典有值且 mainInput 为空,预填示例
    if (!this.mainInput && EXAMPLES[tool.id]) this.mainInput = EXAMPLES[tool.id];
  }

  /** Toggle subgroup 展开/折叠(左侧树 header 点击) */
  toggleSubgroup(id: string) {
    const next = new Set(this.expandedSubgroups);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    this.expandedSubgroups = next;
    localStorage.setItem(EXPANDED_SUBGROUPS_KEY, JSON.stringify([...this.expandedSubgroups]));
  }

  /** 展开 subgroup(选中工具时自动展开,不折叠) */
  expandSubgroup(id: string) {
    if (this.expandedSubgroups.has(id)) return;
    const next = new Set(this.expandedSubgroups);
    next.add(id);
    this.expandedSubgroups = next;
    localStorage.setItem(EXPANDED_SUBGROUPS_KEY, JSON.stringify([...this.expandedSubgroups]));
  }

  /** Subgroup 是否展开(搜索时强制展开有匹配的;当前选中工具的 subgroup 始终保持展开) */
  isSubgroupExpanded(id: string, hasMatchingTools: boolean): boolean {
    if (this.query.trim() && hasMatchingTools) return true;
    if (this.selectedSubgroup === id) return true;
    return this.expandedSubgroups.has(id);
  }

  /** 展开全部 subgroup + group */
  expandAllSubgroups() {
    const all = new Set(SUBGROUPS.map((s) => s.id));
    this.expandedSubgroups = all;
    this.collapsedGroups = new Set();
    localStorage.setItem(EXPANDED_SUBGROUPS_KEY, JSON.stringify([...all]));
    localStorage.setItem(COLLAPSED_KEY, '[]');
  }

  /** 收起全部 subgroup + group(保留当前选中工具的 group 展开) */
  collapseAllSubgroups() {
    this.expandedSubgroups = new Set();
    const currentGroup = this.selectedTool?.group;
    const collapsed = new Set(GROUPS.filter((g) => g !== currentGroup));
    this.collapsedGroups = collapsed;
    localStorage.setItem(EXPANDED_SUBGROUPS_KEY, '[]');
    localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...collapsed]));
  }

  /** 全部展开?(toggle 按钮状态判定) */
  allSubgroupsExpanded = $derived.by(() => {
    return SUBGROUPS.every((s) => this.expandedSubgroups.has(s.id));
  });

  /** 统一 toggle:全展开时收起,否则展开全部 */
  toggleAllSubgroups() {
    if (this.allSubgroupsExpanded) {
      this.collapseAllSubgroups();
    } else {
      this.expandAllSubgroups();
    }
  }

  /** tab 切工具(同 subgroup 内):保留 mainInput,切 selectedTool */
  switchTool(tool: ToolMetaDto) {
    if (this.selectedTool?.id === tool.id) return;
    this.selectTool(tool);
  }

  /** 切 encode/decode 模式:切到对应方向工具 + input/output 互换 */
  switchMode() {
    const id = this.selectedTool?.id;
    if (!id) return;
    const pair = BIDIRECTIONAL[id] ?? Object.entries(BIDIRECTIONAL).find(([, v]) => v === id)?.[0];
    if (!pair) return;
    const target = this.tools.find((tool) => tool.id === pair);
    if (!target) return;
    // 互换 input/output
    const prevOutput = this.output;
    this.selectTool(target);
    this.mode = target.id in BIDIRECTIONAL ? 'encode' : 'decode';
    if (prevOutput) this.mainInput = prevOutput;
  }

  /** Swap:input/output 互换 + 翻转模式(双向工具) */
  swap() {
    if (!this.output) return;
    const prev = this.output;
    this.output = this.mainInput;
    this.mainInput = prev;
    if (this.isBidirectional) this.switchMode();
  }

  /** 清空输入/输出 */
  clear() {
    this.mainInput = '';
    this.output = '';
    this.error = '';
    this.multiOutputs = {};
  }

  /** 用户点击打开:选择 + 计入最近 + 自动展开所属 subgroup。
   *  点已选中工具不清空输出/参数(只关闭命令面板),避免误操作丢失结果。 */
  openTool(tool: ToolMetaDto) {
    if (this.selectedTool?.id === tool.id) {
      this.paletteOpen = false;
      return;
    }
    this.selectTool(tool);
    this.addRecent(tool.id);
    const sg = this.subgroupOf(tool);
    if (sg) this.expandSubgroup(sg);
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
  async pickFile(key: string, multiple: boolean, directory = false) {
    const sel = await openDialog({ multiple, directory });
    this.files = { ...this.files, [key]: sel ? (Array.isArray(sel) ? sel : [sel]) : [] };
  }

  // ── 执行 ──────────────────────────────────────────
  async run() {
    const tool = this.selectedTool;
    if (!tool) return;
    this.loading = true;
    this.error = '';
    this.output = '';
    this.multiOutputs = {};
    try {
      // 多结果并出(hash 类):循环该 subgroup 所有文本工具,各调 run_tool
      if (this.isMultiOutput) {
        const tools = this.subgroupTools.filter((t) => this.textIds.has(t.id));
        const results: Record<string, string> = {};
        await Promise.all(
          tools.map(async (t) => {
            const args = t.params
              .filter((p) => p.kind !== 'file')
              .map((p) => [p.key, String(this.params[p.key] ?? p.default ?? '')] as [string, string]);
            try {
              const r = await invoke<string>('run_tool', { id: t.id, input: this.mainInput, args });
              results[t.id] = r;
            } catch (e) {
              results[t.id] = e instanceof Error ? e.message : String(e);
            }
          }),
        );
        this.multiOutputs = results;
        return;
      }
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

  toggleEngineManager() {
    this.selectedView = this.selectedView === 'engines' ? 'tools' : 'engines';
  }

  // ── i18n ──────────────────────────────────────────
  t(zh: string, en: string): string {
    return this.lang === 'zh' ? zh : en;
  }

  toggleLang() {
    this.lang = this.lang === 'zh' ? 'en' : 'zh';
  }

  // ── 主题 ──────────────────────────────────────────
  applyTheme() {
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.theme = this.theme;
    }
  }

  toggleTheme() {
    const order: Array<'light' | 'dark' | 'auto'> = ['auto', 'light', 'dark'];
    const idx = order.indexOf(this.theme);
    this.theme = order[(idx + 1) % order.length];
    localStorage.setItem(THEME_KEY, this.theme);
    this.applyTheme();
  }
}

// 示例预填字典(首屏不空,选工具时若 mainInput 为空则预填)
const EXAMPLES: Record<string, string> = {
  // encode
  base64_encode: 'Hello NexToolkit',
  base64_decode: 'SGVsbG8gTmV4VG9vbGtpdA==',
  base32_encode: 'Hello',
  base58_encode: 'Hello',
  base85_encode: 'Man ',
  url_encode: '你好 world?foo=bar',
  html_encode: '<a href="#">link & text</a>',
  hex_encode: 'AABB',
  jwt_decode: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c',
  punycode_encode: 'münchen.de',
  morse_encode: 'SOS',
  braille_encode: 'hello',
  zero_width_encode: 'secret',
  // convert
  json_to_yaml: '{"name":"NexToolkit","version":3,"tags":["rust","tauri","svelte"]}',
  json_to_toml: '{"name":"NexToolkit","version":3}',
  json_to_csv: '[{"name":"Alice","age":30},{"name":"Bob","age":25}]',
  numbase_convert: '255',
  case_convert: 'Hello World Example',
  md_to_html: '# Title\n\nHello **NexToolkit**\n',
  unit_convert: '1',
  // format
  json_format: '{"b":2,"a":1,"c":[3,2,1]}',
  sql_format: 'select*from t where a=1 and b=2',
  // generate
  hash: 'abc',
  hmac_compute: 'message',
  password_generate: '',
  // text
  text_stats: 'Hello World\nNexToolkit 本地工具集\n第二行',
  sort_lines: 'banana\napple\ncherry\nApple',
  dedup_lines: 'a\nb\na\nc\nb',
  reverse_text: 'Hello NexToolkit',
  regex_match: 'a12b3c45',
  diff_text: 'Hello World\nNexToolkit',
  text_replace: 'Hello World, Hello NexToolkit',
  text_escape: "echo 'hello $USER'",
  // crypto
  aes_gcm_encrypt: 'Secret message',
  rsa_sign: 'data to sign',
  bcrypt_hash: 'password',
  crc32: '123456789',
  // nettime
  ipcalc: '192.168.1.5/24',
  timestamp_to_human: '1700000000',
  cron_next: '0 * * * *',
  http_probe: 'https://example.com',
};

export const appState = new AppState();
