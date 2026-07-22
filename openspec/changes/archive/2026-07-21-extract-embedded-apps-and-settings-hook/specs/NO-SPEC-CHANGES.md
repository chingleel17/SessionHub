## 本次變更無需求層級（spec）異動

本 change（`extract-embedded-apps-and-settings-hook`）為純內部重構：

- 將 `EmbeddedQuotaOverlayApp`、`EmbeddedTrayPanelApp`、`RoutedApp` 搬到獨立檔案
- 將設定表單相關邏輯抽為 `useAppSettingsForm()` custom hook

不新增、不修改、不移除任何既有 capability 的行為需求。`statusbar-quota-popup`、`tray-quota-widget`、`app-settings`、`provider-integration` 等既有 spec 描述的行為（overlay/tray panel 渲染結果、設定儲存/讀取邏輯）維持不變，僅程式碼組織方式調整。因此本目錄不含任何 `ADDED / MODIFIED / REMOVED Requirements` delta 檔案。

驗證方式見 `tasks.md`：逐一核對搬移前後 mutation 的 callback 內容與 embedded webview 渲染結果一致。
