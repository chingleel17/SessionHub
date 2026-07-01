## 1. CSS Bug 修正

- [x] 1.1 在 `src/App.css` 新增 `.explorer-panel--collapsed .explorer-panel-header` 規則：設定 `border-bottom: none` 與 `min-height: unset`
- [x] 1.2 在 `src/App.css` 的 `.explorer-cols-entry` 加入 `box-sizing: border-box`

## 2. Cols 模式指令狀態邏輯

- [x] 2.1 在 `PlansSpecsView.tsx` 的 `renderColumnsPanel` 中，新增 `resolveChangeAction(entryNode)` 輔助函式，依 proposal/tasks artifact 存在與否及 progress 狀態回傳 `{ label, command, tone }`
- [x] 2.2 從 `entryNode.id` 解析 change-name（去除 `openspec:change:` 前綴），作為 slash command 的參數

## 3. Cols 模式 action 列 UI

- [x] 3.1 在 `renderColumnsPanel` 的每個 `explorer-cols-entry` 內，於進度條下方新增 `explorer-cols-action` 行，包含狀態 pill (`explorer-cols-action-label`) 與複製按鈕 (`explorer-cols-action-copy`)
- [x] 3.2 使用 `useState<string | null>` 追蹤「已複製的 entryNode.id」，複製後設定，500ms 後清除，用以顯示 ✓ 圖示
- [x] 3.3 複製按鈕點擊事件：呼叫 `navigator.clipboard.writeText(command)`，成功則設定已複製狀態；捕捉例外靜默處理

## 4. CSS：action 列樣式

- [x] 4.1 新增 `.explorer-cols-action`：flex 排列，align-items: center，justify-content: space-between，height: 18px
- [x] 4.2 新增 `.explorer-cols-action-label`：字號 10px，對應 tone 顏色（not_started=灰 #94a3b8、in_progress=橘 #f59e0b、done=綠 #22c55e）
- [x] 4.3 新增 `.explorer-cols-action-copy`：opacity: 0，transition，`.explorer-cols-entry:hover .explorer-cols-action-copy { opacity: 1 }`；已複製狀態顯示綠色 ✓
