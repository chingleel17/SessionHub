# Spec Delta

## MODIFIED Requirements

### Requirement: 系統匣圖示動態反映 quota 用量

系統 SHALL 在 quota monitoring 啟用時，依設定模式在系統匣圖示上動態反映主要 provider 的當前用量。在 macOS 上圖示為 template（單色）呈現，用量等級 MUST 不依賴顏色即可辨識。

#### Scenario: 依模式渲染 tray 圖示

- **WHEN** `enable_quota_monitoring: true` 且 `tray_quota_mode != hidden`，且 `"quota-snapshots-updated"` 事件觸發
- **THEN** 系統取 `tray_quota_primary_provider` 指定 provider（`None` 時自動取所有 status:ok provider 中最高用量 window）的 utilization
- **AND** 依 `tray_quota_mode` 重繪 tray 圖示：`icon_only` 疊彩色圓點 / `percentage` 疊百分比文字 / `bar` 疊進度條
- **AND** 顏色三段：綠 <50%、黃 50-80%、紅 >80%
- **AND** 更新 tooltip 為各 enabled provider 的多行用量摘要，window 名稱取自 `QuotaWindow.label`；無任何 window 的 provider 於 tooltip 中顯示無額度資料說明

#### Scenario: 隱藏模式

- **WHEN** `tray_quota_mode = hidden` 或 `enable_quota_monitoring = false`
- **THEN** tray 圖示維持原始 SessionHub icon，不疊加任何 quota 資訊

#### Scenario: macOS template 模式下的用量呈現

- **WHEN** 執行於 macOS 且 tray 圖示為 template 呈現
- **THEN** `icon_only` 模式以圖示上標記的填充程度或形狀變化表示用量三段（<50%、50-80%、>80%），不依賴顏色
- **AND** `percentage` 與 `bar` 模式以單色繪製，淺色與深色選單列皆清晰可讀
- **AND** tooltip 的多行用量摘要維持不變
