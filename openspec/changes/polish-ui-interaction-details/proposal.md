## Why

`standardize-frontend-ui-foundation` 完成共用元件基礎後，實際使用回饋指出多處互動細節仍不到位：session 卡片 provider 標籤前出現無意義的空白圓點、多數按鈕與下拉選單缺乏 hover 回饋、primary 按鈕 hover 使用淺色底導致白字難以辨識、設定頁「立即刷新」仍是文字按鈕，以及側欄收折動畫生硬且收折按鈕位置在兩種狀態下不一致。這些細節直接影響日常操作的辨識度與質感，應趁 UI 基礎剛建立時一次收斂。

## What Changes

- 修正 `ProviderIcon`：目前以 `background: currentColor` 搭配 8px 縮寫呈現，實際顯示為近乎空白的圓點；改為可清楚辨識的 provider 縮寫圖示（或在無法辨識時移除），不得再出現空白佔位。
- 設定頁 provider quota 監控卡片的「立即刷新」文字按鈕改為 `IconButton`（refresh 圖示 + tooltip），與 Dashboard/Tray 的刷新操作一致。
- 強化共用互動元件的視覺回饋：
  - 所有 Button variant（primary/secondary/ghost/danger）皆有可辨識的 hover 與 active 樣式及過渡動畫，不再只有游標變化。
  - primary/danger 的 hover 改為同色系加深（或提亮），不得改用淺色底造成白字對比不足。
  - Select 與 checkbox 提供統一的 hover、focus-visible 樣式與過渡，取代原生平面外觀。
  - 尚未套用共用樣式的既有按鈕（如 `ghost-button` 等）遷移到 `ui-button` 體系，確保 hover 規則全站生效。
- 側欄收折體驗重做：
  - 收折/展開加入平滑過渡動畫（寬度與內容淡出協調），並尊重 `prefers-reduced-motion`。
  - 收折按鈕在展開與收折狀態下維持相同的固定位置（不再於展開時移到品牌 icon 旁）。
  - 收折狀態下 app icon 與導覽 icon 的水平對齊一致，切換時不產生位移跳動。

## Capabilities

### New Capabilities
- `sidebar-collapse`: 側欄收折/展開的動畫、收折按鈕固定位置與 icon 對齊規則。

### Modified Capabilities
- `ui-primitives`: 補強按鈕/選單/checkbox 的 hover、active、過渡與對比要求（primary hover 不得降低文字對比）。
- `provider-tag`: provider 標籤內的 icon 必須呈現可辨識內容，不得為空白圓點。

## Impact

- 前端：`src/App.css`（ui-button、provider-icon、sidebar 收折相關規則）、`src/components/ProviderIcon.tsx`、`src/components/SettingsView.tsx`、`src/components/Sidebar.tsx`、`src/components/ui/*`。
- 不涉及 Rust/Tauri 後端、IPC 或資料層；純前端視覺與互動調整。
- 文案：如需新增 tooltip 文案，補充 `src/locales/zh-TW.ts`、`en-US.ts`（既有 `quota.monitoring.manualRefresh` 可沿用）。
