## Why

SessionHub 目前僅支援 GitHub Copilot CLI 的 session 資料來源（YAML 檔案），且專案頁面只有 session 列表。但使用者同時使用多個 AI 開發工具（OpenCode、oh-my-opencode 的 Sisyphus plan agent），這些工具各自有獨立的紀錄散落在不同位置。此外，專案中可能存在結構化的開發計畫（`.sisyphus/plans/`、`openspec/changes/`），目前無法在 SessionHub 中統一檢視。

需要：
1. 支援多平台 session 整合（Copilot + OpenCode），讓同一專案的所有 AI 使用紀錄集中顯示
2. 將 `.sisyphus`（plans、notepads、evidence）與 `openspec`（changes、specs）納入專案資訊管理
3. 重構專案頁面為分頁架構（Sessions / Plans & Specs），讓不同類型的資訊有獨立且清晰的檢視空間

## What Changes

### 多平台 Session 支援
- 新增 OpenCode session 資料來源：從 `~/.local/share/opencode/opencode.db`（SQLite）讀取 session 與 project 資料
- 擴展 `AppSettings`：新增 `opencodeRoot` 路徑設定（預設 `~/.local/share/opencode/`）與 `enabledProviders` 啟用的平台篩選
- 擴展 `SessionInfo`：新增 `provider` 欄位標識來源平台（`"copilot"` | `"opencode"`）
- 統一 session 資料模型：將 OpenCode 的 session 欄位映射至 SessionInfo 共用結構
- 前端 session card 新增平台標籤（provider tag）顯示來源
- 前端新增平台篩選 UI（checkbox filter），預設全部勾選
- 合併不同平台 session 至同一專案分組（以 cwd/worktree 路徑匹配）

### 專案頁面分頁架構
- **專案頁面重構**：將現有 `ProjectView` 改為分頁式架構，包含多個子分頁
  - **Sessions 分頁**：現有 session 列表（含搜尋、篩選、排序），加上 provider tag 與 provider filter
  - **Plans & Specs 分頁**：顯示該專案的 `.sisyphus`（plans、notepads、evidence）與 `openspec`（changes、specs）資料

### .sisyphus 資料整合
- 掃描各專案目錄下的 `.sisyphus/` 目錄
- 讀取 `boulder.json`（active plan、session 關聯、agent 資訊）
- 讀取 `plans/*.md`（plan 清單與內容摘要）
- 讀取 `notepads/*/`（notepad 清單，含 issues.md、learnings.md）
- 讀取 `evidence/*.txt`（task 執行證據清單）
- 讀取 `drafts/*.md`（草稿清單）

### openspec 資料整合
- 掃描各專案目錄下的 `openspec/` 目錄
- 讀取 `config.yaml`（schema 設定）
- 讀取 `changes/`（進行中的 change 清單與狀態）
- 讀取 `changes/archive/`（已完成的 change 清單）
- 讀取 `specs/`（規格清單）

### Dashboard 擴展
- 統計數據納入多平台 session
- 新增各平台 session 數量分佈

## Capabilities

### New Capabilities
- `opencode-provider`: OpenCode session 資料來源的讀取、解析與資料映射邏輯
- `provider-filter`: 多平台篩選功能，讓使用者選擇要顯示哪些平台的 session
- `provider-tag`: Session card 上的平台來源標籤顯示
- `sisyphus-reader`: 讀取與解析專案目錄下 `.sisyphus/` 的 plans、notepads、evidence、drafts 資料
- `openspec-reader`: 讀取與解析專案目錄下 `openspec/` 的 changes、specs 資料
- `project-subtabs`: 專案頁面分頁架構，將 Sessions 與 Plans & Specs 以子分頁形式組織

### Modified Capabilities
- `app-settings`: 新增 `opencodeRoot` 與 `enabledProviders` 設定欄位
- `session-list`: 掃描邏輯擴展為多資料來源合併，SessionInfo 新增 `provider` 欄位
- `session-grouping`: 專案分組需跨平台合併（同 cwd 的 Copilot 與 OpenCode session 歸入同一組）
- `dashboard`: 統計資訊需涵蓋多平台數據
- `file-watcher`: 需監聽 OpenCode 資料庫變更
- `tabbed-ui`: 專案分頁內新增子分頁（sub-tabs）機制

## Impact

- **Rust 後端（lib.rs）**：新增 OpenCode SQLite 讀取邏輯、新增 `.sisyphus` 與 `openspec` 目錄掃描 command、修改 `SessionInfo` struct、擴展 `get_sessions` 合併多來源、修改 `AppSettings` struct、修改 FS watcher 邏輯
- **前端型別（types/index.ts）**：`SessionInfo` 新增 `provider` 欄位、`AppSettings` 新增設定欄位、新增 `SisyphusData`/`OpenSpecData` 型別
- **前端元件**：重構 `ProjectView` 為分頁式架構、新增 `PlansSpecsView` 元件、修改 `SessionCard` 顯示 provider tag、新增篩選 UI 元件、修改 Dashboard 統計
- **新增依賴**：無（Rust 已有 `rusqlite`、`serde_yaml`）
- **資料庫**：`metadata.db` schema 無需變更
- **向後相容**：完全向後相容，既有功能不受影響；未安裝 OpenCode 或不存在 `.sisyphus`/`openspec` 時自動忽略
