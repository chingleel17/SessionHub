## ADDED Requirements

### Requirement: Explorer 排序切換器 UI
Explorer 標頭 SHALL 在 view 切換器旁顯示一組排序切換按鈕，供使用者選擇排序欄位與方向，無需開啟額外下拉選單。

#### Scenario: 排序切換器顯示
- **WHEN** Explorer 面板展開且標頭可見
- **THEN** view 切換器旁 SHALL 顯示三個排序按鈕：「進度」、「名稱」、「時間」

#### Scenario: 選中欄位顯示方向箭頭
- **WHEN** 使用者選擇某個排序欄位
- **THEN** 該按鈕 SHALL 顯示目前方向（升冪顯示 ↑，降冪顯示 ↓），其餘按鈕顯示中性圖示（⇅）

### Requirement: 排序欄位切換行為
點擊排序按鈕 SHALL 立即更新 change 清單順序，無需確認。

#### Scenario: 點擊未選中欄位
- **WHEN** 使用者點擊目前未選中的排序欄位按鈕
- **THEN** 系統 SHALL 切換至該欄位並設為升冪（asc），change 清單立即重新排列

#### Scenario: 點擊已選中欄位切換方向
- **WHEN** 使用者點擊目前已選中的排序欄位按鈕
- **THEN** 系統 SHALL 切換排序方向（asc ↔ desc），change 清單立即重新排列

### Requirement: 三種排序規則
Explorer SHALL 支援以下三種排序欄位，適用於 Tree / List / Cols 三種模式。

#### Scenario: 依名稱排序
- **WHEN** 排序欄位為「名稱」
- **THEN** change 清單 SHALL 依 change 名稱字母序（locale 比較）排列，升冪為 A→Z，降冪為 Z→A

#### Scenario: 依進度排序
- **WHEN** 排序欄位為「進度」
- **THEN** change 清單 SHALL 依任務完成率（done/total）排列；無 tasks.md 的 change 視為 -1（升冪排在最後，降冪排在最前）

#### Scenario: 依建立時間排序
- **WHEN** 排序欄位為「時間」
- **THEN** change 清單 SHALL 依 `.openspec.yaml` 的 `created` 欄位排列；無此欄位的 change 維持相對原序

### Requirement: 排序設定持久化
排序設定 SHALL 在同一專案的跨瀏覽 session 中保持。

#### Scenario: 重新開啟 Explorer 後排序設定保留
- **WHEN** 使用者設定排序後切換頁面再切回
- **THEN** Explorer SHALL 恢復上次的排序欄位與方向

### Requirement: 後端提供 createdAt 欄位
`OpenSpecChange` SHALL 包含 `createdAt` 欄位，由後端從 `.openspec.yaml` 的 `created` 欄位讀取。

#### Scenario: 有 created 欄位的 change
- **WHEN** 後端掃描一個 `.openspec.yaml` 內含 `created: YYYY-MM-DD` 的 change 目錄
- **THEN** 回傳的 `OpenSpecChange.createdAt` SHALL 為該日期字串

#### Scenario: 無 created 欄位的 change
- **WHEN** 後端掃描一個無 `.openspec.yaml` 或 `created` 欄位的 change 目錄
- **THEN** 回傳的 `OpenSpecChange.createdAt` SHALL 為 `null`
