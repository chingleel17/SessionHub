## Why

`Agents.dc.html` 設計稿將 Skills / Commands 清單的每一列改為顯示四個平台（claude / codex / opencode / copilot）的個別同步狀態晶片（chips），取代目前僅顯示單一「已同步 / 需同步 N」聚合徽章的呈現。使用者可在不開啟同步 modal 的情況下，一眼看出每個 skill/command 在四個 agent 平台上的實際狀態，並在全域清單直接觸發整批同步。此為既有 `agents-config-view-ux` 能力的呈現層精修，後端已提供逐平台 `TargetStatus`，無需新增資料來源。

## What Changes

- Skills / Commands 清單每一列的右側，改以四個固定順序（claude / codex / opencode / copilot）的狀態晶片呈現逐平台同步狀態，取代原本單一聚合徽章。
- 每個晶片依該 target 的 `SyncStatus` 對應三種視覺狀態：**已載入**（綠，`in-sync` / `linked` / `canonical`）、**需同步**（藍，`differs` / `target-missing` / `link-broken`）、**未安裝**（灰，`source-missing` / `not-in-source` / 無此 target）；晶片附 `title` tooltip 顯示「平台名：狀態」。
- 清單區塊標頭（全域群組）新增內嵌 sync-note 說明列，含狀態圖例（已載入 / 需同步 / 未安裝三色點）、「同步」主按鈕與重新整理按鈕，讓全域整批同步入口更明確。
- 晶片狀態只讀取 `prefs.enabledTargets` 中已啟用的 target 為「有意義」狀態；未啟用的 target 一律以「未安裝」灰色晶片呈現，與設計稿的固定四欄一致。
- 純前端呈現變更：不改動後端掃描邏輯、Tauri commands 或型別，不新增相依套件。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `agents-config-view-ux`: Skills / Commands 清單列的狀態呈現由「單一聚合徽章」改為「四平台逐一狀態晶片 + 圖例」；全域清單標頭新增 sync-note 圖例與同步入口。

## Impact

- 前端：`src/components/AgentsConfigView.tsx`（`renderListGroup` 內列的呈現、chips 建構函式、全域 sync-note 標頭）；`src/App.css`（chips、圖例、sync-note 樣式）；`src/locales/zh-TW.ts`、`src/locales/en-US.ts`（已載入 / 需同步 / 未安裝、圖例與 sync-note 文案鍵）。
- 後端：無變更（`SkillEntry.targets` / `CommandEntry.targets` 的 `TargetStatus[]` 已提供所需資料）。
- 相依：無新增。
- 風險：低；純呈現層調整，不影響同步行為或檔案寫入。
