## Why

設定頁目前在窄欄寬下會讓 provider 名稱、路徑與偵測狀態互相擠壓或溢出，而且平台啟用、整合管理與 quota 監控對「未偵測」狀態的互動規則不一致。一般設定也混入多個低頻路徑與進階欄位，使主要控制項不易瀏覽。

## What Changes

- 將 provider 啟用控制、根目錄編輯與偵測指示整合至平台整合管理，移除一般設定中的重複平台清單。
- 每個平台整合列展開後顯示工具根目錄與選擇資料夾操作；plugin、hook 與 bridge 路徑僅在有值時以統一的淡色無外框文字顯示。
- 專案內既有的路徑文字與檔案／資料夾選擇 icon 採共用樣式，避免各功能各自呈現。
- 所有 provider 清單統一依 Claude Code、OpenCode、Codex、GitHub Copilot CLI、Antigravity 排序，未啟用項目照常略過。
- 已偵測 provider 僅顯示成功圖示，不顯示狀態文字；未偵測時指示位置留空、整列反灰且不能啟用。
- 平台整合管理與 quota provider 選項沿用同一份根目錄偵測結果；未偵測 provider 反灰並停用整合操作與 quota 勾選。
- 未偵測 provider 的路徑編輯入口維持可用，讓使用者可指定有效資料根目錄後重新偵測。
- 新增預設收折的「進階設定」區，收納終端程式路徑、外部編輯器路徑、Analytics 自動重整間隔、Agents 正本根目錄與專案 `.sessionhub` 建立權限。
- 重排一般設定的啟動、系統匣與通知開關，並將顯示已封存及底部狀態列移至進階設定最前方。
- 保留目前設定值，不因控制項移入收折區或 provider 暫時未偵測而自動刪除已儲存值。

## Capabilities

### New Capabilities

無。

### Modified Capabilities

- `app-settings`: 調整 provider 根目錄偵測後的啟用規則、設定頁響應式呈現、平台整合不可用狀態，以及低頻設定的預設收折分組。
- `provider-quota-monitoring`: quota provider 選項須依 provider 根目錄偵測結果停用未偵測項目。

## Impact

- 前端設定頁：`src/components/SettingsView.tsx`
- 設定頁樣式與響應式規則：`src/App.css`
- 中英文設定文案：`src/locales/zh-TW.ts`、`src/locales/en-US.ts`
- 不新增後端欄位、IPC command 或第三方相依套件。
