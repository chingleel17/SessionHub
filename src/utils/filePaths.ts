import type { TreeNode } from "../types";

function trimTrailingSeparators(path: string): string {
  return path.replace(/[\\/]+$/, "");
}

/** 依根目錄既有的分隔符組合路徑，避免寫死 Windows 的 `\`。 */
export function joinPath(root: string, relativePath: string): string {
  const separator = root.includes("\\") ? "\\" : "/";
  const normalizedRelative = relativePath.replace(/[\\/]+/g, separator).replace(/^[\\/]+/, "");
  return `${trimTrailingSeparators(root)}${separator}${normalizedRelative}`;
}

/** 取得相對於根目錄的路徑（以 `/` 分隔）；不在根目錄底下時回傳 null。 */
export function toRelativePath(root: string, absolutePath: string): string | null {
  const normalizedRoot = trimTrailingSeparators(root.replace(/\\/g, "/"));
  const normalizedPath = absolutePath.replace(/\\/g, "/");
  if (!normalizedRoot || !normalizedPath.toLowerCase().startsWith(`${normalizedRoot.toLowerCase()}/`)) {
    return null;
  }
  return normalizedPath.slice(normalizedRoot.length + 1);
}

/** 將樹節點的檔案路徑解析為絕對路徑與專案相對路徑。 */
export function resolveNodeFilePaths(
  projectCwd: string,
  filePath: string,
  filePathType: TreeNode["filePathType"] | null,
): { absolutePath: string; relativePath: string | null } {
  const absolutePath = filePathType === "openspec"
    ? joinPath(projectCwd, `openspec/${filePath}`)
    : filePath;
  return { absolutePath, relativePath: toRelativePath(projectCwd, absolutePath) };
}
