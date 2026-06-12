## ADDED Requirements

### Requirement: OpenCode 最新 session 的統計來源需與新版 storage 相容

系統 SHALL 確保最新 OpenCode session 在使用新版 storage 模型時，仍能正確定位其統計資料來源或提供可接受的 fallback。

#### Scenario: 最新 session 可讀取基本統計
- **WHEN** 使用者打開一個只存在於 `opencode.db` 的最新 OpenCode session
- **THEN** 系統可正確定位其對應的 message / stats 資料來源
- **OR** 在尚未有完整統計資料時回傳合理 fallback，而非直接失敗

#### Scenario: 單一 session stats 失敗不影響列表
- **WHEN** 某個 OpenCode session 的 stats 解析失敗
- **THEN** 該失敗不得阻止 session 本身顯示在列表中
