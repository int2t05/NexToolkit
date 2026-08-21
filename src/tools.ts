// 工具元数据:声明式 schema 驱动 UI 渲染与 invoke 调用
// 新增工具只需在此追加一项,无需改 UI 组件

export type ParamKind = 'text' | 'textarea' | 'select' | 'number' | 'password' | 'file';

export interface ToolParam {
  key: string;
  label: string;
  kind: ParamKind;
  options?: string[]; // select 选项
  default?: string;
  placeholder?: string;
  multiple?: boolean; // file 专用:多选
}

export interface Tool {
  id: string;          // invoke command 名(蛇形)
  group: Group;
  name: string;
  desc: string;
  params: ToolParam[]; // 不含主输入文本(主输入固定为 textarea)
  needsMainInput: boolean; // 是否需要主文本输入(生成类工具不需要)
}

export type Group =
  | 'encode' | 'convert' | 'format' | 'generate' | 'text' | 'crypto' | 'nettime' | 'fileconv';

export const GROUP_LABEL: Record<Group, { zh: string; en: string }> = {
  encode: { zh: '编解码', en: 'Encoders' },
  convert: { zh: '转换', en: 'Converters' },
  format: { zh: '格式化', en: 'Formatters' },
  generate: { zh: '生成器', en: 'Generators' },
  text: { zh: '文本', en: 'Text' },
  crypto: { zh: '加密', en: 'Crypto' },
  nettime: { zh: '网络/时间', en: 'Net/Time' },
  fileconv: { zh: '文件转换', en: 'Files' },
};

// 工具清单:与 CLI 子命令一一对应,invoke 命令名为蛇形
export const TOOLS: Tool[] = [
  // ---- encode ----
  { id: 'base64_encode', group: 'encode', name: 'Base64 编码', desc: 'Base64 标准编码', params: [], needsMainInput: true },
  { id: 'base64_decode', group: 'encode', name: 'Base64 解码', desc: 'Base64 标准解码', params: [], needsMainInput: true },
  { id: 'url_encode', group: 'encode', name: 'URL 编码', desc: '百分号编码', params: [], needsMainInput: true },
  { id: 'url_decode', group: 'encode', name: 'URL 解码', desc: '百分号解码', params: [], needsMainInput: true },
  { id: 'html_encode', group: 'encode', name: 'HTML 编码', desc: 'HTML 实体编码', params: [], needsMainInput: true },
  { id: 'html_decode', group: 'encode', name: 'HTML 解码', desc: 'HTML 实体解码', params: [], needsMainInput: true },
  { id: 'hex_encode', group: 'encode', name: 'Hex 编码', desc: '十六进制编码', params: [], needsMainInput: true },
  { id: 'hex_decode', group: 'encode', name: 'Hex 解码', desc: '十六进制解码', params: [], needsMainInput: true },
  { id: 'jwt_decode', group: 'encode', name: 'JWT 解码', desc: '解析 header/payload(不验签)', params: [], needsMainInput: true },

  // ---- convert ----
  { id: 'json_to_yaml', group: 'convert', name: 'JSON → YAML', desc: 'JSON 转 YAML', params: [], needsMainInput: true },
  { id: 'yaml_to_json', group: 'convert', name: 'YAML → JSON', desc: 'YAML 转 JSON', params: [], needsMainInput: true },
  { id: 'json_to_toml', group: 'convert', name: 'JSON → TOML', desc: 'JSON 转 TOML', params: [], needsMainInput: true },
  { id: 'toml_to_json', group: 'convert', name: 'TOML → JSON', desc: 'TOML 转 JSON', params: [], needsMainInput: true },
  { id: 'json_to_csv', group: 'convert', name: 'JSON → CSV', desc: 'JSON 数组转 CSV', params: [], needsMainInput: true },
  { id: 'csv_to_json', group: 'convert', name: 'CSV → JSON', desc: 'CSV 转 JSON 数组', params: [], needsMainInput: true },
  { id: 'md_to_html', group: 'convert', name: 'Markdown → HTML', desc: 'Markdown 转 HTML', params: [], needsMainInput: true },
  {
    id: 'numbase_convert', group: 'convert', name: '进制转换', desc: '任意进制互转(2..36)',
    params: [
      { key: 'from', label: '源进制', kind: 'number', default: '10' },
      { key: 'to', label: '目标进制', kind: 'number', default: '16' },
    ], needsMainInput: true,
  },

  // ---- format ----
  { id: 'json_format', group: 'format', name: 'JSON 美化', desc: '2 空格缩进', params: [], needsMainInput: true },
  { id: 'json_minify', group: 'format', name: 'JSON 压缩', desc: '紧凑输出', params: [], needsMainInput: true },
  { id: 'sql_format', group: 'format', name: 'SQL 美化', desc: '关键字大写', params: [], needsMainInput: true },
  { id: 'xml_format', group: 'format', name: 'XML 美化', desc: '2 空格缩进', params: [], needsMainInput: true },
  { id: 'xml_minify', group: 'format', name: 'XML 压缩', desc: '去空白', params: [], needsMainInput: true },
  { id: 'css_minify', group: 'format', name: 'CSS 压缩', desc: '去注释空白', params: [], needsMainInput: true },

  // ---- generate ----
  { id: 'uuid_v4', group: 'generate', name: 'UUID v4', desc: '随机 UUID', params: [], needsMainInput: false },
  { id: 'uuid_v7', group: 'generate', name: 'UUID v7', desc: '基于时间戳', params: [], needsMainInput: false },
  {
    id: 'hash', group: 'generate', name: '哈希', desc: 'MD5/SHA1/SHA256/SHA512',
    params: [{ key: 'algo', label: '算法', kind: 'select', options: ['md5', 'sha1', 'sha256', 'sha512'], default: 'sha256' }],
    needsMainInput: true,
  },
  {
    id: 'hmac_compute', group: 'generate', name: 'HMAC', desc: 'HMAC 消息认证码',
    params: [
      { key: 'algo', label: '算法', kind: 'select', options: ['md5', 'sha1', 'sha256', 'sha512'], default: 'sha256' },
      { key: 'key', label: '密钥', kind: 'text', placeholder: 'HMAC 密钥' },
    ], needsMainInput: true,
  },
  {
    id: 'password_generate', group: 'generate', name: '密码生成', desc: '随机密码',
    params: [
      { key: 'length', label: '长度', kind: 'number', default: '16' },
      { key: 'upper', label: '大写', kind: 'select', options: ['true', 'false'], default: 'true' },
      { key: 'lower', label: '小写', kind: 'select', options: ['true', 'false'], default: 'true' },
      { key: 'digits', label: '数字', kind: 'select', options: ['true', 'false'], default: 'true' },
      { key: 'symbols', label: '符号', kind: 'select', options: ['true', 'false'], default: 'false' },
    ], needsMainInput: false,
  },
  {
    id: 'lorem_ipsum', group: 'generate', name: 'Lorem Ipsum', desc: '占位文本',
    params: [{ key: 'paragraphs', label: '段落数', kind: 'number', default: '3' }], needsMainInput: false,
  },
  { id: 'qr_svg', group: 'generate', name: '二维码 SVG', desc: '生成 SVG 二维码', params: [], needsMainInput: true },

  // ---- text ----
  {
    id: 'case_convert', group: 'text', name: '大小写转换', desc: 'snake/camel/kebab 等',
    params: [{ key: 'mode', label: '模式', kind: 'select', options: ['upper', 'lower', 'title', 'snake', 'camel', 'kebab'], default: 'snake' }],
    needsMainInput: true,
  },
  { id: 'sort_lines', group: 'text', name: '行排序', desc: '升序排序', params: [], needsMainInput: true },
  { id: 'dedup_lines', group: 'text', name: '行去重', desc: '保序去重', params: [], needsMainInput: true },
  { id: 'reverse_text', group: 'text', name: '文本反转', desc: '按 Unicode 字符', params: [], needsMainInput: true },
  {
    id: 'regex_match', group: 'text', name: '正则匹配', desc: '每匹配一行',
    params: [{ key: 'pattern', label: '正则', kind: 'text', placeholder: '\\d+' }], needsMainInput: true,
  },
  {
    id: 'regex_replace', group: 'text', name: '正则替换', desc: '支持 $0/$1',
    params: [
      { key: 'pattern', label: '正则', kind: 'text' },
      { key: 'replacement', label: '替换', kind: 'text' },
    ], needsMainInput: true,
  },
  {
    id: 'diff_text', group: 'text', name: '文本 Diff', desc: 'unified diff',
    params: [{ key: 'other', label: '对比文本', kind: 'textarea' }], needsMainInput: true,
  },

  // ---- crypto ----
  {
    id: 'aes_gcm_encrypt', group: 'crypto', name: 'AES 加密', desc: 'AES-256-GCM',
    params: [{ key: 'password', label: '口令', kind: 'password' }], needsMainInput: true,
  },
  {
    id: 'aes_gcm_decrypt', group: 'crypto', name: 'AES 解密', desc: 'AES-256-GCM',
    params: [{ key: 'password', label: '口令', kind: 'password' }], needsMainInput: true,
  },
  {
    id: 'rsa_keygen', group: 'crypto', name: 'RSA 密钥对', desc: '生成 PEM 密钥对',
    params: [{ key: 'bits', label: '位数', kind: 'number', default: '2048' }], needsMainInput: false,
  },
  {
    id: 'rsa_encrypt', group: 'crypto', name: 'RSA 加密', desc: 'RSA-OAEP/SHA256',
    params: [{ key: 'pubPem', label: '公钥 PEM', kind: 'textarea' }], needsMainInput: true,
  },
  {
    id: 'rsa_decrypt', group: 'crypto', name: 'RSA 解密', desc: 'RSA-OAEP/SHA256',
    params: [{ key: 'privPem', label: '私钥 PEM', kind: 'textarea' }], needsMainInput: true,
  },
  {
    id: 'pbkdf2', group: 'crypto', name: 'PBKDF2', desc: '密钥派生',
    params: [
      { key: 'salt', label: 'salt', kind: 'text' },
      { key: 'iterations', label: '迭代', kind: 'number', default: '100000' },
    ], needsMainInput: true,
  },
  {
    id: 'argon2', group: 'crypto', name: 'Argon2', desc: 'Argon2id 派生',
    params: [{ key: 'salt', label: 'salt(≥8字节)', kind: 'text' }], needsMainInput: true,
  },

  // ---- nettime ----
  { id: 'ipcalc', group: 'nettime', name: 'IP 子网计算', desc: 'CIDR 解析', params: [], needsMainInput: true },
  {
    id: 'timestamp_to_human', group: 'nettime', name: '时间戳→可读', desc: 'Unix 秒转可读时间',
    params: [{ key: 'tz', label: '时区', kind: 'text', default: 'UTC', placeholder: 'Asia/Shanghai' }], needsMainInput: true,
  },
  {
    id: 'timestamp_from_human', group: 'nettime', name: '可读→时间戳', desc: '可读时间转 Unix 秒',
    params: [{ key: 'tz', label: '时区', kind: 'text', default: 'UTC' }], needsMainInput: true,
  },
  {
    id: 'cron_next', group: 'nettime', name: 'cron 下次触发', desc: '接下来 N 次',
    params: [{ key: 'count', label: '次数', kind: 'number', default: '3' }], needsMainInput: true,
  },
  {
    id: 'dns_lookup', group: 'nettime', name: 'DNS 查询', desc: 'A/AAAA/MX/TXT',
    params: [{ key: 'rtype', label: '类型', kind: 'select', options: ['A', 'AAAA', 'MX', 'TXT'], default: 'A' }],
    needsMainInput: true,
  },

  // ---- fileconv ----
  {
    id: 'archive_list', group: 'fileconv', name: '归档列表', desc: '列出归档内文件(zip/tar/gz)',
    params: [{ key: 'path', label: '归档文件', kind: 'file' }],
    needsMainInput: false,
  },
  {
    id: 'archive_extract', group: 'fileconv', name: '解压归档', desc: '解压到源文件旁目录',
    params: [
      { key: 'path', label: '归档文件', kind: 'file' },
      { key: 'outputDir', label: '输出目录', kind: 'text', placeholder: '默认源文件旁' },
    ],
    needsMainInput: false,
  },
  {
    id: 'archive_compress', group: 'fileconv', name: '压缩文件', desc: '创建归档(zip/tar/gz)',
    params: [
      { key: 'paths', label: '文件', kind: 'file', multiple: true },
      { key: 'format', label: '格式', kind: 'select', options: ['zip', 'tar', 'targz', 'gz'], default: 'zip' },
    ],
    needsMainInput: false,
  },
  {
    id: 'archive_convert', group: 'fileconv', name: '归档转换', desc: '归档格式互转',
    params: [
      { key: 'path', label: '归档文件', kind: 'file' },
      { key: 'targetFormat', label: '目标格式', kind: 'select', options: ['zip', 'tar', 'targz', 'gz'], default: 'zip' },
    ],
    needsMainInput: false,
  },
  {
    id: 'image_convert', group: 'fileconv', name: '图像转换', desc: '图像格式互转(png/jpg/gif/bmp/webp/tiff/ico)',
    params: [
      { key: 'path', label: '图像文件', kind: 'file' },
      { key: 'target', label: '目标格式', kind: 'select', options: ['png', 'jpg', 'gif', 'bmp', 'webp', 'tiff', 'ico'], default: 'png' },
    ],
    needsMainInput: false,
  },
  {
    id: 'image_resize', group: 'fileconv', name: '图像缩放', desc: '缩放(一维 0 等比)',
    params: [
      { key: 'path', label: '图像文件', kind: 'file' },
      { key: 'width', label: '宽', kind: 'number', default: '0' },
      { key: 'height', label: '高', kind: 'number', default: '0' },
    ],
    needsMainInput: false,
  },
  {
    id: 'pdf_split', group: 'fileconv', name: 'PDF 拆分', desc: '每页一个 PDF',
    params: [
      { key: 'path', label: 'PDF 文件', kind: 'file' },
      { key: 'outputDir', label: '输出目录', kind: 'text', placeholder: '默认源文件旁' },
    ],
    needsMainInput: false,
  },
  {
    id: 'pdf_rotate', group: 'fileconv', name: 'PDF 旋转', desc: '所有页顺时针 90°',
    params: [{ key: 'path', label: 'PDF 文件', kind: 'file' }],
    needsMainInput: false,
  },
  {
    id: 'pdf_encrypt', group: 'fileconv', name: 'PDF 加密', desc: '口令加密(AES)',
    params: [
      { key: 'path', label: 'PDF 文件', kind: 'file' },
      { key: 'password', label: '口令', kind: 'password' },
    ],
    needsMainInput: false,
  },
  {
    id: 'pdf_decrypt', group: 'fileconv', name: 'PDF 解密', desc: '口令解密',
    params: [
      { key: 'path', label: 'PDF 文件', kind: 'file' },
      { key: 'password', label: '口令', kind: 'password' },
    ],
    needsMainInput: false,
  },
];
