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

// 工具分组(顺序驱动侧栏渲染,按用户使用频率 + 心智模型排序)
export const GROUPS: string[] = [
  'encode',
  'text',
  'convert',
  'format',
  'crypto',
  'generate',
  'network',
  'time',
  'fileconv',
];

export const GROUP_LABEL: Record<string, { zh: string; en: string }> = {
  encode: { zh: '编解码', en: 'Encoders' },
  text: { zh: '文本处理', en: 'Text' },
  convert: { zh: '数据转换', en: 'Data' },
  format: { zh: '格式化', en: 'Formatters' },
  crypto: { zh: '加密与哈希', en: 'Crypto' },
  generate: { zh: '生成器', en: 'Generators' },
  network: { zh: '网络', en: 'Network' },
  time: { zh: '时间', en: 'Time' },
  fileconv: { zh: '文件转换', en: 'Files' },
};

// 分类色 CSS 变量名(对应 tokens.css --ntx-cat-*)
export const GROUP_CAT_VAR: Record<string, string> = {
  encode: '--ntx-cat-encode',
  text: '--ntx-cat-text',
  convert: '--ntx-cat-convert',
  format: '--ntx-cat-format',
  crypto: '--ntx-cat-crypto',
  generate: '--ntx-cat-generate',
  network: '--ntx-cat-nettime',
  time: '--ntx-cat-nettime',
  fileconv: '--ntx-cat-fileconv',
};

// 分类图标(lucide 组件名,Sidebar 渲染)
export const GROUP_ICON: Record<string, string> = {
  encode: 'Braces',
  text: 'Type',
  convert: 'ArrowRightLeft',
  format: 'AlignLeft',
  crypto: 'Shield',
  generate: 'Sparkles',
  network: 'Globe',
  time: 'Clock',
  fileconv: 'FileBox',
};

// ── 子分类(3 级树第 2 级:group > subgroup > tool)─────────
// 纯展示分组(不影响执行),前端维护。按功能域分,不按操作类型(方向用 ModeTabs 切)。

export interface SubCategory {
  id: string;
  parent: string; // group id
  label: { zh: string; en: string };
}

// 子分类定义(顺序驱动侧栏渲染)
export const SUBGROUPS: SubCategory[] = [
  // encode(7)
  { id: 'base', parent: 'encode', label: { zh: 'Base 系列', en: 'Base' } },
  { id: 'url-html', parent: 'encode', label: { zh: 'URL/HTML', en: 'URL/HTML' } },
  { id: 'hex', parent: 'encode', label: { zh: 'Hex', en: 'Hex' } },
  { id: 'charset', parent: 'encode', label: { zh: '字符编码', en: 'Charset' } },
  { id: 'jwt', parent: 'encode', label: { zh: 'JWT', en: 'JWT' } },
  { id: 'punycode-qp', parent: 'encode', label: { zh: 'Punycode/QP', en: 'Punycode/QP' } },
  { id: 'morse-braille', parent: 'encode', label: { zh: '摩斯/盲文', en: 'Morse/Braille' } },
  // text(3)
  { id: 'stats', parent: 'text', label: { zh: '统计', en: 'Stats' } },
  { id: 'transform', parent: 'text', label: { zh: '变换', en: 'Transform' } },
  { id: 'trim-align', parent: 'text', label: { zh: '整理/转义', en: 'Trim/Escape' } },
  // convert(5)
  { id: 'number-base', parent: 'convert', label: { zh: '进制', en: 'Number Base' } },
  { id: 'format-ir', parent: 'convert', label: { zh: '格式互转', en: 'Format IR' } },
  { id: 'table', parent: 'convert', label: { zh: '表格', en: 'Table' } },
  { id: 'md', parent: 'convert', label: { zh: 'Markdown', en: 'Markdown' } },
  { id: 'unit', parent: 'convert', label: { zh: '单位换算', en: 'Unit' } },
  // format(2)
  { id: 'json-fmt', parent: 'format', label: { zh: 'JSON', en: 'JSON' } },
  { id: 'sql-xml', parent: 'format', label: { zh: 'SQL/XML/CSS', en: 'SQL/XML/CSS' } },
  // crypto(5)
  { id: 'hash-hmac', parent: 'crypto', label: { zh: '哈希/HMAC', en: 'Hash/HMAC' } },
  { id: 'password-hash', parent: 'crypto', label: { zh: '口令哈希', en: 'Password Hash' } },
  { id: 'asymmetric', parent: 'crypto', label: { zh: '非对称加密', en: 'Asymmetric' } },
  { id: 'symmetric', parent: 'crypto', label: { zh: '对称加密', en: 'Symmetric' } },
  { id: 'kdf', parent: 'crypto', label: { zh: '密钥派生', en: 'KDF' } },
  // generate(3)
  { id: 'random', parent: 'generate', label: { zh: '随机', en: 'Random' } },
  { id: 'qr', parent: 'generate', label: { zh: '二维码', en: 'QR' } },
  // network(3)
  { id: 'ipcalc', parent: 'network', label: { zh: 'IP 计算', en: 'IP Calc' } },
  { id: 'dns', parent: 'network', label: { zh: 'DNS', en: 'DNS' } },
  { id: 'http', parent: 'network', label: { zh: 'HTTP', en: 'HTTP' } },
  // time(2)
  { id: 'timestamp', parent: 'time', label: { zh: '时间戳', en: 'Timestamp' } },
  { id: 'cron', parent: 'time', label: { zh: 'Cron', en: 'Cron' } },
  // fileconv(7)
  { id: 'convert-any', parent: 'fileconv', label: { zh: '通用转换', en: 'Convert' } },
  { id: 'image', parent: 'fileconv', label: { zh: '图像', en: 'Image' } },
  { id: 'pdf', parent: 'fileconv', label: { zh: 'PDF', en: 'PDF' } },
  { id: 'extract', parent: 'fileconv', label: { zh: '文本提取', en: 'Extract' } },
  { id: 'engine-av', parent: 'fileconv', label: { zh: '音视频', en: 'Audio/Video' } },
  { id: 'engine-ocr', parent: 'fileconv', label: { zh: 'OCR', en: 'OCR' } },
  { id: 'archive-font', parent: 'fileconv', label: { zh: '归档/字体/SVG', en: 'Archive/Font/SVG' } },
];

// tool id → subgroup id 映射(未列出的归入父 group 的"其他"兜底)
export const SUBCATEGORY: Record<string, string> = {
  // encode
  base64_encode: 'base', base64_decode: 'base',
  base32_encode: 'base', base32_decode: 'base',
  base58_encode: 'base', base58_decode: 'base',
  base85_encode: 'base', base85_decode: 'base',
  url_encode: 'url-html', url_decode: 'url-html',
  html_encode: 'url-html', html_decode: 'url-html',
  hex_encode: 'hex', hex_decode: 'hex',
  charset_encode: 'charset', charset_decode: 'charset',
  jwt_decode: 'jwt', jwt_verify: 'jwt',
  punycode_encode: 'punycode-qp', punycode_decode: 'punycode-qp',
  quoted_printable_encode: 'punycode-qp', quoted_printable_decode: 'punycode-qp',
  morse_encode: 'morse-braille', morse_decode: 'morse-braille',
  braille_encode: 'morse-braille', braille_decode: 'morse-braille',
  zero_width_encode: 'morse-braille', zero_width_decode: 'morse-braille',
  // text
  text_stats: 'stats',
  case_convert: 'transform', sort_lines: 'transform', dedup_lines: 'transform',
  reverse_text: 'transform', regex_match: 'transform', regex_replace: 'transform',
  diff_text: 'transform',
  text_trim_blank: 'trim-align', tab_to_space: 'trim-align', space_to_tab: 'trim-align',
  text_align: 'trim-align', number_lines: 'trim-align', text_replace: 'trim-align',
  text_escape: 'trim-align',
  // convert
  numbase_convert: 'number-base',
  json_to_yaml: 'format-ir', json_to_toml: 'format-ir', json_to_xml: 'format-ir',
  yaml_to_json: 'format-ir', yaml_to_toml: 'format-ir', yaml_to_xml: 'format-ir',
  toml_to_json: 'format-ir', toml_to_yaml: 'format-ir', toml_to_xml: 'format-ir',
  xml_to_json: 'format-ir', xml_to_yaml: 'format-ir', xml_to_toml: 'format-ir',
  csv_to_json: 'table', csv_to_tsv: 'table', csv_to_yaml: 'table', csv_to_xml: 'table',
  json_to_csv: 'table', json_to_tsv: 'table',
  tsv_to_csv: 'table', tsv_to_json: 'table',
  xml_to_csv: 'table', yaml_to_csv: 'table',
  md_to_html: 'md', md_to_txt: 'md',
  unit_convert: 'unit',
  // format
  json_format: 'json-fmt', json_minify: 'json-fmt',
  sql_format: 'sql-xml', xml_format: 'sql-xml', xml_minify: 'sql-xml', css_minify: 'sql-xml',
  // crypto(哈希归此,含从 generate 移来的 hash/hmac_compute)
  hash: 'hash-hmac', hmac_compute: 'hash-hmac', hmac_sha2: 'hash-hmac',
  crc32: 'hash-hmac', crc64: 'hash-hmac',
  bcrypt_hash: 'password-hash', bcrypt_verify: 'password-hash',
  scrypt_hash: 'password-hash', scrypt_verify: 'password-hash',
  rsa_keygen: 'asymmetric', rsa_sign: 'asymmetric', rsa_verify: 'asymmetric',
  rsa_encrypt: 'asymmetric', rsa_decrypt: 'asymmetric',
  ed25519_keygen: 'asymmetric', ed25519_sign: 'asymmetric', ed25519_verify: 'asymmetric',
  chacha20_encrypt: 'symmetric', chacha20_decrypt: 'symmetric',
  aes_gcm_encrypt: 'symmetric', aes_gcm_decrypt: 'symmetric',
  argon2: 'kdf', pbkdf2: 'kdf',
  // generate
  uuid_v4: 'random', uuid_v7: 'random', password_generate: 'random', lorem_ipsum: 'random',
  qr_svg: 'qr',
  // network
  ipcalc: 'ipcalc', dns_lookup: 'dns', http_probe: 'http',
  // time
  timestamp_to_human: 'timestamp', timestamp_from_human: 'timestamp', cron_next: 'cron',
  // fileconv
  convert_file: 'convert-any',
  image_convert: 'image', image_resize: 'image',
  image_crop: 'image', image_flip: 'image', image_filter: 'image',
  image_adjust: 'image', image_compress_jpeg: 'image',
  pdf_split: 'pdf', pdf_split_ranges: 'pdf', pdf_split_every_n: 'pdf',
  pdf_split_parity: 'pdf', pdf_rotate: 'pdf', pdf_merge: 'pdf',
  pdf_delete_pages: 'pdf', pdf_extract_pages: 'pdf',
  pdf_encrypt: 'pdf', pdf_decrypt: 'pdf',
  pdf_set_metadata: 'pdf', pdf_add_page_numbers: 'pdf',
  docx_to_text: 'extract',
  av_convert: 'engine-av', ocr: 'engine-ocr',
  archive_list: 'archive-font', archive_extract: 'archive-font',
  archive_compress: 'archive-font', archive_convert: 'archive-font',
  font_convert: 'archive-font', font_meta: 'archive-font',
  svg_convert: 'archive-font',
  xlsx_to_json: 'archive-font', json_to_xlsx: 'archive-font',
};

// 双向工具(encode/decode 对,显 ModeTabs + Swap)。key 是 _encode id,value 是对应 _decode id
export const BIDIRECTIONAL: Record<string, string> = {
  base64_encode: 'base64_decode',
  base32_encode: 'base32_decode',
  base58_encode: 'base58_decode',
  base85_encode: 'base85_decode',
  url_encode: 'url_decode',
  html_encode: 'html_decode',
  hex_encode: 'hex_decode',
  charset_encode: 'charset_decode',
  punycode_encode: 'punycode_decode',
  quoted_printable_encode: 'quoted_printable_decode',
  morse_encode: 'morse_decode',
  braille_encode: 'braille_decode',
  zero_width_encode: 'zero_width_decode',
  chacha20_encrypt: 'chacha20_decrypt',
  aes_gcm_encrypt: 'aes_gcm_decrypt',
};

// 多结果并出的 subgroup(单输入 → 循环该 subgroup 所有工具调 run_tool,每算法一行结果)
export const MULTI_OUTPUT_SUBGROUPS = new Set(['hash-hmac']);

// 引擎状态(运行时探测,对齐 Rust EngineStatusDto)
export interface EngineStatusDto {
  binary: string;
  desc: string;
  available: boolean;
  resolved_path: string | null;
}

// 引擎安装信息(对齐 Rust EngineInstallInfoDto)
export interface EngineInstallInfoDto {
  binary: string;
  desc: string;
  available: boolean;
  is_portable: boolean;
  download_url: string;
  install_path: string | null;
}
