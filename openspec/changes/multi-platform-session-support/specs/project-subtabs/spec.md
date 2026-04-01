## ADDED Requirements

### Requirement: 專案頁面分頁架構
系統 SHALL 將專案頁面（`ProjectView`）重構為包含子分頁的架構，預設包含兩個分頁。

#### Scenario: 預設分頁
- **WHEN** 使用者進入某個專案頁面
- **THEN** 頁面頂部顯示子分頁列（sub-tab bar），包含 "Sessions" 與 "Plans & Specs" 兩個分頁，預設選中 "Sessions"

#### Scenario: 切換分頁
- **WHEN** 使用者點擊 "Plans & Specs" 分頁
- **THEN** 頁面內容切換為 Plans & Specs 檢視，Sessions 分頁內容隱藏

#### Scenario: 分頁狀態保持
- **WHEN** 使用者在 "Plans & Specs" 分頁中切換到另一個專案再切回
- **THEN** 分頁狀態重置為預設的 "Sessions"（不持久化分頁選擇）

### Requirement: Sessions 分頁
系統 SHALL 將現有 session 列表功能完整保留於 "Sessions" 子分頁中。

#### Scenario: Sessions 分頁內容
- **WHEN** 使用者選中 "Sessions" 分頁
- **THEN** 顯示與目前 `ProjectView` 完全相同的 session 列表，包含：搜尋列、排序控制、標籤篩選、platform filter、session card 清單

#### Scenario: 功能無退化
- **WHEN** 使用者在 Sessions 分頁中執行任何既有操作（搜尋、排序、封存、開啟終端等）
- **THEN** 行為與重構前完全一致

### Requirement: Plans & Specs 分頁
系統 SHALL 在 "Plans & Specs" 子分頁中顯示該專案的 `.sisyphus` 與 `openspec` 資料。

#### Scenario: Plans & Specs 分頁內容結構
- **WHEN** 使用者選中 "Plans & Specs" 分頁
- **THEN** 頁面以 section 形式依序顯示：
  1. `.sisyphus` 區塊：Active Plan 狀態、Plans 清單、Notepads 清單、Evidence/Drafts（可收合）
  2. `openspec` 區塊：Active Changes 清單、Archived Changes（可收合）、Specs 清單

#### Scenario: 延遲載入
- **WHEN** 使用者首次切換到 "Plans & Specs" 分頁
- **THEN** 系統此時才發送 `get_project_plans` 與 `get_project_specs` 請求（lazy load），不在 Sessions 分頁時預載

#### Scenario: 兩者皆不存在
- **WHEN** 專案目錄下既無 `.sisyphus/` 也無 `openspec/` 目錄
- **THEN** Plans & Specs 分頁顯示空狀態提示（如「此專案尚無 plan 或 spec 資料」）

#### Scenario: 僅存在其中一者
- **WHEN** 專案目錄下僅存在 `.sisyphus/` 或僅存在 `openspec/`
- **THEN** Plans & Specs 分頁僅顯示有資料的區塊，另一個區塊顯示「尚無資料」提示或直接隱藏

### Requirement: Plan/Spec 內容預覽
系統 SHALL 在使用者點擊 plan 或 spec 條目時，顯示 Markdown 原始內容預覽。

#### Scenario: 點擊 plan 條目
- **WHEN** 使用者點擊某個 `.sisyphus/plans/*.md` 條目
- **THEN** 系統呼叫 `read_plan_content` 取得檔案內容，在 detail panel 中以 Markdown 預覽顯示

#### Scenario: 點擊 spec 條目
- **WHEN** 使用者點擊某個 `openspec/specs/*/spec.md` 條目
- **THEN** 系統呼叫 `read_plan_content` 取得檔案內容，在 detail panel 中以 Markdown 預覽顯示

#### Scenario: 外部編輯器開啟
- **WHEN** 使用者在內容預覽中點擊「以外部編輯器開啟」按鈕
- **THEN** 系統以設定中的外部編輯器路徑開啟該檔案
