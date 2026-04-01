# src/ — React 前端

## OVERVIEW
React 19 + TypeScript 前端。所有 Tauri IPC 集中在 `App.tsx`，子元件為純顯示元件。

## STRUCTURE
```
src/
├── App.tsx          # 唯一業務邏輯層（所有 invoke、mutations、event listeners）
├── main.tsx         # ReactDOM.createRoot 入口，掛載 I18nProvider + QueryClientProvider
├── components/      # 顯示元件（View/Dialog 各一檔）
├── types/index.ts   # 所有共用 TS 型別
├── i18n/            # I18nProvider（useI18n hook）
├── locales/         # 翻譯 JSON
├── styles/          # 純 CSS（app.css, variables.css 等）
└── theme/           # 主題相關 CSS 變數
```

## WHERE TO LOOK

| 任務 | 位置 |
|------|------|
| 新增 invoke 呼叫 | `App.tsx` — 找現有 `useMutation` / `useQuery` 樣板 |
| 新增 UI 元件 | `components/` — 新增 `.tsx`，props 由 App.tsx 傳入 |
| 新增型別 | `types/index.ts` |
| 新增翻譯 key | `locales/zh-TW.json`（或其他語言檔）+ `i18n/I18nProvider.tsx` |
| 新增 CSS class | `styles/` 對應檔案，命名遵循 BEM-like |

## KEY PATTERNS

- **Query keys**：`["sessions", copilotRoot, showArchived]`、`["plan", sessionDir]`、`["settings"]`
- **Toast 系統**：`showToast(message)` → `toastMessage` state → 2600ms 後自動消失
- **Dialog 系統**：`setConfirmDialog({...})` / `setEditDialog({...})` → ConfirmDialog / EditDialog
- **activeView 路由**：`"dashboard"` | `"settings"` | `{projectKey}`。Plan 編輯器**不是**頂層路由，位於 ProjectView 的子 Tab 內，以 session ID 為 key。
- **planDraft**：plan 編輯的本地暫存，儲存時才寫入後端

## ANTI-PATTERNS

- 子元件不得直接 `invoke()` — 所有 IPC 只在 App.tsx
- 不得 hardcode 中文文字在 JSX — 必須用 `t("key")`
- 不得新增全域狀態管理庫（已有 React Query，避免引入 Redux/Zustand）
- 不得在頂層 Tab 顯示 Plan 編輯器 — Plan 必須是 ProjectView 子 Tab
