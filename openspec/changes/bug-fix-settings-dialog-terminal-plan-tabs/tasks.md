## 1. Rust 後端修正

- [x] 1.1 在 `src-tauri/capabilities/default.json` 加入 `"dialog:default"` 權限
- [x] 1.2 在 `get_settings_internal` 回傳前補填空白的 `opencode_root`（呼叫 `default_opencode_root()`）
- [x] 1.3 在 `validate_terminal_path_internal` 加入 `file_stem()` 白名單驗證（`pwsh`、`powershell`、`cmd`、`bash`、`sh`，不區分大小寫）
- [x] 1.4 在 `open_terminal_internal` 依 `file_stem()` 分支帶入終端機啟動參數（pwsh/powershell 用 `-NoExit -Command`；cmd 用 `/K`；bash/sh 用 `-i` + `current_dir`）

## 2. 前端邏輯修正

- [x] 2.1 修改 `handleCopyCommand`（`App.tsx`）依 `session.provider` 輸出對應 resume 指令（`copilot --resume=<id>` 或 `opencode session <id>`）
- [x] 2.2 確認 `SessionInfo` TypeScript 型別（`src/types/index.ts`）含有 `provider` 欄位

## 3. Plan Tab 移入 ProjectView

- [x] 3.1 在 `App.tsx` 中移除 Plan 頂層 Tab 邏輯（`openPlanKeys` 不再推送至頂層 tabs），改以 per-project plan key map 傳遞至 ProjectView
- [x] 3.2 更新 `ProjectView` props 介面，新增 plan 相關 state 與 handlers（`openPlanKeys`、`planDraft`、`planPreviewHtml`、`onOpenPlan`、`onSavePlan`、`onOpenPlanExternal`、`onReadFileContent`）
- [x] 3.3 在 `ProjectView` 新增第二層子分頁列，固定包含「Sessions」與「Plans & Specs」子分頁
- [x] 3.4 在 `ProjectView` 實作動態 `Plan:<sessionId>` 子分頁（開啟、切換、關閉），渲染 `PlanEditor` 元件
- [x] 3.5 在 `PlansSpecsView` 渲染中確認「Plans & Specs」子分頁可正常從 ProjectView 觸發

## 4. 驗證

- [x] 4.1 執行 `cd src-tauri && cargo test`，確認所有 Rust 單元測試通過（8/8 passed）
- [x] 4.2 執行 `bun run build`，確認 TypeScript 型別檢查無錯誤
- [ ] 4.3 手動驗證：設定頁面「選擇資料夾」與「選擇檔案」可正常開啟 dialog
- [ ] 4.4 手動驗證：設定頁面 OpenCode 路徑欄位顯示預設值（非空白）
- [ ] 4.5 手動驗證：輸入非法終端機路徑（如 `notepad.exe`）時儲存被拒
- [ ] 4.6 手動驗證：以 cmd.exe 開啟終端機，確認工作目錄正確切換
- [ ] 4.7 手動驗證：頂層分頁不再出現 Plan 分頁；Plan 編輯器在 ProjectView 子分頁內開啟
