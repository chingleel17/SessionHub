## 1. 共用 Modal 元件

- [x] 1.1 在 `src/components/ui/` 建立實心 Modal 容器，集中 backdrop、dialog semantics、面板 class 合併與 theme-token 樣式，並以元件測試或渲染檢查確認具 `role="dialog"`、`aria-modal="true"` 及不透明面板 class
- [x] 1.2 將 `EditDialog` 與 Skills 預覽遷移至共用 Modal，同時保留表單與預覽各自尺寸／捲動 class，並於深色與淺色主題手動確認遮罩、面板、輸入欄位及文字皆清楚且背景不穿透

## 2. 前端 Metadata 狀態同步

- [x] 2.1 實作依 session id 更新 `sessions` 與 `sessions_cached` 所有相符 React Query cache 的型別安全 helper，並以單元測試驗證新增、修改、清除 notes/tags 只影響目標 session
- [x] 2.2 在 `saveMetaMutation` 成功後套用 cache helper 並保留背景 invalidation，在失敗時顯示錯誤且不寫入 cache；以測試驗證成功與失敗流程，並手動確認卡片及重新開啟的編輯器立即顯示最新值

## 3. Session Card 資訊精簡

- [x] 3.1 從 Project 頁面的 `SessionCard` 移除 Git Repo 標題與 repository 名稱，保留 `repoName` 資料型別供其他功能使用，並以元件測試或渲染檢查確認卡片只顯示更新時間、建立時間與 Summary 數量
- [x] 3.2 調整 `session-meta-grid` 的三欄桌面與窄視窗排列，並手動確認沒有多餘空白欄、溢位或可讀性退化

## 4. 後端 Metadata 一致性

- [x] 4.1 在 session 清單共同回傳路徑以 SQLite `session_meta` 覆寫每筆 session 的 notes/tags，確保 full scan、incremental scan 與記憶體 cache 都使用最新 metadata，並以 Rust 測試驗證 stale provider cache 不會覆蓋新值
- [x] 4.2 增加 metadata 新增、更新及清除後重新取得 sessions 的 regression tests，並執行 `cargo test` 確認所有 provider 與既有 session cache 測試通過

## 5. 整合與品質檢查

- [x] 5.1 手動完成標籤新增／編輯／刪除與備註新增／編輯／清除流程，確認卡片、再次編輯、重新整理及重啟後均顯示正確資料
- [x] 5.2 執行 `bun run lint`、`bun run build`、`cargo test` 與 `openspec validate fix-session-metadata-modals --strict`，確認前後端品質檢查及規格驗證全部通過
