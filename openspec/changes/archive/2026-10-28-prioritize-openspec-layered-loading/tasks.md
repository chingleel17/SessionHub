# Tasks

## 1. 掃描契約與後端測試

- [x] 1.1 讓 `get_project_specs` 支援可選區段、保留完整掃描，並以 `cargo test scan_openspec_layers_preserve_full_results_with_large_archive` 驗證大型 fixture 的分層合併等價。
- [x] 1.2 將同步掃描移至 blocking pool，並以完整 `cargo test` 驗證後端命令與既有掃描測試。

## 2. 漸進畫面與互動測試

- [x] 2.1 加入專案／區段 query、分層重整與 tasks 樂觀更新，以 `bun run build` 驗證型別及串接。
- [x] 2.2 顯示背景階段與錯誤，保留 Explorer 選取與收折並隔離專案，以 `bun test tests/PlansSpecsView.test.tsx tests/buildTree.test.ts` 驗證互動與 fixture。
- [x] 2.3 確認前一層失敗時不持續顯示未啟動層載入中，並以相關前端測試及 `bun run build` 驗證。

## 3. 整合品質檢查

- [x] 3.1 執行完整 `bun test`、`node --test tests/hooks/*.test.cjs`、`bun run lint`、`bun run build`、`cargo test`、`cargo fmt -- --check`、`git diff --check` 與 `openspec validate prioritize-openspec-layered-loading --strict`，確認全部成功。
