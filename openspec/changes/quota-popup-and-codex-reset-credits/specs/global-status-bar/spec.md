## ADDED Requirements

### Requirement: 狀態列 quota 區域為可點擊控制項

狀態列右側的 quota 區域（含本地彙總 chips 與 remote snapshot chips）SHALL 以可點擊控制項呈現（button 語意），點擊行為為開關狀態列 quota 彈出面板（詳見 `statusbar-quota-popup` capability）；既有 hover tooltip 摘要 SHALL 保留。

#### Scenario: 點擊 quota 區域

- **WHEN** 使用者點擊狀態列任一 quota chip 所在區域
- **THEN** 系統開關 quota 彈出面板
- **AND** chip 的視覺樣式（縮寫、圓環、百分比）維持既有精簡呈現

### Requirement: Codex quota chip tooltip 顯示重置額度摘要

當 Codex 的 quota snapshot 含 `reset_credits` 時，狀態列 Codex chip 的 hover tooltip SHALL 在既有視窗用量行之後追加一行重置額度摘要：可用次數與最近一筆到期時間（本地化格式，與既有 reset 時間格式一致）。

#### Scenario: tooltip 顯示重置額度

- **WHEN** Codex snapshot 的 `reset_credits.available_count` 為 2 且最近一筆額度於 07/21 下午11:59 到期
- **THEN** hover Codex chip 的 tooltip 追加類似「重置額度: 2 次 · 最近到期 07/21 下午11:59」的摘要行

#### Scenario: 無重置額度時 tooltip 不變

- **WHEN** Codex snapshot 的 `reset_credits` 為 null
- **THEN** tooltip 維持既有內容，不出現重置額度行
