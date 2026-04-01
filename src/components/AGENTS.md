# src/components/ — UI 元件

## OVERVIEW
純顯示元件層。所有元件只接 props，不持有業務邏輯，不呼叫 `invoke()`。

## FILES

| 檔案 | 用途 |
|------|------|
| `DashboardView.tsx` | 主頁統計總覽（session 數量、近期活動、專案分佈） |
| `ProjectView.tsx` | 單一專案的 session 列表；包含第二層子 Tab（Sessions、Plans & Specs、Plan 編輯器） |
| `SessionCard.tsx` | 單一 session 卡片（摘要、標籤、操作按鈕） |
| `PlanEditor.tsx` | plan.md 雙欄編輯器（原始 Markdown + 預覽），僅在 ProjectView 子 Tab 內渲染 |
| `SettingsView.tsx` | 設定表單（Copilot 路徑、終端、外部編輯器） |
| `Sidebar.tsx` | 左側導覽列（含即時狀態指示器） |
| `ConfirmDialog.tsx` | 通用確認 Dialog（tone: danger \| primary） |
| `EditDialog.tsx` | 通用文字輸入 Dialog（支援 multiline） |

## CONVENTIONS

- 每個元件接 props interface，在檔案頂部命名為 `{ComponentName}Props`
- Callback props 命名：`onXxx` 前綴（`onArchive`, `onOpenTerminal`...）
- 不得在元件內部呼叫 `invoke()`，一律透過 props callback 觸發
- CSS class 名稱對應元件語意，如 `session-card`, `plan-editor-pane`

## ProjectView 子 Tab 規則

- ProjectView 內有兩層 Tab：頂層（Dashboard / Project）與**子 Tab**（Sessions / Plans & Specs / Plan 編輯器）
- Plan 編輯器 Tab 以 `plan:{sessionId}` 為 key，在 ProjectView 內部 state 管理開啟清單
- Plan 相關的 invoke 呼叫（read_plan、write_plan、open_plan_external）仍由 App.tsx 負責，透過 props 傳入 ProjectView

## ANTI-PATTERNS

- 不得在元件內部管理 React Query（useQuery/useMutation）
- 不得直接 import `@tauri-apps/api` — 違反架構分層
- 不得在頂層 Tab 列渲染 PlanEditor — Plan 必須是 ProjectView 的子 Tab
