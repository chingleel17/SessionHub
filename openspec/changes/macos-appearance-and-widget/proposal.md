# Proposal

## Why

SessionHub 在 macOS 上的外觀與系統不一致：使用者把圖示風格切成灰色／tinted 後，Dock 圖示與右上角選單列（tray）圖示都沒有跟著變，看起來像是一個沒有適配 macOS 的 app。另外，使用者想要「小工具」形式的用量／session 狀態顯示，目前只有一個置頂 overlay 視窗，無法像 widget 一樣融入桌面或通知中心。

## What Changes

- **Dock／App 圖示**：提供可隨 macOS 圖示風格（預設、深色、clear／tinted）切換的 app icon 資源，並讓打包流程把它放進 `.app`。
- **選單列（tray）圖示**：macOS 上改用 template image（單色 alpha mask），由系統依淺色／深色／桌布亮度自動著色。原本以彩色圓點表示用量（綠／黃／紅）的繪製方式在 template 模式下無法呈現顏色，需定義替代呈現（形狀／填充量／數字）。
- **小工具顯示**：定義一份與視窗無關的「活動快照」（各平台 quota + 紅／黃／灰狀態與數量），供 widget 類顯示面共用；並評估兩條路線後擇一實作：
  - A. 沿用 Tauri 視窗（overlay／tray panel）加上外觀強化，不新增原生元件。
  - B. 原生 WidgetKit widget extension，經由 App Group 讀取快照檔。
- 不改變既有 Windows 行為；所有 macOS 專屬行為以 `cfg(target_os = "macos")` 隔離。

## Capabilities

### New Capabilities
- `macos-icon-appearance`: App 圖示與選單列圖示在 macOS 上跟隨系統圖示風格與外觀（淺色／深色／tinted）。
- `activity-snapshot-feed`: 提供與視窗無關的活動快照（quota 與 session 狀態）給 widget 類顯示面，包含資料格式、更新時機與過期處理。

### Modified Capabilities
- `tray-quota-widget`: 系統匣圖示的用量呈現在 macOS 上須在 template（單色）限制下仍可辨識用量等級。

## Impact

- **程式**：`src-tauri/src/tray_icon.rs`（繪製邏輯）、`src-tauri/src/app_setup.rs`（`TrayIconBuilder`）、`src-tauri/src/commands/notifications.rs`（`get_overlay_activity` 的合併邏輯抽出共用）、`src-tauri/icons/`、`src-tauri/tauri.conf.json`。
- **打包／簽章**：app icon 多外觀需要 Icon Composer 產物（`Assets.car`／`CFBundleIconName`）；原生 widget 需要 Xcode extension target、App Group entitlement 與簽章，與目前純 Tauri 打包流程有落差。
- **相依**：路線 B 需 Swift／WidgetKit；路線 A 無新增相依。
- **風險**：`tauri dev` 的未簽章 binary 看不到 app icon 風格與 App Group，驗證需用打包後的 `.app`。
