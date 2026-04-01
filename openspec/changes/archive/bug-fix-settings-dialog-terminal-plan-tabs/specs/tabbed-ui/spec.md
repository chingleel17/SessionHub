## MODIFIED Requirements

### Requirement: 固定 Dashboard 分頁
系統 SHALL 在分頁列最左側顯示固定的 Dashboard 分頁，不可關閉。頂層分頁列僅顯示 Dashboard 與 Project 分頁，不顯示 Plan 編輯器分頁。

#### Scenario: 應用程式啟動
- **WHEN** 應用程式啟動
- **THEN** Dashboard 分頁自動開啟且為當前作用分頁

#### Scenario: 頂層分頁不含 Plan
- **WHEN** 使用者在任何 Project 內開啟 Plan 編輯器
- **THEN** 頂層分頁列不新增 Plan 分頁；Plan 編輯器在 ProjectView 的子分頁區域開啟

## ADDED Requirements

### Requirement: ProjectView 子分頁結構
每個 Project 分頁 SHALL 包含第二層子分頁列，固定子分頁為：「Sessions」與「Plans & Specs」。使用者開啟 Plan 編輯器時，新增 `Plan:<sessionId>` 子分頁於此層，可獨立關閉。

#### Scenario: 進入 Project 分頁
- **WHEN** 使用者點擊 Project 分頁
- **THEN** ProjectView 顯示「Sessions」與「Plans & Specs」兩個固定子分頁

#### Scenario: 開啟 Plan 編輯器
- **WHEN** 使用者在 Session 卡片上觸發「開啟 Plan」
- **THEN** ProjectView 子分頁列新增 `Plan:<sessionId>` 子分頁並切換至該分頁

#### Scenario: 關閉 Plan 子分頁
- **WHEN** 使用者點擊 `Plan:<sessionId>` 子分頁的關閉按鈕
- **THEN** 該 Plan 子分頁關閉，ProjectView 回到前一個作用子分頁
