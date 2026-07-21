## ADDED Requirements

### Requirement: 前端 lint 指令

專案 SHALL 提供可由 Bun 執行的 `lint` script，用以檢查前端 TypeScript 與 React 原始碼，且其執行結果不依賴本機 IDE 設定。

#### Scenario: 維護者於本機執行 lint
- **WHEN** 維護者執行 `bun run lint`
- **THEN** 系統執行設定的前端 lint 工具並以非零結束碼回報違規

### Requirement: CI 前端品質門檻

主分支與指向主分支的 Pull Request SHALL 執行前端 lint，且 lint 失敗時 CI 必須失敗。

#### Scenario: Pull Request 有 lint 違規
- **WHEN** Pull Request 的前端 lint 回報違規
- **THEN** CI 顯示失敗的 lint check
- **AND** 該 check 不得設定為 `continue-on-error`

#### Scenario: Pull Request 通過 lint 與建置
- **WHEN** Pull Request 的 lint 與既有前端建置都成功
- **THEN** CI 將兩項結果分別顯示為成功
