## Why

目前 Plans & Specs 僅提供單一樹狀 Explorer，當 OpenSpec change、spec 與 Sisyphus 文件變多時，使用者很難快速切換不同層級的檢視方式，也無法在左側直接辨識 `proposal.md`、`design.md`、`tasks.md` 與任務進度狀態。既然已經完成新的視覺稿，現在需要把這套導覽模型正式定義成產品行為。

## What Changes

- 在 Plans & Specs 左側導覽加入 `Tree`、`List`、`Cols` 三種 explorer 檢視模式，使用者可隨時切換，並以每個專案為單位記住最後使用的模式。
- 對齊新的視覺稿修正三種模式的版型：Tree 僅 artifact 葉節點有 icon、List 改為列表列並顯示 spec 數量與 badge、Cols 改為兩欄且一次只展開一個狀態群組。
- 各檢視模式僅 `Active Changes` 群組預設展開，其餘群組預設折疊。
- 強化樹狀與其他檢視中的 OpenSpec change 呈現，為 `proposal.md`、`design.md`、`tasks.md` 顯示固定 icon。
- 對 `tasks.md` 與所屬 change 顯示任務進度摘要（例如 `2/5`）與狀態顏色，區分未開始、進行中、已完成。
- 保留既有右側內容檢視與 `tasks.md` 互動勾選能力，讓不同左側檢視共用同一套內容讀寫流程。

## Capabilities

### New Capabilities

- `openspec-task-progress`: 掃描 OpenSpec `tasks.md` 任務進度，並提供前端可用的 done/total/status 資訊。

### Modified Capabilities

- `plans-specs-explorer-layout`: 左側導覽從單一樹狀模式擴充為三種檢視模式，並加入 icon、進度 badge 與狀態色彩要求。

## Impact

- 前端：`src/components/PlansSpecsView.tsx`、`src/components/ExplorerTree.tsx`、`src/utils/buildTree.ts`、`src/App.css`、翻譯字串與型別。
- 後端：OpenSpec 掃描流程需回傳 change/task 進度資料。
- IPC / 型別：`OpenSpecData` 與 `OpenSpecChange` 回傳內容擴充，但不影響既有命令名稱。
