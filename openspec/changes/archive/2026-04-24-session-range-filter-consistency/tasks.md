## 1. OpenSpec 與資料範圍對齊

- [x] 1.1 更新 project-subtabs、session-list、dashboard-stats-period、dashboard-kanban、dashboard 的 delta specs
- [x] 1.2 補齊本 change 的 proposal 與 design，描述時間篩選一致性與 sticky 背景需求

## 2. 前端實作

- [x] 2.1 在 App.tsx 建立 dashboard 共用時間篩選資料集，讓統計卡、專案預覽、最近活動與 Kanban 共用
- [x] 2.2 在 ProjectView 新增更新時間篩選、分頁與分頁重置邏輯
- [x] 2.3 調整 sticky project header、Plans & Specs 相關樣式與摘要文案，避免內容透出並清楚顯示篩選結果

## 3. 驗證

- [x] 3.1 執行既有前端 build
- [x] 3.2 執行既有 Rust 測試，確認 UI 相關調整未影響 Tauri 端
