## Why

SessionHub 目前已能掃描 Codex session，但尚未把 Codex 接入 provider bridge / hook integration，導致 Codex 只能依賴檔案 watcher 刷新，無法與 Copilot、OpenCode 共用一致的即時事件管線。現有 provider integration 腳本與模板邏輯也偏向混在一起管理，擴充第三個 provider 後會增加維護與除錯成本。

## What Changes

- 新增 Codex provider integration，讓 SessionHub 可安裝、更新、檢查 Codex hook 設定，並接收標準化 bridge 事件。
- 調整 provider integration 狀態管理與設定頁顯示，讓 Codex 與 Copilot、OpenCode 一樣可查看安裝狀態、設定路徑、bridge 路徑與最後事件時間。
- 將 provider integration 產物拆成 provider-specific 腳本或模板資產，分開管理 Copilot、OpenCode、Codex 的 hook / plugin 內容，避免三者混用在同一段模板組裝邏輯。
- 更新 watcher 策略，當 Codex integration 已安裝時，優先使用 bridge 事件觸發 refresh；未安裝時才退回既有檔案 watcher。

## Capabilities

### New Capabilities
- `codex-provider-integration`: 定義 Codex hook 的安裝位置、bridge 事件格式、狀態檢查與更新流程。

### Modified Capabilities
- `provider-integration`: 擴充既有 provider integration 規格，納入 Codex 並要求 provider-specific hook / plugin 腳本分離管理。
- `app-settings`: 調整設定頁 provider integration 狀態需求，讓 Codex integration 與其路徑資訊一併顯示與操作。
- `file-watcher`: 更新 bridge 優先、filesystem fallback 的規則，將 Codex 納入相同策略。

## Impact

- Rust backend: `src-tauri/src/provider/*`、`src-tauri/src/watcher.rs`、`src-tauri/src/settings.rs`、provider integration commands 與狀態型別。
- Frontend: `src/App.tsx`、`src/components/SettingsView.tsx` 與 provider integration 相關文案、操作流程。
- OpenSpec: `provider-integration`、`app-settings`、`file-watcher` delta specs，以及新增 `codex-provider-integration` capability。
- Managed integration assets: Copilot hook、OpenCode plugin、Codex hook 的模板或腳本檔案需改成分開維護，以降低耦合並方便後續版本管理。
