## Context

`SettingsView` 已取得 `providerDirectoryExists`，而後端固定回傳五個 provider 的 integration 狀態，因此平台啟用與偵測資訊可直接合併至 integration 管理，不需要在一般設定重複列出。變更動機見 `proposal.md`。

## Goals / Non-Goals

**Goals:**

- 以既有 `providerDirectoryExists` 建立一致的設定頁可用性語意，不增加 IPC 或另一套偵測狀態。
- 保留不可用 provider 的可見性與修復入口，避免停用後無法自行恢復。
- 使用語意化 disclosure 控制進階區，預設收折且支援鍵盤操作。
- 讓 provider 列在目前雙欄與單欄版面中都不發生水平溢出。

**Non-Goals:**

- 不改變 provider 根目錄的後端偵測方式。
- 不清理或自動改寫既有 `enabledProviders`、quota provider 設定。
- 不重新設計整個設定頁資訊架構或 provider integration 功能。

## Decisions

### 以根目錄偵測結果作為單一 UI 可用性來源

integration 與 quota 控制項都直接讀取 `providerDirectoryExists[provider]`。只有明確為 `false` 時停用；`undefined` 代表偵測尚未完成，維持目前可操作狀態，避免載入期間閃爍為不可用。

替代方案是從 `toolAvailability` 或 integration status 推導，但兩者分別代表 CLI 與 hook/plugin 狀態，無法正確代表 session 資料根目錄。

### 保留修復操作，僅停用選取與整合變更

未偵測 provider 的啟用與 quota checkbox，以及 install/update/uninstall 等變更操作停用；瀏覽根目錄、重新檢查與路徑修正仍可使用。這能同時滿足不可誤選與可自行恢復。

### 將平台啟用與偵測資訊整合至 integration 標題列

每個 integration 標題列顯示啟用 checkbox、provider 標籤、整合狀態與偵測勾勾。工具根目錄移至展開內容，與選填的 plugin、hook、bridge 路徑使用相同的淡色等寬文字；所有路徑移除 details、複製按鈕、背景與邊框，且選填欄位只渲染有值的項目。共用 `path-text` 及 `path-picker-button` 樣式同步套用到內容檢視器、MCP 與 Agents 既有路徑 UI。

替代方案是保留一般設定的 provider 清單，但會讓相同平台資訊分散在兩處，增加畫面長度與理解成本。

### 以原生 details/summary 實作進階區

進階設定使用未帶 `open` 的 `details`，自然得到預設收折、鍵盤切換與 `aria-expanded` 語意。內容仍位於原表單，不引入持久化的展開狀態。

替代方案是新增 React state 與自訂按鈕，但目前不需要跨頁記憶狀態，會增加不必要的控制邏輯。

## Risks / Trade-offs

- [已勾選但未偵測的 checkbox 會呈現 checked + disabled] → 保留既有設定值並以整列不可用樣式及狀態文案說明，不在 UI 階段默默刪除設定。
- [偵測狀態為 undefined 時短暫可操作] → 只將明確 false 視為不可用，避免初始載入錯誤停用；偵測完成後立即反映結果。
- [integration 卡片部分操作停用可能不夠直覺] → 整卡降對比，但讓重新檢查與路徑操作維持正常對比及可點擊游標。

## Migration Plan

這是純前端呈現與互動限制，無資料 migration。部署後沿用既有 settings；回復版本時也不需轉換設定檔。

### Provider 顯示順序

以 `PROVIDER_DISPLAY_ORDER` 與 `compareProviders` 作為所有前端 provider 清單的單一排序來源。畫面先依既有規則過濾未啟用或無資料項目，再使用 comparator 排序；未知 provider 排在已知 provider 之後並依名稱排序。
