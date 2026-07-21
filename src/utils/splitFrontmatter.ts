// 將 Markdown 內容中的 YAML frontmatter 區塊與內文分離的工具函式。
// 用途：avoid `marked` 將 frontmatter（以 `---` 包夾的區塊）誤判為 setext 標題，
// 導致預覽畫面將整個 frontmatter 擠壓成一行粗體文字。

export interface FrontmatterSplitResult {
  /** frontmatter 的原始內容（不含前後的 `---` 標記），若未偵測到則為 null */
  frontmatter: string | null;
  /** 扣除 frontmatter 後剩餘的 Markdown 內文 */
  body: string;
}

const FRONTMATTER_DELIMITER = /^---\r?\n/;

/**
 * 偵測並拆分字串開頭的 YAML frontmatter 區塊。
 *
 * 判斷條件：第一行必須「恰好」是 `---`，且在其後能找到對應的結尾 `---` 行。
 * 若條件不成立（例如第一行不是 `---`，或找不到結尾標記），視為沒有 frontmatter，
 * 保守地將整份內容原封不動當作 body 回傳，確保既有不含 frontmatter 的
 * Markdown（例如 plans/specs 文件）完全不受影響。
 */
export function splitFrontmatter(content: string): FrontmatterSplitResult {
  const openMatch = content.match(FRONTMATTER_DELIMITER);
  if (!openMatch) {
    return { frontmatter: null, body: content };
  }

  const afterOpen = content.slice(openMatch[0].length);
  const closeMatch = afterOpen.match(/\r?\n---[ \t]*(\r?\n|$)/);
  if (!closeMatch || closeMatch.index === undefined) {
    return { frontmatter: null, body: content };
  }

  const frontmatter = afterOpen.slice(0, closeMatch.index);
  let body = afterOpen.slice(closeMatch.index + closeMatch[0].length);
  // 移除 frontmatter 區塊後緊接的第一個空白行，避免內文開頭多出空行。
  body = body.replace(/^\r?\n/, "");

  return { frontmatter, body };
}

/**
 * 將內容中偵測到的 frontmatter 重新包裝成 fenced yaml 區塊，
 * 提供給 `marked` 解析前使用，避免其被誤判為 setext 標題。
 * 若未偵測到 frontmatter，則原樣返回輸入內容。
 */
export function prepareMarkdownForPreview(content: string): string {
  const { frontmatter, body } = splitFrontmatter(content);
  if (frontmatter === null) {
    return content;
  }
  return "```yaml\n" + frontmatter.trimEnd() + "\n```\n\n" + body;
}
