## ADDED Requirements

### Requirement: Skills / Commands 清單列逐平台狀態晶片

Skills / Commands 清單（非同步 modal）的每一列 SHALL 於名稱與描述之後，以固定順序（claude / codex / opencode / copilot）呈現四個逐平台同步狀態晶片，取代原本單一聚合徽章。每個晶片 SHALL 依該項目對應 target 的 `SyncStatus` 對應為三種視覺狀態之一：

- **已載入**（綠）：`in-sync` / `linked` / `canonical`
- **需同步**（藍）：`differs` / `target-missing` / `link-broken`
- **未安裝**（灰）：`source-missing` / `not-in-source`，或該 target 未被 `prefs.enabledTargets` 啟用、或該項目無此 target 狀態

每個晶片 SHALL 顯示平台名稱並附 `title` tooltip，內容為「平台名：狀態」（例如 `claude：需同步`）。晶片配色 SHALL 柔和一致，不使用過重邊框與全大寫粗體膠囊樣式。

#### Scenario: 四平台晶片依狀態上色

- **WHEN** 使用者檢視 Skills 清單，某 skill 在 claude 已同步、在 codex 有差異、opencode/copilot 未安裝且該兩 target 未啟用
- **THEN** 該列右側依序顯示 claude（綠·已載入）、codex（藍·需同步）、opencode（灰·未安裝）、copilot（灰·未安裝）四個晶片，且各晶片 tooltip 為「平台名：狀態」

#### Scenario: 未啟用的 target 一律呈現未安裝

- **WHEN** 某 target 未被 `prefs.enabledTargets` 啟用
- **THEN** 該 target 的晶片 SHALL 一律以「未安裝」灰色呈現，不反映底層掃描狀態

### Requirement: 全域清單標頭內嵌同步圖例與入口

Skills / Commands 全域清單群組的標頭 SHALL 內嵌一列 sync-note 說明，包含：相容性說明文字、三色狀態圖例（已載入綠點 / 需同步藍點 / 未安裝灰點）、「同步」主按鈕與重新整理按鈕。「同步」按鈕 SHALL 開啟該頁籤的同步 modal；重新整理按鈕 SHALL 重新載入該 scope 的掃描資料。所有文案 SHALL 透過 `t()` 取得，不得於 JSX 硬編中文。

#### Scenario: 全域標頭圖例與同步入口

- **WHEN** 使用者在專案 Agents 分頁展開 Skills 的「全域」群組
- **THEN** 群組標頭顯示 sync-note 說明、三色圖例，以及「同步」與重新整理按鈕；點擊「同步」開啟 Skills 同步 modal
