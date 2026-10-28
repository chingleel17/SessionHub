# Proposal

## Why

大量封存 change 與主規格會讓 Plans & Specs 等待完整 OpenSpec 目錄掃描後才顯示進行中的變更，延遲了最常用文件的操作。需要先呈現進行中變更，再逐層補入較重的資料，且維持既有閱讀與編輯體驗。

## What Changes

- `get_project_specs` 接受可選的 `section`，依 `active`、`specs`、`archived` 掃描指定層；未提供或指定 `all` 時維持完整掃描的既有回傳契約。
- 專案查詢依序載入進行中變更、主規格與封存變更，合併顯示各層資料；重整與 watcher 更新按層處理，`tasks.md` 寫入即時更新對應層進度。
- Explorer 顯示載入階段或錯誤，讓已載入文件可持續操作，背景結果加入時保留使用者的選取及收折狀態；切換專案時隔離畫面狀態。
- 以大量封存 fixture 與前端互動測試驗證分層與完整掃描結果等價及漸進顯示。

## Capabilities

### New Capabilities

- `openspec-layered-loading`: 定義分層掃描、相依載入、分層快取與失敗時的部分資料呈現契約。

### Modified Capabilities

- `plans-specs-explorer-layout`: 明定背景載入新群組與分層重整期間保留 Explorer 操作狀態。

## Impact

影響 `get_project_specs` 的可選 IPC 參數、Rust OpenSpec 掃描與背景執行、`App.tsx` React Query 快取與 watcher 重整、Plans & Specs Explorer、繁中／英文文案及相應測試；不需資料遷移或新套件。
