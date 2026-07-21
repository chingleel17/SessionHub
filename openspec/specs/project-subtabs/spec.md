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

ProjectView 的 sticky 子分頁列 SHALL 呈現為 Minimal 設計語言下的連續表面：不使用厚重實底色區塊、粗邊框或重陰影，僅以極淡邊框與輕量遮罩維持捲動時的層級。

#### Scenario: Sessions 子分頁捲動時遮罩內容

- **WHEN** 使用者在 Sessions 子分頁向下捲動 session 卡片列表
- **THEN** sticky 子分頁列 SHALL 以不透明或輕量遮罩蓋住底下內容，不出現內容透出
- **AND** sticky 容器使用 `--radius-card`，不呈現硬矩形底板
- **AND** 容器 SHALL NOT 使用重陰影（僅 `--shadow-panel` 或無陰影）或粗邊框

#### Scenario: Plans & Specs 子分頁維持清晰頂部

- **WHEN** 使用者切換至 Plans & Specs 子分頁並在 explorer 或內容面板捲動
- **THEN** 子分頁列頂部 SHALL 保持透明感遮罩與穩定層級
- **AND** 不會因透明背景而混入下方內容文字

#### Scenario: Header 內部不出現巢狀卡片感

- **WHEN** 使用者查看 sticky 子分頁列與篩選區塊
- **THEN** 系統 SHALL 以單一 header shell 呈現主要視覺邊界
- **AND** 內部的 toolbar 與 tag 區塊不再顯示額外的厚邊框或陰影卡片感

### Requirement: 子分頁列採 Linear/Apple 式頁籤樣式

ProjectView 的 sub-tab 項目 SHALL 採用 Minimal 頁籤樣式：active 以底部 accent line 標示，inactive 無背景、以次要文字色呈現，全站一致，不使用 Bootstrap 式帶框頁籤。

#### Scenario: Active 頁籤樣式

- **WHEN** 某個 sub-tab 為 active
- **THEN** 該頁籤底部顯示 `2px` 的 `--color-action-primary` accent line
- **AND** 文字為主要文字色

#### Scenario: Inactive 頁籤樣式

- **WHEN** 某個 sub-tab 為 inactive
- **THEN** 該頁籤無背景填色、無邊框，文字為 `--color-text-secondary`
- **AND** hover 時文字轉為主要文字色，過場採 `--motion-fast` `ease-out`

### Requirement: Sessions 內容區採開放留白與 row-hover 清單

Sessions 分頁的內容區 SHALL 保持開放感（增加留白、減少邊框），session 列表 SHALL 避免粗邊框與重陰影的卡片堆疊，改以 row-hover 柔和背景切換或輕量卡片呈現。

#### Scenario: Session 列 hover 樣式

- **WHEN** 使用者將游標移至一個 session 列/卡片
- **THEN** 該列以柔和背景切換標示（Light `rgba(0,0,0,0.03)`、Dark `rgba(255,255,255,0.04)`），過場採 `--motion-fast` `ease-out`
- **AND** session 列/卡片邊框為 `--color-border-subtle` 或無邊框，圓角採 `--radius-card`，陰影不超過 `--shadow-panel`

#### Scenario: 內容區留白與層級

- **WHEN** 使用者檢視 Sessions 分頁
- **THEN** 內容區以留白與字級/字重建立層級，SHALL NOT 以多層巢狀卡片或粗分隔線切割
