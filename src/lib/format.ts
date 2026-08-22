// 纯格式化函数(无状态,从 App.svelte 抽出)

/** HTML 转义:输出区与 SVG 直渲染前的安全处理 */
export function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** 取路径末段(文件名),跨平台斜杠/反斜杠 */
export function basename(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

/** 文件选择展示:多个文件名逗号分隔 */
export function joinFileNames(paths: string[]): string {
  return paths.map(basename).join(', ');
}
