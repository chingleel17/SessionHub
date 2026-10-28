# Tasks

## 1. 後端分層掃描

- [x] 1.1 為 AGENTS.md 與 Skills 加入 root/nested/all 參數並維持未指定參數的完整掃描；以 Rust 等價性、子目錄同名技能、忽略與預算測試驗證。
- [x] 1.2 合併指示檔 hash 與內容指標讀取；以 UTF-8、多語與非 UTF-8 指紋／metrics 測試驗證。

## 2. 前端漸進快取與顯示

- [x] 2.1 建立共用分層掃描調度與合併，涵蓋取消、保留舊 nested、背景失敗及寫入競爭；以真實 QueryClient 測試驗證。
- [x] 2.2 App 串接初載、手動刷新與寫入後更新，延後附帶全域載入，移除跨專案 placeholder；以快取／載入流程測試驗證。
- [x] 2.3 加入中英文背景載入／錯誤提示，檔案預覽不受背景掃描阻擋；以元件互動測試驗證根層可操作、空根層不誤報與深淺主題。

## 3. 整合驗證

- [x] 3.1 執行完整前端與 Rust 測試、lint、build、Rust fmt、git diff --check、OpenSpec strict validate，記錄結果並修正本次相關問題。

驗證：`bun test` 54 passed；Node hooks 8 passed；`cargo test` 320 passed、6 ignored；`bun run lint` 無錯誤（2 個既有 Hook 相依性警告）；`bun run build`、`cargo fmt -- --check`、`git diff --check` 與本 change 的 OpenSpec strict validate 均通過。
