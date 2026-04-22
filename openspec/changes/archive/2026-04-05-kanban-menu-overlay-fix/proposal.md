## Why

SessionCard 的「選擇開啟方式」下拉選單（`⋯` 按鈕）存在三個互相關聯的 UX 缺陷：選單不受 z-index 控制，會被同欄其他卡片遮蓋；沒有點擊外部自動關閉機制；且多個選單可同時展開，導致使用者必須逐一點回觸發按鈕才能關閉。這些問題在看板欄位卡片密集時特別明顯，造成操作混亂。

## What Changes

- 選單改以 **`position: fixed`** + 高 `z-index` 渲染，永遠顯示在所有卡片之上
- 新增 **全域 click-outside 偵測**：點擊選單外部任意區域即自動關閉
- 實作 **單一選單同時開啟**：開啟新選單時，先關閉其他所有已展開的選單
- 選單位置動態跟隨觸發按鈕（`getBoundingClientRect`），避免跑版

## Capabilities

### New Capabilities

- `launcher-menu-overlay`: 選單以 fixed overlay 方式渲染，支援 click-outside 自動關閉與單例互斥邏輯

### Modified Capabilities

- `multi-ide-launcher`: 現有下拉選單的互動行為新增「同時只能開啟一個」與「點擊外部關閉」兩條 requirement

## Impact

- `src/components/SessionCard.tsx`：選單開關狀態需提升至父層或改用 context，以支援單例互斥
- `src/styles/`：新增 `.launcher-menu--fixed` 定位樣式與 z-index 層級
- `src/App.tsx`：可能需傳遞 `openMenuId` / `setOpenMenuId` props 給 SessionCard，或改用 React context 管理
