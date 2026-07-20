## ADDED Requirements

### Requirement: Analytics 頁籤置於 Agents 之後

專案頁的 sub-tab 順序 SHALL 將 Analytics 頁籤排在 Agents 頁籤之後，維持 Sessions、Plans/Specs、Agents、Analytics 的先後順序。

#### Scenario: 專案頁 sub-tab 列渲染

- **WHEN** 使用者開啟任一專案頁
- **THEN** sub-tab 由左至右依序為 Sessions、Plans/Specs、Agents、Analytics，Analytics 位於 Agents 之後

### Requirement: Analytics 操作列水平排列與快速區間

Analytics 頁的控制列 SHALL 以水平方式排列快速區間、日期欄位與產生按鈕，避免自動換欄造成的空間浪費；快速區間 SHALL 提供「近一週」「本週」「近一個月」「本月」四個選項，且文案經 i18n（zh-TW / en-US）。

#### Scenario: 控制列排版

- **WHEN** 使用者開啟 Analytics 頁
- **THEN** 快速區間、開始/結束日期、分組方式與產生按鈕以水平列排列，產生按鈕靠右對齊

#### Scenario: 快速區間選項

- **WHEN** 使用者檢視快速區間按鈕
- **THEN** 顯示「近一週」（往前 7 天）、「本週」（本週一起算）、「近一個月」（往前 30 天）、「本月」（當月 1 日起算）四個選項，點擊後套用對應日期範圍

### Requirement: 趨勢圖數據線採用設計系統色票

趨勢圖的主要數據線（輸出 Token）SHALL 採用品牌主色 token（`--color-action-primary`），不得寫死非系統色值，並於 dark / light 主題自動對應。

#### Scenario: 趨勢圖主數據線配色

- **WHEN** 趨勢圖渲染輸出 Token 數據線
- **THEN** 該線採用 `--color-action-primary` 品牌主色，於雙主題下皆維持一致的視覺語言，不出現非設計系統的硬編碼色
