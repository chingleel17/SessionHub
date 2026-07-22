## 本次變更無需求層級（spec）異動

本 change（`cleanup-deps-and-settings-defaults`）為純內部重構：

- 移除未使用的前端相依套件（`react-router-dom`、`@codemirror/lang-markdown`、`@codemirror/view`）
- 收斂 `AppSettings` 預設值的重複定義來源（前端三處、後端一處 → 各自單一來源）

不新增、不修改、不移除任何既有 capability 的行為需求。`openspec/specs/app-settings/spec.md` 描述的欄位清單與預設值語意維持不變，僅實作方式（程式碼組織）調整。因此本目錄不含任何 `ADDED / MODIFIED / REMOVED Requirements` delta 檔案。

驗證方式見 `tasks.md`：以逐欄核對確保重構前後行為一致。
