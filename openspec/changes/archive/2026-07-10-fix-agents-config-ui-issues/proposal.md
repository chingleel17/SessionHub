# Proposal: fix-agents-config-ui-issues

## Why

Agents 設定頁（AgentsConfigView）在實際操作中暴露三個 UI 問題：頁籤切換後同步預覽報告與內容預覽殘留、同步衝突對話框被頁面工具列蓋住、內容預覽只能檢視來源端版本。這些問題讓同步操作流程混亂且無法比對目標端差異，需一併修正。

## What Changes

- 切換 AGENTS.md / Skills / Commands 頁籤時，清除同步預覽報告（syncReport）與內容預覽面板的殘留狀態，各頁籤的操作狀態互不干擾。
- 修正 SyncConflictDialog 的疊層順序：`dialog-backdrop` 需以固定定位覆蓋全畫面，且 z-index 高於 sub-tab bar、sticky 工具列等既有堆疊元素，避免對話框被蓋住。
- Skills / Commands 矩陣中各目標（claude / codex / opencode / copilot）的狀態 pill 在目標檔案存在時可點擊，點擊後預覽該目標端的檔案內容（skill 為 `<targetRoot>/<name>/SKILL.md`，command 為 `<targetRoot>/<name>.md`），預覽標題需標示目標名稱以資區別。
- 內容檢視改為整頁詳情模式：點擊項目名稱或目標狀態進入全幅詳情頁（含返回控制項），取代原本在列表下方展開需捲動的預覽面板；返回後保留列表捲動與勾選狀態。
- 修正矩陣欄位對齊：各平台欄寬一致、欄標題勾選框與狀態格垂直置中對齊；狀態呈現改為精簡圖示 + 文字、柔和配色，取代過重的全大寫粗體膠囊。
- 修正詳情頁「外部開啟」按鈕失效：Tauri opener 外掛對 `open-path` 指令套用路徑 scope 限制，原 `capabilities/default.json` 僅宣告 `opener:allow-open-path` 而未帶 scope，導致 `openPath` 對任何路徑皆回傳 `Not allowed to open path`。改以帶 `allow` scope 的物件宣告該權限，允許應用實際會開啟的檔案路徑。
- 修正 Commands 掃描的檔名比對錯位：GitHub Copilot 的 `.github/prompts/` 慣例要求副檔名為 `.prompt.md`，先前掃描僅剝離 `.md`，導致 copilot 端的 `<name>.prompt.md` 被誤判為獨立於 `<name>` 的 command，使該項目在 claude/codex/opencode 全數顯示「錯誤」。掃描與同步（含目標端預覽路徑組裝）現依 target 分流副檔名：copilot 用 `.prompt.md`，其餘沿用 `.md`。

## Capabilities

### New Capabilities

- `agents-config-view-ux`: Agents 設定頁的檢視互動行為——頁籤切換時的狀態清理、同步衝突對話框的疊層規範、以及來源端與各目標端內容的預覽能力。

### Modified Capabilities

<!-- agents-skills-sync / agents-commands-sync 規格仍在 add-agents-config-maintenance change 中尚未歸檔，本次僅涉及檢視層互動，不變更其同步需求。 -->

## Impact

- `src/components/AgentsConfigView.tsx`：頁籤切換狀態清理、狀態 pill 可點擊預覽目標端內容。
- `src/components/SyncConflictDialog.tsx` / `src/App.css`：對話框疊層（z-index / position）修正。
- `src/locales/zh-TW.ts`、`src/locales/en-US.ts`：目標端預覽標示所需字串。
- `src-tauri/capabilities/default.json`：`opener:allow-open-path` 改為帶 `allow` scope 的物件宣告，修正外部開啟被權限拒絕。
- 無 Rust 程式碼變更；`read_agents_file` 既有命令即可讀取目標端檔案（僅新增 Tauri capability 權限 scope 設定）。
