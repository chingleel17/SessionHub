## ADDED Requirements

### Requirement: tasks.md 變更後左側進度即時同步
當使用者在外部編輯器更新 tasks.md 的 checkbox 狀態後，Plan 介面左側的 change 進度條與 badge 數字 SHALL 在 watcher 觸發後約 600ms 內自動更新，無需手動切換頁面或重整。

#### Scenario: 勾選 tasks.md 中的任務項目
- **WHEN** 使用者在外部編輯器將 tasks.md 中的 `- [ ]` 改為 `- [x]` 並儲存
- **THEN** 系統 SHALL 透過 watcher 偵測到 openspec/ 目錄的檔案變更，觸發 `project-files-changed` 事件，最終在 600ms 內更新左側 badge（如 `3/5` → `4/5`）與進度條寬度

#### Scenario: 路徑格式一致性
- **WHEN** `project-files-changed` 事件的 payload（project_dir）與 `activeProject.pathLabel` 的格式存在差異（如大小寫、斜線方向）
- **THEN** 系統 SHALL 在比較前將兩者正規化（統一小寫、統一使用正斜線），確保 invalidateQueries 能正確命中 `["project_specs", pathLabel]` 快取鍵

#### Scenario: 多個任務項目快速連續修改
- **WHEN** 使用者在短時間內連續儲存 tasks.md 多次
- **THEN** 系統 SHALL 透過去抖動機制（watcher 100ms + App.tsx 500ms）合併多次觸發，最終只執行一次 openspecQuery 重整，避免 UI 閃爍
