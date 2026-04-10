## ADDED Requirements

### Requirement: 選單以 fixed overlay 渲染於最上層

系統 SHALL 以 `position: fixed` 渲染 launcher 下拉選單，使其顯示在所有看板卡片之上，不受 stacking context 影響。

#### Scenario: 選單不被其他卡片遮蓋

- **WHEN** 使用者點擊任意 SessionCard 的「⋯」按鈕
- **THEN** 展開的選單 SHALL 顯示在畫面最上層，不被同欄或其他欄的卡片遮蓋

#### Scenario: 選單位置跟隨觸發按鈕

- **WHEN** 選單展開
- **THEN** 選單 SHALL 出現在觸發按鈕附近（根據 `getBoundingClientRect` 動態計算 `top` / `left`）

### Requirement: 點擊外部自動關閉選單

系統 SHALL 在選單展開時監聽全域點擊事件，點擊選單 DOM 外部時自動關閉選單。

#### Scenario: 點擊卡片外部關閉選單

- **WHEN** 選單已展開
- **AND** 使用者點擊選單以外的任意區域
- **THEN** 系統 SHALL 自動關閉選單

#### Scenario: 點擊選單內部不關閉

- **WHEN** 選單已展開
- **AND** 使用者點擊選單內的選項
- **THEN** 選單 SHALL 在執行對應動作後關閉（現有行為不變）

### Requirement: 同時只能展開一個選單

系統 SHALL 確保在任何時間點最多只有一個 launcher 選單處於展開狀態。

#### Scenario: 開啟新選單時關閉舊選單

- **WHEN** 選單 A 已展開
- **AND** 使用者點擊另一個 SessionCard 的「⋯」按鈕
- **THEN** 選單 A SHALL 自動關閉
- **AND** 新選單 SHALL 展開

#### Scenario: 再次點擊觸發按鈕關閉選單

- **WHEN** 選單已展開
- **AND** 使用者再次點擊同一個「⋯」按鈕
- **THEN** 系統 SHALL 關閉選單（toggle 行為）
