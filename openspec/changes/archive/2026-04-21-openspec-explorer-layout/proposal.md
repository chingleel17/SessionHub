## Why

目前 PlansSpecsView 採用「手風琴展開」設計：點擊文件項目後，內容預覽出現在同一區塊內。當 spec 數量多時，使用者必須大幅捲動才能同時看到文件清單與內容，嚴重影響閱讀體驗（如截圖所示）。此外，錯誤訊息（Failed to read file）直接顯示在清單內，缺乏明確的視覺分區。

## What Changes

- **重新佈局 PlansSpecsView** 為左右雙面板設計：
  - **左側面板（Explorer 樹狀導覽）**：可收折的多層樹狀選單，結構類似 VSCode 檔案總管，支援巢狀展開（Sisyphus / OpenSpec 為根節點，Changes/Specs 為子節點，各 md 文件為葉節點）
  - **右側面板（內容檢視區）**：固定的 Markdown 純文字渲染區，選取左側任何文件後立即顯示於此，不影響左側捲動位置
- **左側面板可收折**：拖曳分隔線或點擊收折按鈕可縮小左側面板，增加檢視區空間
- **選取狀態追蹤**：左側高亮顯示當前選取的文件節點
- **移除舊的行內預覽**：廢除 `ChangeItem` 內嵌的預覽區塊，統一由右側面板顯示

## Capabilities

### New Capabilities

- `plans-specs-explorer-layout`：PlansSpecsView 重新設計為左右雙面板 Explorer 佈局，左側樹狀導覽 + 右側內容檢視

### Modified Capabilities

- `openspec-content-viewer`：展開/折疊切換行為改為「點擊節點選取並在右側面板顯示」，移除行內預覽，更新展開/折疊為純樹狀導覽行為

## Impact

- `src/components/PlansSpecsView.tsx`：整個元件重構，拆出 `ExplorerTree`、`ContentViewer` 子元件
- `src/styles/`：新增 `plans-specs-explorer.css`（或更新現有 CSS），添加雙面板佈局、樹狀節點、分隔線樣式
- 不影響 Rust backend（無 Tauri command 變更）
- 不影響 `src/App.tsx`（props 介面維持不變）
