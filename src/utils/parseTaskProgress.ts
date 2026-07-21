import type { OpenSpecTaskProgress } from "../types";

/**
 * 解析 tasks.md 內容計算進度，規則與 Rust 後端 parse_task_progress 保持一致。
 * 支援 markdown checkbox 格式：- [ ] / - [x] / * [ ] / + [ ] / 1. [ ] 等。
 * 用於前端樂觀更新，避免等待後端 watcher 掃描的延遲。
 */
export function parseTaskProgress(content: string): OpenSpecTaskProgress | null {
  let done = 0;
  let total = 0;

  for (const line of content.split("\n")) {
    const trimmed = line.replace(/^\s+/, "");

    // 解析 - [ ], * [ ], + [ ] 格式
    const bulletPrefix = ["- [", "* [", "+ ["].find((p) => trimmed.startsWith(p));
    if (bulletPrefix) {
      const rest = trimmed.slice(bulletPrefix.length);
      const marker = rest[0];
      if (marker !== undefined && rest[1] === "]") {
        total += 1;
        if (marker === "x" || marker === "X") {
          done += 1;
        }
      }
      continue;
    }

    // 解析 1. [ ], 2. [ ] 等數字格式
    const numberedMatch = /^\d+\.\s\[(.)\]/.exec(trimmed);
    if (numberedMatch) {
      total += 1;
      const marker = numberedMatch[1];
      if (marker === "x" || marker === "X") {
        done += 1;
      }
    }
  }

  if (total === 0) {
    return null;
  }

  const status = done === 0 ? "not_started" : done === total ? "done" : "in_progress";
  return { done, total, status };
}
