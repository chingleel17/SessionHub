## Why

Explorer 的 List 和 Cols 模式中，change 名稱與其他文字視覺權重相同，導致每個項目不夠清晰易讀；另外三種檢視模式（Tree / List / Cols）目前都只能依固定順序瀏覽 change，缺乏依進度、名稱或新增時間快速排序的能力。

## What Changes

- List 模式的 `.explorer-list-row-name` 與 Cols 模式的 `.explorer-cols-entry-name` 加上 `font-weight: 600`，與 Tree 模式的群組標籤視覺一致
- 在 Explorer 標頭的 view 切換器旁新增排序切換器，支援三個排序欄位（進度、名稱、新增時間）；點擊已選中的欄位切換升冪／降冪，點擊新欄位則設為升冪
- 排序設定以 `localStorage` 持久化（key 與 view mode 同層，per-project）
- 排序僅套用在 openspec change 清單；sisyphus 區段不受影響

## Capabilities

### New Capabilities

- `explorer-change-name-bold`: List 與 Cols 模式 change 名稱加粗
- `explorer-change-sort`: Explorer 排序切換器，支援進度 / 名稱 / 時間三欄位及升降冪切換

### Modified Capabilities

（無）

## Impact

- `src/components/PlansSpecsView.tsx`：新增 `sortField` / `sortDir` state、排序邏輯、排序切換器 UI
- `src/App.css`：`.explorer-list-row-name` 與 `.explorer-cols-entry-name` 加 `font-weight: 600`；新增排序切換器樣式
- `src/utils/buildTree.ts`：確認 `OpenSpecChange` 是否帶有建立時間欄位（供時間排序使用）
- `src/types/index.ts`：確認 `OpenSpecChange` 型別，若無 `createdAt` 欄位需評估替代方案
