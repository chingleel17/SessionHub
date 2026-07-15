# Integration Card Collapse

## Why

「平台整合管理」區塊目前每張 provider 卡片都完整展開（路徑、Bridge 路徑、最後事件時間、整合版本、錯誤訊息），在 5 個 providers 全數顯示時頁面過長，使用者多數時候只需要確認「哪個平台裝了、版本多少、最近有沒有事件」。需要一個收折系統讓總覽更緊湊。

## What Changes

- 每張 provider 整合卡片改為可收折，**預設為收起**；有 `lastError` 的卡片例外，**預設展開**以便立即處理異常。
- 收起狀態僅顯示一列摘要：平台名 badge（維持現有 provider-tag 樣式）、安裝狀態 badge（已安裝／未安裝等）、版本 badge（如有）、最後事件時間（如無事件則顯示「尚無事件」文案）。
- 展開狀態顯示現有完整內容（操作按鈕、設定路徑、Bridge 路徑、最後事件時間、整合版本、錯誤訊息）。
- 點擊卡片標題列（badge 區域）切換收折／展開；操作按鈕不觸發切換。
- 收折狀態為前端 UI state（不持久化到設定檔），符合 YAGNI。
- 有 `lastError` 的卡片維持錯誤視覺提示（被手動收起時也可辨識）。

## Capabilities

### New Capabilities

- `integration-card-collapse`: 平台整合管理卡片的收折互動與收起摘要列顯示規則。

### Modified Capabilities

（無 — 現有 spec 未涵蓋此區塊的展示需求，純新增 UI 行為。）

## Impact

- `src/components/SettingsView.tsx`：卡片結構加入收折 state 與摘要列。
- `src/App.css`：新增收折相關樣式（依循 sessionhub-minimal-ui 設計 token）。
- `src/locales/zh-TW.ts`、`src/locales/en-US.ts`：新增展開／收起相關文案（如 aria/title）。
- 無後端（Rust）變更、無設定持久化變更。
