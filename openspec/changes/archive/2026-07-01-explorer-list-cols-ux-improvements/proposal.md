## Why

Explorer 左側面板在收折狀態下 header 底線仍然顯示、內容區空白，造成視覺殘留；Cols 模式的 change 列名稱底線未填滿外框，且目前缺乏根據 change 狀態快速複製 slash command 指令的能力，使用者需要手動記憶或切換視窗才能執行下一步操作。

## What Changes

- 修正收折狀態 (`explorer-panel--collapsed`) 下 header `border-bottom` 的殘留顯示問題
- 修正 Cols 模式 `.explorer-cols-entry` name 區塊底線未填滿外框的問題
- 在 Cols 模式左側每個 change 列新增「智慧指令按鈕」，依 change 狀態（未 propose、可 apply、進行中、可封存）顯示對應 slash command，並支援一鍵複製

## Capabilities

### New Capabilities

- `explorer-collapsed-header-fix`: 收折後 header 隱藏 border-bottom，保持視覺乾淨
- `cols-entry-name-fill-fix`: Cols 模式 change 列名稱欄位填滿可用寬度，底線對齊外框
- `cols-change-action-command`: 依 change 進度狀態自動判斷對應 slash command，在 Cols 左側顯示狀態標籤與複製按鈕

### Modified Capabilities

- `plans-specs-explorer-layout`: 折疊後標題列視覺行為變更（移除 border-bottom）

## Impact

- `src/components/PlansSpecsView.tsx`：Cols 模式 renderColumnsPanel 新增指令狀態邏輯與複製按鈕
- `src/App.css`：新增 `.explorer-panel--collapsed .explorer-panel-header`、`.explorer-cols-entry` box-sizing 修正、新增 `.explorer-cols-action` 相關樣式
- 無後端變更、無 API 變更
