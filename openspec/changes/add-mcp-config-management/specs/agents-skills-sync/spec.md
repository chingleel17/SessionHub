# agents-skills-sync

## ADDED Requirements

### Requirement: 掃描結果包含 skill 描述

Skills 掃描 SHALL 為每個 skill 抽取描述並隨掃描結果回傳（`SkillEntry` 新增 `description` 欄位，camelCase serde）：描述取自該 skill 預覽來源 `SKILL.md` 的 YAML frontmatter `description` 欄位（沿用既有 frontmatter 解析工具）；檔案不存在、無 frontmatter 或無該欄位時 SHALL 回傳空值而非錯誤，且不影響該 skill 其餘掃描結果。

#### Scenario: 有描述的 skill

- **WHEN** 某 skill 的 `SKILL.md` frontmatter 含 `description` 欄位
- **THEN** 掃描結果中該 skill 的 `description` 為該欄位值

#### Scenario: 無描述不視為錯誤

- **WHEN** 某 skill 的 `SKILL.md` 無 frontmatter 或無 `description` 欄位
- **THEN** 該 skill 的 `description` 為空值，狀態矩陣與其餘欄位照常回傳
