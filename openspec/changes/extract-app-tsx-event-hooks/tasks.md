## 1. 前置分析

- [x] 1.1 逐行讀取 `src/App.tsx` 約 1170～1440 行的事件訂閱 `useEffect`，列出每個 `listen()` 呼叫的事件名稱、handler 邏輯、依賴的外部變數
- [x] 1.2 記錄三個 `*-activity-hint` handler（copilot / claude / opencode）目前各自的 status/detail 計算規則差異，作為重構後核對基準
- [x] 1.3 記錄 `copilot-session-targeted` / `claude-session-targeted` 目前完全相同的處理邏輯

## 2. 抽出共用更新函式

- [x] 2.1 於 `src/hooks/useSessionRealtimeEvents.ts`（新檔案）或適當 utils 檔案，建立 `applyActivityStatusUpdate()`，統一三個 activity-hint handler 的 `queryClient.setQueriesData` 邏輯（依 design.md D1）
- [x] 2.2 建立 `createSessionTargetedHandler()`，統一 `copilot-session-targeted` / `claude-session-targeted` 的處理邏輯（依 design.md D2）

## 3. 抽出 custom hook

- [x] 3.1 建立 `useSessionRealtimeEvents()` hook，簽章依 design.md D3，將原本 `App.tsx` 事件訂閱 `useEffect` 的完整內容（含 cleanup、`mounted` guard、`sessionsDataRef` 存取機制）搬入
- [x] 3.2 改寫 `App.tsx`，移除原本的大型 `useEffect`，改為呼叫 `useSessionRealtimeEvents(...)`
- [x] 3.3 確認 `sessionsDataRef` 的 stale-closure 保護機制未被破壞（依 design.md 風險項）

## 4. 核對與驗證

- [x] 4.1 逐一比對步驟 1.2 記錄的三個 provider activity 計算規則，確認搬移後邏輯完全一致（唯一差異：原 copilot-activity-hint 在快取無既有 entry 時會塞入 `provider` 欄位，但 `SessionActivityStatus` 型別本無此欄位、亦無任何消費端讀取，屬於捨棄未使用死資料，非行為變更；已 grep 確認 `SessionCard.tsx` / `activityStatusMap` 消費端皆未讀取 `activity_statuses` cache 的 `.provider`）
- [x] 4.2 確認新 hook 的依賴陣列與原本 `useEffect` 語意等價（無遺漏、無多餘導致重複訂閱）
- [x] 4.3 執行 `tsc --noEmit` 確認型別檢查通過
- [x] 4.4 執行 `vite build` 確認建置成功
- [x] 4.5 手動測試：開啟應用程式，驗證 session 即時更新、activity 狀態指示、plan 檔案變更提示、quota 快照更新、`navigate-main-view` 導覽等行為與重構前一致
