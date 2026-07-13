# agents-commands-sync

## ADDED Requirements

### Requirement: 掃描結果包含 command 描述

Commands 掃描 SHALL 為每個 command 抽取描述並隨掃描結果回傳（`CommandEntry` 新增 `description` 欄位，camelCase serde）：描述取自該 command 預覽來源 `.md` 檔的 YAML frontmatter `description` 欄位（沿用既有 frontmatter 解析工具）；檔案不存在、無 frontmatter 或無該欄位時 SHALL 回傳空值而非錯誤，且不影響該 command 其餘掃描結果。

#### Scenario: 有描述的 command

- **WHEN** 某 command 的 `.md` frontmatter 含 `description` 欄位
- **THEN** 掃描結果中該 command 的 `description` 為該欄位值

#### Scenario: 無描述不視為錯誤

- **WHEN** 某 command 的 `.md` 無 frontmatter
- **THEN** 該 command 的 `description` 為空值，其餘欄位照常回傳
