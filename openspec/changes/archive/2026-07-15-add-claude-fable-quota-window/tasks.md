# Tasks: Claude scoped-model 每週額度視窗（Fable）

## 1. 後端解析

- [x] 1.1 在 `src-tauri/src/quota/claude.rs` 新增 `parse_scoped_weekly_windows(body, existing_keys)`：遍歷 `body["limits"]`，篩 `group == "weekly"` 且 `scope.model.display_name` 非空者，各建 `QuotaWindow`（`window_key = "seven_day_" + display_name.to_lowercase()`、`label = display_name`、`utilization = percent/100`、`resets_at = 項目自身 resets_at`），並對已存在的 window_key 去重
- [x] 1.2 在 `fetch_snapshot()` 頂層 `window_defs` 迴圈之後呼叫該函式，將結果併入 `windows`
- [x] 1.3 新增單元測試：以實際 API 回應的 `limits` 結構（見 scratchpad `claude_usage_fable_fixture.json`）驗證 Fable 視窗（`seven_day_fable`、`utilization == 1.0`、正確 `resets_at`）、未知模型退回、去重、無 scoped 項目時不新增
- [x] 1.4 `cargo test` 全綠

## 2. 前端本地化

- [x] 2.1 `src/utils/quotaWindowLabel.ts` 的 `WINDOW_KEY_MAP` 新增 `seven_day_fable: "quota.window.sevenDayFable"`
- [x] 2.2 `src/locales/zh-TW.ts` 新增 `"quota.window.sevenDayFable": "1 週・Fable"`；`src/locales/en-US.ts` 新增 `"quota.window.sevenDayFable": "1 Week · Fable"`

## 3. 驗證

- [x] 3.1 `npm run build`（tsc + vite）與 `cargo test` 通過
- [x] 3.2 實機驗證：Claude quota 面板與狀態列 tooltip 出現 Fable 視窗，顯示其 utilization 與重置時間
- [x] 3.3 `openspec validate add-claude-fable-quota-window` 通過
