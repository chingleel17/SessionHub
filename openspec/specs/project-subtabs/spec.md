## ADDED Requirements

### Requirement: ProjectView 子分頁架構

ProjectView SHALL 以子分頁（sub-tab）機制組織專案內的不同視圖，頂層分頁列只包含 Dashboard 與各專案。

#### Scenario: 子分頁清單（預設）

- **WHEN** 使用者進入一個專案 tab
- **THEN** ProjectView 子分頁包含：
  1. Sessions（永遠存在，顯示 session 列表）
  2. Plans & Specs（當專案目錄下有 openspec/ 資料夾時顯示）
  3. Analytics（永遠存在，顯示統計圖表查詢介面，固定位於最後）

#### Scenario: Plan sub-tab 動態新增

- **WHEN** 使用者從 session 卡片點擊開啟 plan
- **THEN** 在 ProjectView 子分頁列新增 `Plan: <session summary>` 子分頁
- **AND** sub-tab 以 session_id 為唯一 key

#### Scenario: Plan sub-tab 關閉

- **WHEN** 使用者點擊 plan sub-tab 上的 × 按鈕
- **THEN** 該 sub-tab 從列表中移除，視圖返回 Sessions

### Requirement: 跨專案切換 plan sub-tab 保留

已開啟的 plan sub-tab SHALL 在使用者切換專案後仍保留，當切回時仍可存取。

#### Scenario: 切換專案後切回

- **WHEN** 使用者切換至其他專案後再切回到有已開啟 plan 的專案
- **THEN** plan sub-tab 仍存在，且顯示相同內容

### Requirement: Sticky 子分頁列背景遮罩

ProjectView 的 sticky 子分頁列 SHALL 在 Sessions、Plans & Specs 與 Plan 子分頁中使用透明感外觀的遮罩容器，避免底下內容在捲動時透出，同時不形成厚重的實底色區塊。

#### Scenario: Sessions 子分頁捲動時遮罩內容

- **WHEN** 使用者在 Sessions 子分頁向下捲動 session 卡片列表
- **THEN** sticky 子分頁列與其外層容器 SHALL 以半透明或玻璃感遮罩完整蓋住底下卡片
- **AND** 不會出現 session 標題或 badge 從 sticky 區塊後方透出
- **AND** sticky 容器本身具有圓角，不呈現硬矩形底板

#### Scenario: Plans & Specs 子分頁維持清晰頂部

- **WHEN** 使用者切換至 Plans & Specs 子分頁並在 explorer 或內容面板捲動
- **THEN** 子分頁列頂部 SHALL 保持透明感遮罩與穩定層級
- **AND** 不會因透明背景而混入下方內容文字

#### Scenario: Header 內部不出現巢狀卡片感

- **WHEN** 使用者查看 sticky 子分頁列與篩選區塊
- **THEN** 系統 SHALL 以單一 header shell 呈現主要視覺邊界
- **AND** 內部的 toolbar 與 tag 區塊不再顯示額外的厚邊框或陰影卡片感
