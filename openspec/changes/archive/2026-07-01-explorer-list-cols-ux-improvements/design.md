## Context

PlansSpecsView 的 Explorer 面板有三個檢視模式（Tree / List / Cols）。目前有兩個視覺 bug：

1. **收折 header 底線殘留**：收折後 `.explorer-panel-header` 的 `border-bottom` 仍顯示，且 `min-height: 50px` 使空白區域可見。
2. **Cols entry 名稱未填滿**：`.explorer-cols-entry` 缺少 `box-sizing: border-box`，導致 padding 計算不一致，name 底線視覺上未對齊外框右側。

此外，Cols 模式缺乏「下一步操作提示」：使用者必須記憶或手動輸入 slash command，才能對一個 change 執行 propose / apply / archive。

## Goals / Non-Goals

**Goals:**
- 收折後 header 不顯示 border-bottom，視覺乾淨
- Cols 模式 change 列名稱填滿外框寬度
- Cols 模式每個 change 列顯示「狀態 + 複製指令」按鈕，支援一鍵複製 slash command

**Non-Goals:**
- 不修改 Tree 模式或 List 模式的任何行為
- 不新增後端 IPC 呼叫
- 不實作執行指令功能（只複製，不發送）

## Decisions

### 指令狀態判斷邏輯

依 change 的 artifact 狀態推導「下一步」：

| 條件 | 狀態標籤 | Slash Command |
|------|---------|---------------|
| `proposal` artifact 不存在（`tone === "not_started"` 或無 children） | `待 propose` | `/opsx:propose` |
| `tasks` artifact 不存在或 progress 為 null | `可 apply` | `/opsx:apply` |
| progress.done < progress.total | `進行中 X/Y` | `/opsx:apply` |
| progress.done === progress.total（且 > 0） | `可封存` | `/opsx:archive` |

判斷來源：`entryNode.children`（artifact 節點陣列）與 `entryNode.progress`，這些資料已在 `buildOpenSpecTree` 時填入 TreeNode，無需額外 IPC。

指令格式：`/opsx:apply <change-name>`，change-name 從 `entryNode.id` 解析（去掉 `openspec:change:` 前綴）。

### 複製機制

使用 `navigator.clipboard.writeText()`。複製後在按鈕旁短暫顯示「✓」（500ms timeout），不用 toast，避免干擾主內容區。

### UI 位置

在 `explorer-cols-entry-top` 下方新增 `explorer-cols-action` 行，包含：
- 左側：狀態 pill（`explorer-cols-action-label`），顏色對應狀態（not_started=灰、in_progress=橘、done=綠）
- 右側：複製按鈕（`explorer-cols-action-copy`），hover 才顯示，點擊後切換為 ✓ icon

收折狀態（`isCollapsed`）不受影響，action 行只在 Cols 模式的 master 欄中出現。

### CSS 修正方式

- `explorer-panel--collapsed .explorer-panel-header`：override `border-bottom: none` 與 `min-height: unset`
- `explorer-cols-entry`：加入 `box-sizing: border-box`

## Risks / Trade-offs

- [Risk] `entryNode.id` 格式若變更會導致 change-name 解析錯誤 → Mitigation：加 fallback，解析失敗時 command 不帶參數
- [Risk] `navigator.clipboard` 在某些 Tauri WebView 設定下可能需要權限 → Mitigation：加 try/catch，失敗時靜默（不顯示 ✓）
