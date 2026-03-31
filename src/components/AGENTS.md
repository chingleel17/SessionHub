# src/components/ — UI 元件

## OVERVIEW
純顯示元件層。所有元件只接 props，不持有業務邏輯，不呼叫 `invoke()`。

## FILES

| 檔案 | 用途 |
|------|------|
| `DashboardView.tsx` | 主頁統計總覽（session 數量、近期活動、專案分佈） |
| `ProjectView.tsx` | 單一專案的 session 列表（含篩選、排序、封存切換） |
| `SessionCard.tsx` | 單一 session 卡片（摘要、標籤、操作按鈕） |
| `PlanEditor.tsx` | plan.md 雙欄編輯器（原始 Markdown + 預覽） |
| `SettingsView.tsx` | 設定表單（Copilot 路徑、終端、外部編輯器） |
| `Sidebar.tsx` | 左側導覽列（含即時狀態指示器） |
| `ConfirmDialog.tsx` | 通用確認 Dialog（tone: danger \| primary） |
| `EditDialog.tsx` | 通用文字輸入 Dialog（支援 multiline） |

## CONVENTIONS

- 每個元件接 props interface，在檔案頂部命名為 `{ComponentName}Props`
- Callback props 命名：`onXxx` 前綴（`onArchive`, `onOpenTerminal`...）
- 不得在元件內部呼叫 `invoke()`，一律透過 props callback 觸發
- CSS class 名稱對應元件語意，如 `session-card`, `plan-editor-pane`

## ANTI-PATTERNS

- 不得在元件內部管理 React Query（useQuery/useMutation）
- 不得直接 import `@tauri-apps/api` — 違反架構分層
