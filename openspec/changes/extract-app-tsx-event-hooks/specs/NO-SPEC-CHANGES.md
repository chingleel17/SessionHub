## 本次變更無需求層級（spec）異動

本 change（`extract-app-tsx-event-hooks`）為純內部重構：

- 消除 `copilot-session-targeted` / `claude-session-targeted` 事件處理的重複程式碼
- 消除三個 `*-activity-hint` 事件處理中重複的 `setQueriesData` 更新 pattern
- 將事件訂閱邏輯從 `App.tsx` 搬到獨立 custom hook

不新增、不修改、不移除任何既有 capability 的行為需求。`hook-driven-activity-status`、`session-activity-status`、`targeted-session-refresh`、`live-progress-sync` 等既有 spec 描述的行為（事件觸發時機、狀態更新結果）維持不變，僅程式碼組織方式調整。因此本目錄不含任何 `ADDED / MODIFIED / REMOVED Requirements` delta 檔案。

驗證方式見 `tasks.md`：逐一核對重構前後各 provider 的 activity 計算邏輯與事件訂閱生命週期一致。
