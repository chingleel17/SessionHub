# Tasks: fix-agents-config-ui-issues

## 1. 頁籤切換狀態清理

- [x] 1.1 在 `AgentsConfigView.tsx` 頁籤按鈕 `onClick` 中，於既有的 `setSelectedNode(null)` / `setContent(null)` / `setContentError(null)` 之外，加入 `setSyncReport(null)` 與 `setSelectedActionKeys([])`
- [x] 1.2 驗證：Skills 產生同步報告後切換至 AGENTS.md / Commands，報告區塊與預覽面板皆不殘留

## 2. 對話框疊層修正

- [x] 2.1 在 `App.css` 的 `.dialog-backdrop` 加上 `z-index: 1000`，並以註解標明疊層約定（toast 9999 > dialog 1000 > sticky ≤ 300）
- [x] 2.2 驗證：套用同步觸發 SyncConflictDialog 時，對話框完整覆蓋 sub-tab bar 與工具列；其他共用對話框（編輯備註/標籤等）不受影響

## 3. 目標端內容預覽

- [x] 3.1 在 `AgentsConfigView.tsx` `renderMatrix` 的目標儲存格中，當狀態非 `target-missing` 時將狀態 pill 渲染為可點擊按鈕，點擊組出目標端路徑（skills：`joinPath(joinPath(targetRoot, name), "SKILL.md")`；commands：`joinPath(targetRoot, name + ".md")`，巢狀名稱需正確處理分隔符）並呼叫 `loadContent`
- [x] 3.2 預覽節點 label 設為 `${entry.name} (${targetId})`，與來源端預覽（點擊項目名稱）區隔
- [x] 3.3 pill 可點擊狀態加上 hover 效果與 `cursor: pointer`（App.css，含 dark.css 若需要）
- [x] 3.4 驗證：狀態為「內容不同」的目標可點擊預覽該端內容；「缺少目標」不可點擊；點擊項目名稱仍預覽來源端

## 4. 整頁詳情模式

- [x] 4.1 在 `AgentsConfigView.tsx` 以既有 `previewNode`（`agents-preview:` 前綴）為條件，改為整頁切換：`previewNode` 存在時 `renderMatrix` 僅渲染詳情頁（返回鈕 + 工具列 + ContentViewer），隱藏工具列、矩陣表格與同步報告
- [x] 4.2 新增返回控制項（清除 `selectedNode`／`content`／`contentError`），回到列表；勾選狀態（`selectedMatrixNames`）與 `syncReport` 不清除以保留現場
- [x] 4.3 詳情頁樣式：`.agents-detail-view` 全幅、頂部返回列 sticky，內容區起始於視窗頂部

## 5. 矩陣對齊與精簡狀態

- [x] 5.1 `.agents-matrix-table` 改用 `table-layout: fixed`，第一欄固定寬（30%）、其餘平台欄由瀏覽器等分
- [x] 5.2 欄標題 `.agents-target-toggle` 與狀態格內容皆水平置中（平台欄 `text-align: center`、flex `justify-content: center`）
- [x] 5.3 重繪 `.agents-status-pill`：狀態點（`::before`）+ 文字、去除全大寫與粗邊框、柔和底色；保留 done/not_started/in_progress/neutral tone 對映與可點擊 hover；移除格內重複符號
- [x] 5.4 dark.css 對應調整：`.agents-matrix-preview` → `.agents-detail-view`，新增 `.agents-detail-header` 深色背景

## 6. 品質檢查

- [x] 6.1 `tsc --noEmit` 型別檢查通過
- [x] 6.2 手動走查：整頁詳情進出、返回保留勾選、矩陣欄位對齊、狀態呈現於專案級與全域 scope

## 7. Commands 掃描：Copilot prompt 副檔名比對錯位

- [x] 7.1 `src-tauri/src/agents_config.rs`：新增 `command_file_suffix` / `strip_command_file_suffix` / `command_relative_path`，copilot target 一律用 `.prompt.md`，其餘沿用 `.md`；`scan_agents_commands_internal` 掃描與 target 路徑組裝改用上述 helper
- [x] 7.2 新增 Rust 測試 `project_commands_scan_matches_copilot_prompt_suffix_to_shared_name`，驗證 copilot 端 `<name>.prompt.md` 正確歸併至同一 command 且狀態為 in-sync，不產生 `<name>.prompt` 獨立列
- [x] 7.3 `src/components/AgentsConfigView.tsx`：新增 `commandFileName` helper，`buildMatrixSyncRequest`（套用同步）與狀態 pill 目標端預覽路徑組裝皆改用此 helper
- [x] 7.4 `cargo test --lib agents_config`（15 個測試通過）與 `tsc --noEmit` 驗證無回歸

## 8. 外部開啟 opener 路徑 scope

- [x] 8.1 `src-tauri/capabilities/default.json`：`opener:allow-open-path` 由字串改為帶 `allow` scope 的物件（`{ "path": "**" }`、`{ "path": "**/*" }`），修正 `openPath` 對掃描路徑回傳 `Not allowed to open path`
- [x] 8.2 驗證：重啟 `tauri dev` 後，Skills 詳情頁「外部開啟」可成功開啟 `.agents/skills/<name>/SKILL.md`，不再跳出權限錯誤
