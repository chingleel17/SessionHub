## Why

SessionHub 的 opencode bridge 插件（`sessionhub-provider-event-bridge.ts`）是由應用程式負責安裝到使用者的 opencode plugins 目錄。當插件邏輯有 bug 修復或格式升版時，已安裝的舊版插件不會自動更新，導致 bridge 事件持續無法正確寫入，且使用者毫不知情。

## What Changes

- **新增** Rust command：讀取已安裝插件的版本號（解析 header comment）
- **新增** Rust command：將最新版本插件寫入 opencode plugins 目錄（含路徑替換）
- **新增** 設定頁「opencode 整合」區塊：顯示插件狀態（未安裝 / 已安裝最新版 / 版本過舊）、提供安裝/更新按鈕
- **新增** 啟動時版本偵測：若插件未安裝或版本過舊，顯示提示 toast 引導使用者更新
- **修改** `provider-integration` spec：擴充版本管理相關 requirement

## Capabilities

### New Capabilities
- `plugin-installer`: 讀取已安裝插件版本、將最新版本插件寫入 opencode plugins 目錄（Rust backend）
- `plugin-version-check`: 啟動時自動偵測版本落差並提示使用者（frontend）
- `plugin-install-ui`: 設定頁 opencode 整合區塊，顯示狀態、提供安裝/更新操作（frontend UI）

### Modified Capabilities
- `provider-integration`: 擴充「安裝整合」scenario，加入版本偵測與升版流程

## Impact

- `src-tauri/src/lib.rs`：新增 `get_plugin_status`、`install_opencode_plugin` 兩個 Tauri command
- `src/App.tsx`：新增 useQuery 查詢插件狀態、新增 mutation 觸發安裝、啟動時 toast 提示
- `src/components/SettingsView.tsx`：新增 opencode 整合 UI 區塊
- `src/types/index.ts`：新增 `PluginStatus` 型別
- `src/i18n/`：新增插件狀態相關翻譯 key
- 不影響現有 session 掃描、SQLite schema、bridge JSONL 格式
