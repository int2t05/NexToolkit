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

// ── 子分类(3 级树第 2 级:group > subgroup > tool)─────────
// 纯展示分组(不影响执行),前端维护。按功能域分,不按操作类型(方向用 ModeTabs 切)。

export interface SubCategory {
  id: string;
  parent: string; // group id
  label: { zh: string; en: string };
}

// 子分类定义(顺序驱动侧栏渲染)
export const SUBGROUPS: SubCategory[] = [
  // encode
  { id: 'base', parent: 'encode', label: { zh: 'Base 系列', en: 'Base' } },
  { id: 'url-html', parent: 'encode', label: { zh: 'URL/HTML', en: 'URL/HTML' } },
  { id: 'hex', parent: 'encode', label: { zh: 'Hex', en: 'Hex' } },
  { id: 'jwt', parent: 'encode', label: { zh: 'JWT', en: 'JWT' } },
  { id: 'punycode', parent: 'encode', label: { zh: 'Punycode', en: 'Punycode' } },
  { id: 'quoted-printable', parent: 'encode', label: { zh: 'QP', en: 'QP' } },
  { id: 'morse-braille', parent: 'encode', label: { zh: '摩斯/盲文', en: 'Morse/Braille' } },
  // convert
  { id: 'number-base', parent: 'convert', label: { zh: '进制', en: 'Number Base' } },
  { id: 'format-ir', parent: 'convert', label: { zh: '格式互转', en: 'Format IR' } },
  { id: 'table', parent: 'convert', label: { zh: '表格', en: 'Table' } },
  { id: 'md', parent: 'convert', label: { zh: 'Markdown', en: 'Markdown' } },
  { id: 'unit', parent: 'convert', label: { zh: '单位换算', en: 'Unit' } },
  // format
  { id: 'json-fmt', parent: 'format', label: { zh: 'JSON', en: 'JSON' } },
  { id: 'sql-xml', parent: 'format', label: { zh: 'SQL/XML/CSS', en: 'SQL/XML/CSS' } },
  // generate
  { id: 'hash-gen', parent: 'generate', label: { zh: '哈希/HMAC', en: 'Hash/HMAC' } },
  { id: 'random', parent: 'generate', label: { zh: '随机', en: 'Random' } },
  { id: 'qr', parent: 'generate', label: { zh: '二维码', en: 'QR' } },
  // text
  { id: 'stats', parent: 'text', label: { zh: '统计', en: 'Stats' } },
  { id: 'transform', parent: 'text', label: { zh: '变换', en: 'Transform' } },
  { id: 'trim-align', parent: 'text', label: { zh: '整理/转义', en: 'Trim/Escape' } },
  // crypto
  { id: 'hash', parent: 'crypto', label: { zh: '哈希', en: 'Hash' } },
  { id: 'hmac', parent: 'crypto', label: { zh: 'HMAC', en: 'HMAC' } },
  { id: 'bcrypt-scrypt', parent: 'crypto', label: { zh: 'Bcrypt/Scrypt', en: 'Bcrypt/Scrypt' } },
  { id: 'rsa', parent: 'crypto', label: { zh: 'RSA', en: 'RSA' } },
  { id: 'ed25519', parent: 'crypto', label: { zh: 'Ed25519', en: 'Ed25519' } },
  { id: 'chacha-aes', parent: 'crypto', label: { zh: '对称加密', en: 'Symmetric' } },
  { id: 'kdf', parent: 'crypto', label: { zh: 'KDF', en: 'KDF' } },
  { id: 'crc', parent: 'crypto', label: { zh: 'CRC', en: 'CRC' } },
  // nettime
  { id: 'network', parent: 'nettime', label: { zh: '网络', en: 'Network' } },
  { id: 'time', parent: 'nettime', label: { zh: '时间', en: 'Time' } },
  // fileconv
  { id: 'archive', parent: 'fileconv', label: { zh: '归档', en: 'Archive' } },
  { id: 'image-fmt', parent: 'fileconv', label: { zh: '图像格式', en: 'Image Format' } },
  { id: 'image-edit', parent: 'fileconv', label: { zh: '图像处理', en: 'Image Edit' } },
  { id: 'pdf-page', parent: 'fileconv', label: { zh: 'PDF 页面', en: 'PDF Pages' } },
  { id: 'pdf-sec', parent: 'fileconv', label: { zh: 'PDF 安全', en: 'PDF Security' } },
  { id: 'pdf-meta', parent: 'fileconv', label: { zh: 'PDF 元数据', en: 'PDF Metadata' } },
  { id: 'font', parent: 'fileconv', label: { zh: '字体', en: 'Font' } },
  { id: 'svg', parent: 'fileconv', label: { zh: 'SVG', en: 'SVG' } },
  { id: 'xlsx', parent: 'fileconv', label: { zh: '电子表格', en: 'XLSX' } },
  { id: 'extract', parent: 'fileconv', label: { zh: '文本提取', en: 'Extract' } },
  { id: 'engine-av', parent: 'fileconv', label: { zh: '音视频', en: 'Audio/Video' } },
  { id: 'engine-office', parent: 'fileconv', label: { zh: 'Office→PDF', en: 'Office→PDF' } },
  { id: 'engine-ebook', parent: 'fileconv', label: { zh: '电子书', en: 'Ebook' } },
  { id: 'engine-markup', parent: 'fileconv', label: { zh: '标记语言', en: 'Markup' } },
  { id: 'engine-ocr', parent: 'fileconv', label: { zh: 'OCR', en: 'OCR' } },
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
  jwt_decode: 'jwt', jwt_verify: 'jwt',
  punycode_encode: 'punycode', punycode_decode: 'punycode',
  quoted_printable_encode: 'quoted-printable', quoted_printable_decode: 'quoted-printable',
  morse_encode: 'morse-braille', morse_decode: 'morse-braille',
  braille_encode: 'morse-braille', braille_decode: 'morse-braille',
  zero_width_encode: 'morse-braille', zero_width_decode: 'morse-braille',
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
  // generate
  hash: 'hash-gen', hmac_compute: 'hash-gen',
  uuid_v4: 'random', uuid_v7: 'random', password_generate: 'random', lorem_ipsum: 'random',
  qr_svg: 'qr',
  // text
  text_stats: 'stats',
  case_convert: 'transform', sort_lines: 'transform', dedup_lines: 'transform',
  reverse_text: 'transform', regex_match: 'transform', regex_replace: 'transform',
  diff_text: 'transform',
  text_trim_blank: 'trim-align', tab_to_space: 'trim-align', space_to_tab: 'trim-align',
  text_align: 'trim-align', number_lines: 'trim-align', text_replace: 'trim-align',
  text_escape: 'trim-align',
  // crypto
  hmac_sha2: 'hmac',
  bcrypt_hash: 'bcrypt-scrypt', bcrypt_verify: 'bcrypt-scrypt',
  scrypt_hash: 'bcrypt-scrypt', scrypt_verify: 'bcrypt-scrypt',
  rsa_keygen: 'rsa', rsa_sign: 'rsa', rsa_verify: 'rsa', rsa_encrypt: 'rsa', rsa_decrypt: 'rsa',
  ed25519_keygen: 'ed25519', ed25519_sign: 'ed25519', ed25519_verify: 'ed25519',
  chacha20_encrypt: 'chacha-aes', chacha20_decrypt: 'chacha-aes',
  aes_gcm_encrypt: 'chacha-aes', aes_gcm_decrypt: 'chacha-aes',
  argon2: 'kdf', pbkdf2: 'kdf',
  crc32: 'crc', crc64: 'crc',
  // nettime
  ipcalc: 'network', dns_lookup: 'network', http_probe: 'network',
  timestamp_to_human: 'time', timestamp_from_human: 'time', cron_next: 'time',
  // fileconv
  archive_list: 'archive', archive_extract: 'archive', archive_compress: 'archive', archive_convert: 'archive',
  image_convert: 'image-fmt', image_resize: 'image-fmt',
  image_crop: 'image-edit', image_flip: 'image-edit', image_filter: 'image-edit',
  image_adjust: 'image-edit', image_compress_jpeg: 'image-edit',
  pdf_split: 'pdf-page', pdf_split_ranges: 'pdf-page', pdf_split_every_n: 'pdf-page',
  pdf_split_parity: 'pdf-page', pdf_rotate: 'pdf-page', pdf_merge: 'pdf-page',
  pdf_delete_pages: 'pdf-page', pdf_extract_pages: 'pdf-page',
  pdf_encrypt: 'pdf-sec', pdf_decrypt: 'pdf-sec', pdf_compress: 'pdf-sec',
  pdf_set_metadata: 'pdf-meta', pdf_add_page_numbers: 'pdf-meta',
  font_convert: 'font', font_meta: 'font',
  svg_convert: 'svg',
  xlsx_to_json: 'xlsx', json_to_xlsx: 'xlsx',
  pdf_to_text: 'extract', docx_to_text: 'extract',
  av_convert: 'engine-av', office_to_pdf: 'engine-office', ebook_convert: 'engine-ebook',
  markup_convert: 'engine-markup', ocr: 'engine-ocr',
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
  punycode_encode: 'punycode_decode',
  quoted_printable_encode: 'quoted_printable_decode',
  morse_encode: 'morse_decode',
  braille_encode: 'braille_decode',
  zero_width_encode: 'zero_width_decode',
};

// 多结果并出的 subgroup(单输入 → 循环该 subgroup 所有工具调 run_tool,每算法一行结果)
export const MULTI_OUTPUT_SUBGROUPS = new Set(['hash-gen']);

// 引擎状态(运行时探测,对齐 Rust EngineStatusDto)
export interface EngineStatusDto {
  binary: string;
  desc: string;
  available: boolean;
}
