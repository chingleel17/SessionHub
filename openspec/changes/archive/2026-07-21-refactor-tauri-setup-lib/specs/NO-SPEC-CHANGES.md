## 本次變更無需求層級（spec）異動

本 change（`refactor-tauri-setup-lib`）為純內部重構：

- 將 `lib.rs` 的 `pub fn run()` 內 quota monitoring 啟動、背景輪詢執行緒、tray icon 建構拆成具名函式
- 將 tray 選單的 4 個硬編碼字串抽成具名常數並集中定義

不新增、不修改、不移除任何既有 capability 的行為需求。`tray-quota-widget`、`provider-quota-monitor`、`single-instance-lock` 等既有 spec 描述的行為（背景刷新排程、tray 選單功能與外觀）維持不變，僅程式碼組織方式調整。因此本目錄不含任何 `ADDED / MODIFIED / REMOVED Requirements` delta 檔案。

驗證方式見 `tasks.md`：核對拆分後函式呼叫順序與背景執行緒捕獲變數語意與重構前一致。
