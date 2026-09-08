## ADDED Requirements

### Requirement: 掃描結果包含個別指示檔內容指標
系統 SHALL 在 AGENTS.md / CLAUDE.md 掃描結果中，分別附帶每個已存在檔案的字元數與 token 估算值；來源與目標內容不同時 MUST 保留兩份各自的指標。

#### Scenario: 來源與目標內容一致
- **WHEN** 同一目錄的 AGENTS.md 與 CLAUDE.md 內容一致
- **THEN** 掃描結果分別提供兩個檔案的內容指標
- **AND** UI 可依目前實際選取的檔案取得對應指標

#### Scenario: 來源與目標內容不同
- **WHEN** 同一目錄的 AGENTS.md 與 CLAUDE.md 內容不同
- **THEN** 掃描結果保留 AGENTS.md 與 CLAUDE.md 各自的字元數與 token 估算值

#### Scenario: 儲存後更新指標
- **WHEN** 使用者儲存指示檔並重新整理掃描資料
- **THEN** 掃描結果反映儲存後內容的字元數與 token 估算值
