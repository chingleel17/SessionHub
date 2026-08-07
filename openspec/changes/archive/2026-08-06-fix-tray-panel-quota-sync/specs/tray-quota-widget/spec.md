## MODIFIED Requirements

### Requirement: Tray 點擊彈出 Mini Panel

系統 SHALL 在使用者左鍵點擊 tray 圖示時，於系統匣附近彈出精簡 quota 面板，並於失焦時自動隱藏。panel 顯示的 quota 內容 SHALL 與 overlay widget 保持同步，兩者於同一時間點呈現相同的快照資料。

#### Scenario: 開關 panel

- **WHEN** `tray_quota_panel_enabled: true` 且使用者左鍵點擊 tray 圖示
- **THEN** 於系統匣上方彈出 320px 寬無框、不透明面板，顯示所有 enabled provider 的 quota 詳情（bar、百分比、reset 倒數、錯誤狀態）與刷新 icon 按鈕
- **AND** 無任何額度內容的 provider 顯示無額度資料說明文字
- **AND** panel 建立時設定 skip-taskbar，且每次 `show()` 之後重新呼叫 `set_skip_taskbar(true)`（同 overlay，對應 tauri#10422 Windows 樣式重置問題）
- **AND** panel 以 tray 所在螢幕的座標與 DPI 定位，不可在多螢幕環境超出可視範圍
- **AND** panel 失焦（blur）、再次點擊 tray 圖示或按 Esc 時隱藏
- **AND** panel 的自動隱藏邏輯不影響 overlay widget

#### Scenario: panel 內容即時同步

- **WHEN** panel 顯示中且後端 quota 快照更新（`"quota-snapshots-updated"` 事件觸發）
- **THEN** panel 於同一次更新中呈現與 overlay 相同的快照數值，不停留在舊值

#### Scenario: panel 顯示時取得最新快照

- **WHEN** panel 由隱藏或關閉狀態切換為顯示
- **THEN** panel 於顯示當下即呈現最新的 quota 快照，隱藏期間發生的更新不得遺漏
- **AND** 前次由 panel 開啟設定導致 webview 關閉後重新開啟時，亦適用同一保證

#### Scenario: panel 僅顯示已啟用監控的 provider

- **WHEN** panel 顯示 quota 清單且部分 provider 未列於 `quota_enabled_providers`
- **THEN** panel 僅列出已啟用監控的 provider，未啟用者完全不顯示
- **AND** 該過濾結果與 overlay、Quota Overview 對 enabled provider 的認定一致

#### Scenario: panel 停用時回復預設行為

- **WHEN** `tray_quota_panel_enabled: false` 且使用者左鍵點擊 tray 圖示
- **THEN** 開啟主視窗（與現有行為一致）
