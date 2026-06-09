## 1. Quota Core

- [ ] 1.1 定義 provider quota snapshot 型別、狀態列舉與統一後端回傳模型
- [ ] 1.2 新增 quota manager 與內建 provider adapter 介面，支援統一調度 quota sources
- [ ] 1.3 實作 quota snapshot 快取與最後刷新時間管理，避免 UI 每次載入都直接查遠端來源

## 2. Refresh And Connector Flow

- [ ] 2.1 新增 quota 查詢 / 刷新 Tauri command，支援全量刷新與單一 provider 刷新
- [ ] 2.2 實作 app startup、背景輪詢與手動刷新流程
- [ ] 2.3 將 provider bridge 事件接入 quota refresh trigger，並加上 debounce / rate limit
- [ ] 2.4 預留插件式 quota connector 擴充點，明確區分內建 adapter 與外部 connector 載入邊界

## 3. Settings And Provider Diagnostics

- [ ] 3.1 擴充 AppSettings，加入 quota monitoring 開關與 refresh interval 設定
- [ ] 3.2 更新 Settings 頁，加入 quota monitoring 控制項與 provider quota diagnostics 顯示
- [ ] 3.3 在 provider integration 卡片加入 quota source、auth/錯誤狀態與手動 quota refresh 入口

## 4. Dashboard And Status Bar UI

- [ ] 4.1 在 Dashboard 加入 provider quota overview 區塊與最近更新資訊
- [ ] 4.2 在 global status bar 加入精簡 quota 摘要顯示，且不影響既有 session 活動資訊
- [ ] 4.3 確保 quota overview / status bar 在資料缺失、未支援或查詢失敗時有可理解的 fallback 顯示

## 5. Validation

- [ ] 5.1 補上 quota manager、adapter、快取與 refresh trigger 的後端測試
- [ ] 5.2 補上前端狀態與設定流程的測試或最小驗證案例
- [ ] 5.3 執行相關驗證指令並確認 OpenSpec change 進入可實作狀態
