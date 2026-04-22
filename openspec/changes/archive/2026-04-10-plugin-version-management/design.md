## Context

SessionHub 透過一個 opencode plugin 檔案（`sessionhub-provider-event-bridge.ts`）來接收 opencode 的 session 事件。此插件必須手動安裝到使用者的 `~/.config/opencode/plugins/` 目錄。

現行問題：
1. 插件邏輯修復後（如本次 hook key 錯誤修復），已安裝的舊版插件不會自動更新
2. 應用程式無法得知使用者環境中的插件是否存在或是否為最新版本
3. 使用者不知道需要重新安裝，bridge 事件靜默失敗

版本資訊嵌入在插件的第一行 header comment 中：
```
// sessionhub-provider-event-bridge:{"provider":"opencode","bridgePath":"...","integrationVersion":2}
```

## Goals / Non-Goals

**Goals:**
- Rust backend 能讀取已安裝插件的 `integrationVersion`，與程式內建版本比對
- 提供 Rust command 將最新版本插件寫入正確目錄（含動態替換 `bridgePath`）
- 設定頁顯示插件狀態（not_installed / up_to_date / outdated）
- 應用程式啟動時若插件不是最新版，顯示 toast 提示

**Non-Goals:**
- 自動靜默更新插件（需使用者主動確認）
- 支援多個 opencode plugins 目錄（只處理 `~/.config/opencode/plugins/`）
- Copilot hook 的版本管理（Copilot 不使用插件機制）
- 插件 rollback 機制

## Decisions

### 決策 1：版本號嵌入 header comment，不用獨立版本檔

**選擇**：解析插件第一行的 JSON header comment 取得 `integrationVersion`。

**理由**：
- 插件檔案本身即為唯一真實來源，不需要額外的版本 sidecar 檔案
- 格式已在現有插件中建立（`integrationVersion: 2`），可直接沿用
- 若插件不存在，版本 = 0（視為 not_installed）

**捨棄方案**：獨立 `version.json` 檔 → 兩個檔案容易不同步。

---

### 決策 2：插件內容以 Rust 常數嵌入，不讀取外部檔案

**選擇**：在 `lib.rs` 中以 `const PLUGIN_TEMPLATE: &str = include_str!("...")` 或直接 string literal 儲存最新插件模板，安裝時替換 `bridgePath` 與 `BRIDGE_DIR` 佔位符。

**理由**：
- 應用程式是獨立的 `.exe`，不依賴外部資源路徑
- 版本升級時只需更新 lib.rs 中的常數與 `CURRENT_PLUGIN_VERSION`
- 避免打包時遺漏資源檔案

**捨棄方案**：Tauri asset bundle → 增加打包複雜度。

---

### 決策 3：`get_plugin_status` 為獨立 command，不合併進 `get_settings`

**選擇**：新增獨立的 `get_plugin_status` command 回傳 `PluginStatus`。

**理由**：
- `get_settings` 只讀 `settings.json`，插件狀態是 FS 查詢，職責不同
- 設定頁可以獨立 refetch（無需重新載入整份 settings）
- 符合現有「每個 command 一個職責」慣例

---

### 決策 4：啟動提示用 toast，不用全版 banner

**選擇**：啟動後在 `App.tsx` 的 `useEffect` 中查詢插件狀態，若 `outdated` 或 `not_installed` 則顯示一次性 `showToast`（warning 等級）。

**理由**：
- Toast 是現有的通知機制，不需要新元件
- 不阻塞主要使用流程
- 使用者可忽略，設定頁有完整操作入口

**捨棄方案**：persistent banner → 版面干擾，使用者無法關閉。

## Risks / Trade-offs

| 風險 | 緩解方式 |
|------|---------|
| `~/.config/opencode/plugins/` 路徑在特定環境不存在 | 安裝時以 `create_dir_all` 建立；若失敗回傳清楚的 `Err` 訊息 |
| 使用者手動修改了插件（自訂版本） | 版本號以 `integrationVersion` 為準；若使用者想保留自訂，不要按更新 |
| 插件 header comment 格式解析失敗 | parse 失敗視為 `integrationVersion = 0`（outdated），觸發更新提示 |
| Windows `USERPROFILE` 路徑含空格或特殊字元 | 插件內 `bridgePath` 使用 JSON string，已可正確 escape |

## Migration Plan

1. 此功能為純新增，不破壞現有功能
2. 已安裝舊版（`integrationVersion: 1`，hook key 錯誤版本）的使用者，啟動後看到 toast 提示，點擊前往設定頁更新即可
3. 不需要資料庫 migration
