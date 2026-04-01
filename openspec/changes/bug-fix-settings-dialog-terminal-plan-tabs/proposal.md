## Why

五個已確認根因的錯誤影響核心使用體驗：設定頁面的路徑瀏覽器靜默失效、OpenCode 路徑顯示為空、終端機路徑驗證不足、終端機指令語法硬綁 PowerShell、Plan 編輯器佔用頂層 Tab 使導覽混亂。這些問題在當前版本中已全部重現，需一次性修正。

## What Changes

- **修正 dialog 權限**：在 `capabilities/default.json` 加入 `"dialog:default"`，讓「選擇資料夾」與「選擇檔案」按鈕恢復正常。
- **修正 OpenCode 路徑顯示空白**：`get_settings` 回傳前若 `opencode_root` 為空字串，自動補填 `default_opencode_root()` 的計算值。
- **強化終端機路徑驗證**：`validate_terminal_path_internal` 除檢查檔案存在外，還需確認 `file_stem()` 為 `pwsh`、`powershell`、`cmd`、`bash`、`sh` 之一。
- **終端機指令依型別分支**：`open_terminal_internal` 依偵測到的終端機類型（PowerShell、CMD、bash）帶入對應啟動參數，不再硬綁 `-NoExit -Command`。
- **複製指令依 Session Provider 分支**：`handleCopyCommand` 依 session provider（`copilot` / `opencode`）輸出對應的 resume 指令。
- **Plan Tab 移入 ProjectView**：Plan 編輯器 Tab 從頂層移至 ProjectView 的第二層 Tab，頂層只保留 Dashboard 與 Project Tab。

## Capabilities

### New Capabilities

（無新 capability）

### Modified Capabilities

- `app-settings`: 新增「選擇資料夾/檔案 dialog 權限」與「opencode_root 回傳補填」兩項行為要求；新增「終端機路徑必須包含有效可執行檔名稱」驗證規則。
- `terminal-launcher`: 新增「依終端機類型帶入對應啟動參數」與「複製指令依 provider 分支」兩項行為要求。
- `tabbed-ui`: 新增「Plan 編輯器 Tab 為 ProjectView 第二層 Tab，頂層不顯示 Plan Tab」的結構要求。
- `plan-viewer`: 新增「Plan 編輯器在 ProjectView 內作為子 Tab 開啟，不佔頂層 Tab」的導覽要求。

## Impact

- `src-tauri/capabilities/default.json` — 新增 `"dialog:default"`
- `src-tauri/src/lib.rs` — 修改 `get_settings_internal`、`validate_terminal_path_internal`、`open_terminal_internal`
- `src/App.tsx` — 修改 `handleCopyCommand`；將 plan 相關 state/handlers 往下傳給 ProjectView
- `src/components/ProjectView.tsx` — 新增 Plan sub-tab 支援，接收 plan props/callbacks
- `src/components/PlanEditor.tsx` — 從頂層 Tab 搬移至 ProjectView 子 Tab 內渲染
