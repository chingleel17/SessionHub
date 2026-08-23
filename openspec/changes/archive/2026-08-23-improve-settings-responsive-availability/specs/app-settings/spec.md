## MODIFIED Requirements

### Requirement: 設定頁 provider integration 版面需具備響應式回退

系統 SHALL 在較窄視窗或空間不足時，讓 provider integration 管理區塊回退為可閱讀的堆疊式布局，且所有名稱、狀態與操作 SHALL 保持在容器範圍內。

#### Scenario: 視窗寬度不足

- **WHEN** 設定頁可用寬度不足以容納寬版排版
- **THEN** 系統將 provider integration 內容回退為堆疊式布局
- **AND** 長路徑以截斷或換行方式呈現，不得水平溢出所屬容器
- **AND** 使用者仍可看到所有狀態，並操作目前可用的功能

### Requirement: Provider 資料根目錄偵測狀態提示

設定頁 SHALL 將 provider 啟用控制、根目錄編輯與資料根目錄偵測指示整合於平台整合管理，並以資料根目錄是否存在作為 provider 啟用與相關管理操作的可用性依據；CLI 可執行檔是否存在 SHALL NOT 取代此判定。一般設定區 SHALL NOT 重複顯示 provider 清單。

#### Scenario: 資料根目錄存在

- **WHEN** 設定頁渲染某 provider 且其設定的資料根目錄存在
- **THEN** 該 provider 整合列以正常樣式顯示
- **AND** 偵測指示僅顯示成功圖示，不顯示「已偵測到」文字
- **AND** 該 provider 可被勾選啟用

#### Scenario: 資料根目錄不存在

- **WHEN** 設定頁渲染某 provider 且其設定的資料根目錄不存在
- **THEN** 該 provider 整合列與相關選項以低對比不可用樣式顯示
- **AND** 偵測指示位置留空，不顯示「未偵測到」文字
- **AND** 該 provider 的啟用勾選框停用，不接受選取變更
- **AND** 路徑編輯入口維持可用，以便使用者修正根目錄

#### Scenario: 已啟用 provider 暫時未偵測

- **WHEN** 已儲存於 `enabled_providers` 的 provider 根目錄暫時不存在
- **THEN** 設定頁保留其勾選狀態但停用控制項
- **AND** 系統不得只因本次偵測失敗自動移除已儲存設定

#### Scenario: 勾選可用性不受 CLI 安裝狀態影響

- **WHEN** 某 provider 的 CLI 可執行檔不存在於 PATH，但其資料根目錄存在
- **THEN** 該 provider 仍可被勾選啟用，其既有 session 歷史不因此被隱藏

### Requirement: 狀態列開關設定

系統 SHALL 在設定頁進階設定區塊提供開關，讓使用者啟用或停用全域狀態列。

#### Scenario: 關閉狀態列

- **WHEN** 使用者在設定頁將「顯示狀態列」切換為關閉並儲存
- **THEN** `settings.showStatusBar` 設為 `false`
- **AND** 全域狀態列立即從畫面消失

#### Scenario: 預設啟用

- **WHEN** 使用者首次安裝或 settings.json 尚未包含 `show_status_bar` 欄位
- **THEN** 系統預設 `show_status_bar` 為 `true`，狀態列顯示

### Requirement: 設定頁提供終端啟動器選擇

設定頁 SHALL 提供終端啟動器選擇控制項，讓使用者在 shell 與 herdr 之間切換。

#### Scenario: 顯示啟動器選項

- **WHEN** 使用者開啟設定頁的一般設定區塊
- **THEN** 系統顯示終端啟動器選擇控制項，包含 `shell` 與 `herdr` 兩個選項
- **AND** 當前選取值反映 `terminal_launcher` 設定

#### Scenario: 切換至 herdr 啟動器

- **WHEN** 使用者將啟動器切換為 herdr 並儲存
- **THEN** 系統保存 `terminal_launcher` 為 `"herdr"`
- **AND** 後續終端啟動改走 herdr 流程

#### Scenario: 啟動器文案在地化

- **WHEN** 系統顯示終端啟動器相關文案
- **THEN** 文案透過翻譯鍵取得，且 zh-TW 與 en-US 兩份 locale 皆提供對應字串

#### Scenario: 控制項位置與終端機路徑欄位並存

- **WHEN** 設定頁渲染一般設定
- **THEN** 終端啟動器顯示於「預設開啟工具」正上方
- **AND** 終端程式路徑仍位於進階設定且可正常編輯

#### Scenario: 安裝版程序繼承舊 PATH

- **WHEN** SessionHub 安裝版程序的 PATH 尚未包含 herdr，但 `%LOCALAPPDATA%\Programs\Herdr\bin\herdr.exe` 存在
- **THEN** 系統仍將 herdr 顯示為已偵測
- **AND** herdr 服務狀態與後續指令使用該執行檔路徑

## ADDED Requirements

### Requirement: 平台整合管理反映 provider 可用性

平台整合管理 SHALL 依 provider 資料根目錄偵測結果呈現可用或不可用狀態，展開後顯示每個 provider 的工具根目錄與選擇資料夾操作，並在 provider 未偵測時阻止會變更整合安裝狀態的操作。工具根目錄、plugin、hook 與 bridge 路徑 SHALL 使用一致的淡色、無邊框、無背景且不附複製操作的純文字路徑樣式；plugin、hook 與 bridge 路徑僅在有值時顯示。

#### Scenario: provider 未偵測時顯示整合狀態

- **WHEN** 某 provider 資料根目錄未偵測到
- **THEN** 對應整合項目以低對比樣式顯示
- **AND** 安裝、更新及解除安裝操作停用
- **AND** 重新檢查與路徑修正相關操作維持可用

#### Scenario: 顯示 provider 路徑

- **WHEN** 平台整合管理顯示任一 provider
- **THEN** 整合列展開後顯示該 provider 的工具根目錄
- **AND** 根目錄旁顯示選擇資料夾操作
- **AND** 若 provider 提供 plugin、hook 或 bridge 路徑，展開後以帶語意標籤的純文字路徑顯示
- **AND** 若選填路徑沒有值，對應欄位不顯示

#### Scenario: 專案路徑樣式一致

- **WHEN** UI 顯示平台整合、內容檢視器、MCP 或 Agents 流程中的檔案系統路徑
- **THEN** 路徑使用共用的淡色等寬文字樣式
- **AND** 路徑需要檔案或資料夾選擇操作時使用共用的 icon-only 按鈕樣式與無障礙標籤

#### Scenario: provider 恢復可用

- **WHEN** 使用者修正根目錄且重新偵測為存在
- **THEN** 對應整合項目恢復正常樣式與適用的管理操作

### Requirement: 低頻設定收納於進階設定區

設定頁 SHALL 提供預設收折的進階設定區，依序先顯示「顯示已封存 sessions」與「顯示底部狀態列」，再收納終端程式路徑、外部編輯器路徑、Analytics 自動重整間隔、Agents 正本根目錄與允許建立專案 `.sessionhub` 設定資料夾等低頻控制項。

#### Scenario: 初次開啟設定頁

- **WHEN** 使用者進入設定頁
- **THEN** 進階設定區預設為收折
- **AND** 一般設定與儲存操作仍可直接看到

#### Scenario: 展開進階設定

- **WHEN** 使用者啟用進階設定標題列
- **THEN** 系統展開並顯示所有進階控制項
- **AND** 控制項維持原有設定值與操作能力

#### Scenario: 一般設定開關排序

- **WHEN** 使用者開啟一般設定
- **THEN** 五個開關依序為開機時自動啟動、開機時隱藏至系統匣、關閉時最小化至系統匣、Session 結束通知、AI 介入通知

### Requirement: Provider 顯示順序一致

所有同時顯示多個 provider 的 UI SHALL 依 Claude Code、OpenCode、Codex、GitHub Copilot CLI、Antigravity 的固定順序排列，並在部分 provider 未啟用或沒有資料時維持其餘項目的相對順序。

#### Scenario: 顯示完整 provider 清單

- **WHEN** UI 同時顯示五個 provider
- **THEN** 順序為 Claude Code、OpenCode、Codex、GitHub Copilot CLI、Antigravity

#### Scenario: 略過未啟用 provider

- **WHEN** 固定順序中的部分 provider 未啟用或沒有可顯示資料
- **THEN** UI 不顯示缺席項目
- **AND** 其餘 provider 仍依固定順序排列
