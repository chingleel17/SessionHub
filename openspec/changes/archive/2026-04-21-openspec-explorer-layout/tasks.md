# Tasks: openspec-explorer-layout

## Phase 1: CSS 雙面板佈局

- [x] 新增 `src/styles/plans-specs-explorer.css`，定義雙面板容器（`explorer-layout`）、左側面板（`explorer-panel`）、分隔線（`explorer-resizer`）、右側面板（`explorer-content`）的 flex 佈局樣式
- [x] 在左側面板加入折疊切換按鈕樣式（`explorer-panel--collapsed`）
- [x] 在右側面板加入頂部路徑標題列（`explorer-content-header`）與內容區（`explorer-content-body`）樣式
- [x] 定義樹狀節點樣式：`tree-node`、`tree-node--group`、`tree-node--leaf`、`tree-node--selected`、`tree-node--disabled`、`tree-node-indent`
- [x] 定義錯誤 banner 樣式（`explorer-error-banner`，紅色背景）
- [x] 在 `src/styles/index.css` 或 `App.css` 中 import 新 CSS 檔案

## Phase 2: TreeNode 型別與資料轉換

- [x] 在 `src/types/index.ts` 中新增 `TreeNode` 型別（id, label, icon, badge, children, selectable, filePath, filePathType）
- [x] 實作 `buildSisyphusTree(data: SisyphusData): TreeNode[]` 工具函式（輸出 Plans、Notepads、Evidence、Drafts 群組）
- [x] 實作 `buildOpenSpecTree(data: OpenSpecData): TreeNode[]` 工具函式（輸出 Active Changes、Archived Changes、Specs 群組，Change 子節點含 proposal/design/tasks/delta-specs 葉節點）

## Phase 3: ExplorerTree 元件

- [x] 建立 `src/components/ExplorerTree.tsx`，接受 `nodes: TreeNode[]`、`selectedId: string | null`、`onSelect: (node: TreeNode) => void` props
- [x] 實作遞迴 `TreeNodeItem` 元件，渲染群組節點（可展開/折疊）與葉節點（可選取）
- [x] 群組節點初始展開狀態：Active Changes 與 Sisyphus Plans 預設展開，其餘預設折疊
- [x] 葉節點點擊觸發 `onSelect`；群組節點點擊切換展開狀態，不觸發 `onSelect`

## Phase 4: ContentViewer 元件

- [x] 建立 `src/components/ContentViewer.tsx`，接受 `content: string | null`、`filePath: string | null`、`isLoading: boolean`、`error: string | null` props
- [x] 實作頂部路徑標題列（顯示 filePath，截取最後 2 層路徑作為顯示名稱）
- [x] 實作內容區：loading 狀態顯示 spinner、error 狀態顯示紅色 banner、正常顯示純文字（`<pre>` 或 `<div>` with whitespace-pre-wrap）
- [x] 空白狀態（content=null, loading=false, error=null）顯示提示文字

## Phase 5: PlansSpecsView 重構

- [x] 重構 `src/components/PlansSpecsView.tsx`：移除 `SisyphusSection`、`OpenSpecSection`、`ChangeItem`、`SpecItem` 行內預覽邏輯
- [x] 主元件 state 改為：`selectedNode: TreeNode | null`、`contentState: { content: string; filePath: string } | null`、`isLoading: boolean`、`error: string | null`、`explorerWidth: number`、`isExplorerCollapsed: boolean`
- [x] 使用 `buildSisyphusTree` 與 `buildOpenSpecTree` 產生 `TreeNode[]`，傳入 `ExplorerTree`
- [x] `onSelect` handler：判斷 node.filePathType（'absolute' 呼叫 `onReadFileContent`；'openspec' 呼叫 `onReadOpenspecFile`），更新 contentState
- [x] 實作分隔線拖曳 resize handler（mousedown → mousemove → mouseup，更新 explorerWidth CSS 變數）
- [x] 實作折疊切換 handler
- [x] 整合 `ExplorerTree` 與 `ContentViewer`，套用 `explorer-layout` 雙面板結構

## Phase 6: 翻譯鍵值更新

- [x] 在 `src/i18n/` 新增/更新翻譯鍵：`plansSpecs.explorer.selectPrompt`（右側空白提示）、`plansSpecs.explorer.collapsePanel`、`plansSpecs.explorer.expandPanel`

## Phase 7: 驗證

- [x] 執行 `bun run build` 確認 TypeScript 型別無錯誤
- [x] 手動測試：有 Sisyphus + OpenSpec 資料的 project，確認雙面板正確渲染
- [x] 手動測試：選取 change artifact、spec 文件，確認右側面板正確顯示
- [x] 手動測試：拖曳分隔線，確認寬度調整正常
- [x] 手動測試：折疊左側面板，確認右側面板擴展
- [x] 手動測試：讀取不存在的文件，確認右側顯示紅色錯誤 banner（非清單內錯誤）
