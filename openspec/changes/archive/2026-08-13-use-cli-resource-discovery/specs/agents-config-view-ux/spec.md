## ADDED Requirements

### Requirement: 資源狀態圖例反映查詢可信度

Skills、Commands 與 MCP 清單 SHALL 依後端回傳的 `enabledProviders` 順序建立 provider 欄位，不得在前端固定宣告平台數量。Skills/Commands SHALL 將 provider root discovery、支援的 CLI effective 狀態與同步狀態分開呈現；OpenCode Skills/Commands 與 Copilot/Codex MCP 的 tooltip SHALL 顯示 provider、scope、effective path 與 CLI 資料來源。介面 SHALL NOT 再以啟用目標偏好或同步狀態產生「已載入」「未安裝」標籤。重新整理期間 SHALL 保留前次結果並顯示更新中，完成後原位更新。

#### Scenario: OpenCode CLI 可用但檔案需同步
- **WHEN** 某 skill 被 OpenCode CLI 列為 available，但來源與目標檔案不同
- **THEN** 清單同時顯示「可用」與「需同步」兩個不同維度

#### Scenario: 未支援 CLI 的 provider 只顯示可證實資訊
- **WHEN** 使用者檢視 Claude、Codex、Copilot 或 Antigravity Skills／Commands
- **THEN** 清單顯示 provider root discovery 與同步資訊，不顯示 CLI 狀態或「未驗證」badge
- **AND** 不顯示「已載入」或「未安裝」

#### Scenario: Provider 欄位隨設定變更
- **WHEN** 使用者將 enabled providers 從五個改為 OpenCode、Codex 兩個
- **THEN** Skills 與 Commands 清單只顯示 OpenCode、Codex 欄位
- **AND** 不保留停用 provider 的舊晶片或快取狀態

#### Scenario: 專案與全域來源可辨識
- **WHEN** 使用者在專案 Agents 頁檢視同時呈現的專案與全域群組
- **THEN** 每組查詢結果清楚對應 project 或 global scope
- **AND** project 的 effective 合併結果不得被錯標為純全域結果

### Requirement: CLI 檢查時機受頁籤與變更事件控制

Provider root 掃描及支援的 CLI 查詢 SHALL 在使用者首次切入對應 Skills、Commands 或 MCP 頁籤時執行；手動重新整理、enabledProviders 變更、Skills/Commands 同步成功、或 MCP 新增／編輯／刪除／啟停成功後 SHALL 重新查詢對應 scope。系統 SHALL NOT 在僅檢視 AGENTS.md 時啟動資源掃描或 CLI，也 SHALL NOT 背景輪詢。Skills/Commands 結果 SHALL 快取五分鐘，MCP 結果 SHALL 快取三十秒；query key MUST 包含 enabled provider 集合，專案頁 SHALL 分開維護 project 與 global 查詢。

#### Scenario: 切入 Skills 頁籤才檢查 OpenCode
- **WHEN** 使用者進入 Agents 頁但停留在 AGENTS.md 頁籤
- **THEN** 系統不啟動 OpenCode Skills 查詢
- **AND** 使用者首次切到 Skills 頁籤時才執行 project/global 對應查詢

#### Scenario: 同步後重新檢查
- **WHEN** 使用者成功套用 Skills 或 Commands 同步
- **THEN** 系統使對應 kind 與 scope 的 CLI query 失效並重新查詢

#### Scenario: 無背景輪詢
- **WHEN** 使用者停留在 Agents 頁且沒有操作
- **THEN** 系統不按固定間隔啟動任何資源 CLI

### Requirement: 資源閱讀器使用不透明內容表面

Skills 與 Commands 的閱讀器 modal SHALL 使用主題 token 定義的不透明內容表面，modal header 與內容區 SHALL 完整遮蔽底層頁面文字；light 與 dark 主題均須維持可讀對比。長內容 SHALL 在 modal 內容區內捲動，背景頁 SHALL 不可透過內容區顯現，且不得以移除全站 backdrop 效果破壞其他浮層樣式。

#### Scenario: Light 主題閱讀長篇 skill
- **WHEN** 使用者在 light 主題開啟長篇 `SKILL.md`
- **THEN** modal 內容底色不透明，底層技能清單文字不可見
- **AND** 內容可在 modal 圓角範圍內捲動

#### Scenario: Dark 主題閱讀器對比
- **WHEN** 使用者在 dark 主題開啟 skill 或 command 閱讀器
- **THEN** 內容使用 dark theme surface 與文字 token
- **AND** 不出現硬編碼白底或半透明穿透
