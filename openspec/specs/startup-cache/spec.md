## ADDED Requirements

### Requirement: 啟動快取加速初始顯示
系統 SHALL 在每次完成 session 掃描後，將結果序列化儲存至本機快取檔案，以便下次啟動時立即顯示上次的資料，避免白屏等待。

#### Scenario: 快取讀取
- **WHEN** 應用程式啟動
- **THEN** 系統立即讀取 `%APPDATA%\SessionHub\session_cache.json`（若存在）並呈現上次的 session 清單
- **AND** 後台同步執行最新掃描，完成後更新畫面

#### Scenario: 快取寫入
- **WHEN** `get_sessions` 完成任一次掃描（全掃描或增量掃描）
- **THEN** 系統將最新 session 清單寫入 `session_cache.json`
- **AND** 寫入失敗不影響主流程，僅列印警告日誌

#### Scenario: 快取不存在
- **WHEN** 應用程式首次啟動，尚無 `session_cache.json`
- **THEN** 系統顯示掃描中狀態，等待掃描完成後再呈現清單

### Requirement: 掃描進度顯示
系統 SHALL 在掃描進行期間於側邊欄狀態列顯示「掃描中」指示，讓使用者知悉背景作業正在進行。

#### Scenario: 顯示掃描中狀態
- **WHEN** `get_sessions` 正在執行（含初次啟動與即時更新觸發）
- **THEN** 側邊欄狀態文字顯示「掃描中...」
- **WHEN** 掃描完成
- **THEN** 狀態恢復為「即時更新中」或上次同步時間
