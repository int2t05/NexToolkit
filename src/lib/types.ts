// 工具元数据 DTO(对齐 Rust ToolMetaDto/ParamSpecDto,snake_case 字段经 serde 直传)

export interface ParamSpecDto {
  key: string;
  kind: string; // text|textarea|select|number|password|bool|file
  label: string;
  default: string | null;
  options: string[];
  placeholder: string | null;
  multiple: boolean;
}

export interface ToolMetaDto {
  id: string;
  name: string;
  desc: string;
  group: string;
  params: ParamSpecDto[];
  needs_main_input: boolean;
  output_kind: string; // text|svg|<highlight.js 语言名>
}

// 工具分组(顺序驱动侧栏渲染,标签走 GROUP_LABEL)
export const GROUPS: string[] = [
  'encode',
  'convert',
  'format',
  'generate',
  'text',
  'crypto',
  'nettime',
  'fileconv',
];

export const GROUP_LABEL: Record<string, { zh: string; en: string }> = {
  encode: { zh: '编解码', en: 'Encoders' },
  convert: { zh: '转换', en: 'Converters' },
  format: { zh: '格式化', en: 'Formatters' },
  generate: { zh: '生成器', en: 'Generators' },
  text: { zh: '文本', en: 'Text' },
  crypto: { zh: '加密', en: 'Crypto' },
  nettime: { zh: '网络/时间', en: 'Net/Time' },
  fileconv: { zh: '文件转换', en: 'Files' },
};

// 分类色 CSS 变量名(对应 tokens.css --ntx-cat-*)
export const GROUP_CAT_VAR: Record<string, string> = {
  encode: '--ntx-cat-encode',
  convert: '--ntx-cat-convert',
  format: '--ntx-cat-format',
  generate: '--ntx-cat-generate',
  text: '--ntx-cat-text',
  crypto: '--ntx-cat-crypto',
  nettime: '--ntx-cat-nettime',
  fileconv: '--ntx-cat-fileconv',
};

// 引擎状态(运行时探测,对齐 Rust EngineStatusDto)
export interface EngineStatusDto {
  binary: string;
  desc: string;
  available: boolean;
}
