## ADDED Requirements

### Requirement: Provider 標籤 icon 可辨識性

provider 標籤內的 provider icon SHALL 呈現可清楚辨識的內容（provider 縮寫或圖形），於 dark 與 light 主題下皆與底色有足夠對比；不得渲染為空白或近乎不可辨識的佔位圓點。

#### Scenario: Session 卡片 provider 標籤 icon

- **WHEN** session 卡片渲染任一 provider（copilot、opencode、codex、claude、antigravity）的標籤
- **THEN** 標籤前的 icon 顯示可辨識的 provider 縮寫（1–2 字，字級足以閱讀）或圖形，底色採 provider 專屬 accent

#### Scenario: 雙主題可辨識

- **WHEN** 使用者切換 dark / light 主題
- **THEN** provider icon 的內容（縮寫或圖形）與其底色 contrast ratio ≥ 3:1，肉眼可辨識

#### Scenario: icon 無法提供辨識價值時

- **WHEN** 某 provider 無法以縮寫或圖形呈現可辨識的 icon
- **THEN** 該標籤省略 icon，直接顯示文字 label，不保留空白佔位
