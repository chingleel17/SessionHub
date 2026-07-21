# Design: fix-agents-config-ui-issues

## Context

Agents 設定頁（`AgentsConfigView.tsx`）目前的狀態管理與樣式有三個缺陷：

1. **頁籤切換狀態殘留**：`syncReport` 與 `selectedActionKeys` 在切換 AGENTS.md / Skills / Commands 頁籤時未清除。同步預覽報告區塊渲染於頁籤條件區塊之外（元件底部），因此在任何頁籤都會顯示上一個頁籤產生的報告；內容預覽選取狀態（`selectedNode`）雖已在切換時清除，但報告區塊仍殘留。
2. **對話框疊層錯誤**：`.dialog-backdrop`（App.css:1852）為 `position: fixed` 但未設定 `z-index`，而頁面內多處 sticky 元素有 `z-index: 11 / 50 / 100 / 300`，導致 `SyncConflictDialog`（於 `App.tsx` 根層渲染）被 sub-tab bar 與工具列蓋住。
3. **預覽僅限來源端**：矩陣的內容預覽固定使用 `SkillEntry.skillMdPath` / `CommandEntry.sourcePath`（來源端優先），當目標端（`.claude` / `.codex` / `.opencode` / `.github`）內容與來源不同（狀態為「內容不同」）時，使用者無法檢視目標端版本。

## Goals / Non-Goals

**Goals:**

- 各頁籤操作狀態（同步報告、內容預覽）互相隔離，切換即清除。
- 所有使用 `.dialog-backdrop` 的對話框永遠覆蓋於頁面內容（含 sticky 元素）之上。
- 矩陣中每個「目標檔案存在」的儲存格皆可點擊預覽該目標端內容，並在預覽標題標示目標名稱。

**Non-Goals:**

- 不變更同步引擎（Rust 端）行為與資料結構。
- 不提供來源/目標端 diff 對照檢視（僅單檔預覽）。
- 不調整 AGENTS.md 頁籤既有的樹狀檢視與編輯流程。

## Decisions

1. **頁籤切換時清除報告狀態**：於頁籤按鈕 `onClick`（已清除 `selectedNode` / `content` / `contentError` 處）一併呼叫 `setSyncReport(null)` 與 `setSelectedActionKeys([])`。
   - 替代方案：將報告區塊移入各頁籤條件區塊內、以 per-tab state 保存——複雜度高且無保留跨頁籤報告的需求，捨棄。
2. **`.dialog-backdrop` 加上 `z-index: 1000`**：低於 toast（9999）、高於頁面所有 sticky 疊層（最高 300）。修在共用 class 上，所有對話框一併受惠。
   - 替代方案：只對 `sync-conflict-dialog` 個別加 z-index——其他對話框存在同樣風險，不如修共用層。
3. **狀態 pill 改為可點擊按鈕以預覽目標端內容**：在 `renderMatrix` 目標儲存格中，當該目標狀態非 `target-missing` 時，pill 渲染為 `<button>`，點擊後組出目標端路徑並以既有 `loadContent`（走 `read_agents_file`）載入：
   - skills：`joinPath(target.targetRoot, entry.name, "SKILL.md")`
   - commands：`joinPath(target.targetRoot, entry.name + ".md")`
   - 預覽節點 label 設為 `${entry.name} (${targetId})`，與點擊項目名稱（來源端預覽）區隔。
   - 替代方案：後端擴充 scan 結果回傳各目標檔案路徑——前端以 `targetRoot + name` 即可組出，無需後端變更（commands 的巢狀名稱 `opsx/apply` 亦適用，`joinPath` 需正確處理分隔符）。
4. **目標不存在時 pill 維持非互動**：`target-missing` 狀態無檔案可預覽，維持 `<span>` 呈現，避免點擊無效路徑產生錯誤 toast。

## Risks / Trade-offs

- [目標端路徑由前端拼組，與後端實際掃描路徑可能不一致（如 copilot 的 `.github` / `.copilot` fallback）] → `TargetStatus.targetRoot` 由後端掃描時解析後回傳，前端一律使用該值拼組，不自行判斷 fallback。
- [`z-index: 1000` 未來若新增更高疊層元素仍可能被蓋] → 於 App.css 註解標明疊層約定（toast 9999 > dialog 1000 > sticky ≤ 300）。
- [pill 同時承載狀態顯示與預覽操作，可能誤點] → pill 保持原視覺樣式，僅加 hover 效果與 cursor:pointer 提示可點擊。
