## ADDED Requirements

### Requirement: ProjectGroup 以 repoRoot 加分支作為唯一鍵

ProjectGroup 的 `key` SHALL 由正規化後的 `repoRoot`（或 `cwd`）與 `gitBranch` 組合而成，格式為 `<normalizedPath>:<branch>`，確保同一 repo 下不同分支的 worktree 各自分組。

#### Scenario: 同 repo 不同分支產生不同 group

- **WHEN** 掃描到兩個 sessions，`repoRoot` 相同但 `gitBranch` 分別為 `main` 與 `feature/foo`
- **THEN** 系統 SHALL 建立兩個獨立的 ProjectGroup，key 分別為 `<path>:main` 與 `<path>:feature/foo`

#### Scenario: 無 gitBranch 的 sessions 仍能分組

- **WHEN** session 的 `gitBranch` 為 null 或空字串
- **THEN** 系統 SHALL 以 `<normalizedPath>:` 作為該 group 的 key，不崩潰

#### Scenario: 相同 repoRoot 相同分支合併為一個 group

- **WHEN** 多個 sessions 的 `repoRoot` 與 `gitBranch` 皆相同
- **THEN** 系統 SHALL 將它們合併進同一個 ProjectGroup

### Requirement: ProjectGroup.branchLabel 為單一確定分支值

ProjectGroup 的 `branchLabel` SHALL 為該 group 所有 sessions 共同的分支名稱（因已按分支分組，應為同一值），不再使用 `+N` 聚合格式。

#### Scenario: group 內所有 sessions 分支一致

- **WHEN** ProjectGroup 建立完成，其下所有 sessions 的 `gitBranch` 均為 `main`
- **THEN** `branchLabel` SHALL 為 `"main"`

#### Scenario: group 內無 session 有分支資訊

- **WHEN** ProjectGroup 建立完成，其下所有 sessions 的 `gitBranch` 均為 null
- **THEN** `branchLabel` SHALL 為 `null`
