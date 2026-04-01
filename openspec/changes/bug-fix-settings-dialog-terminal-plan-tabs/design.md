## Context

SessionHub 目前有五個已確認根因的 bug，影響設定頁面核心互動與導覽結構：

1. **dialog 權限缺失**：`capabilities/default.json` 缺少 `"dialog:default"`，導致所有 `open()` dialog 呼叫靜默失效。
2. **opencode_root 空白顯示**：`get_settings` 回傳 `opencode_root: ""`（舊版 settings.json 無此欄位時 serde default），前端顯示空白但後端實際用 fallback 正常運作。
3. **路徑驗證不足**：`validate_terminal_path_internal` 只檢查檔案存在，未確認 `file_stem()` 為合法終端機名稱。
4. **終端機指令硬綁 PowerShell**：`open_terminal_internal` 永遠帶 `-NoExit -Command` 參數，非 PowerShell 終端機會失敗。
5. **Plan Tab 層級錯誤**：Plan 編輯器 Tab 開在頂層，正確位置應是 ProjectView 內的第二層 sub-tab。

前端架構約束：所有 `invoke()` 呼叫集中於 `App.tsx`；子元件透過 props callback 觸發。本次修正遵守現有架構，不引入 hooks 分層（架構重構屬另一個獨立 change）。

## Goals / Non-Goals

**Goals:**
- 修復 dialog 無法開啟（加 `"dialog:default"` 權限）
- 修復設定頁面 opencode_root 顯示空白
- 強化終端機路徑驗證（file_stem 白名單）
- 依終端機類型帶入正確啟動參數
- 複製指令依 session provider 輸出對應語法
- Plan Tab 移入 ProjectView 第二層 sub-tab

**Non-Goals:**
- App.tsx 架構重構（hooks / features 分層）—— 獨立 change
- 新增測試框架 —— 獨立 change
- 支援 Linux / macOS 終端機
- Plan Tab 狀態的跨 Project 保留

## Decisions

### D1：dialog 權限修復方式
**決定**：直接在 `capabilities/default.json` 中加入 `"dialog:default"`。  
**理由**：plugin 已在 `Cargo.toml`、`package.json`、`run()` 中完整初始化，唯一缺少的是 capability 宣告。這是最小變更。  
**替代方案**：改用 Rust 側 `rfd` crate 自行實作 dialog —— 引入不必要依賴，捨棄。

### D2：opencode_root 補填位置
**決定**：在 `get_settings_internal` 回傳前，若 `opencode_root` 為空字串，以 `default_opencode_root()` 填入。  
**理由**：不改動 `AppSettings::default()` 或序列化邏輯，保持 settings.json 格式穩定；只在「讀取時」補填，前端永遠拿到有意義的值。  
**替代方案**：在 `save_settings` 時補填 —— 但若使用者從未開啟設定頁面就不會觸發，不適用。

### D3：終端機型別偵測
**決定**：`open_terminal_internal` 和 `validate_terminal_path_internal` 都從 `path.file_stem()` 取得名稱（case-insensitive），比對白名單 `["pwsh", "powershell", "cmd", "bash", "sh"]`。  
**理由**：`detect_terminal_path()` 已有此模式可參考，延伸一致性。  
**啟動參數分支**：
  - `pwsh` / `powershell` → `-NoExit -Command "cd '<dir>'"`
  - `cmd` → `/K "cd /d <dir>"`
  - `bash` / `sh` → `--rcfile /dev/null -i`（在 `<dir>` 作為 `current_dir`）

### D4：複製指令 provider 分支
**決定**：在 `handleCopyCommand` 中依 `session.provider`（或 session ID 前綴）判斷：
  - `"copilot"` → `copilot --resume=<id>`
  - `"opencode"` → `opencode session <id>`  
**理由**：最小改動，只影響前端 handler，不需新增 Rust command。  
**SessionInfo 確認**：`provider` 欄位已存在於 `SessionInfo` struct，前端 TS type 同步。

### D5：Plan Tab 移入 ProjectView
**決定**：
- 在 `App.tsx` 中保留 plan 相關 state（`openPlanKeys`, `planDraft`, `planQuery`, `planPreviewHtml`）和 handlers（`handleOpenPlan`, `handleSavePlan`, `handleOpenPlanExternal`），透過 props 傳入 `ProjectView`。
- `ProjectView` 新增 `plans` sub-tab 區域，包含「Sessions」、「Plans & Specs」兩個固定 sub-tab，以及動態開啟的 `Plan:<sessionId>` sub-tab。
- 頂層 Tab 只剩 `Dashboard` 和 `Project:<cwd>` 兩類，移除 `Plan:` 頂層 tab 邏輯。  
**理由**：符合現有「IPC 集中 App.tsx、子元件只接 props」架構約束，最小改動。

## Risks / Trade-offs

- **[Risk] Plan state 仍在 App.tsx**：多個 Project Tab 同時開啟時，不同 project 的 plan tabs 共用同一個 state map（以 session ID 為 key），不會混用但 App.tsx 稍微更肥。→ Mitigation：架構重構 change 解決。
- **[Risk] CMD / bash 啟動參數未在 Windows 環境測試**：`bash` 在 Windows 上通常是 Git Bash 或 WSL bash，`--rcfile /dev/null -i` 可能不適用所有版本。→ Mitigation：加上 `current_dir` 設定確保工作目錄正確；`bash`/`sh` 視為 best-effort。
- **[Trade-off] opencode_root 補填不寫回 settings.json**：使用者看到值但若直接關閉設定頁，下次重開仍會重新補填。→ 可接受，因為補填的值就是 default，行為一致。
