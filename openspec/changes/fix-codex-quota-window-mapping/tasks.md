## 1. 窗口分類邏輯

- [ ] 1.1 在 `src-tauri/src/quota/codex.rs` 新增純函式 `classify_window(window_seconds: i64) -> (String, String)`，依時長回傳 `(window_key, label)`：約 5h（含容差）→ `("five_hour", "5h")`；約 7d → `("seven_day", "7d")`；其他 → `("codex_window", <格式化如 "30d"/"3h">)`
- [ ] 1.2 新增時長格式化 helper（秒 → `"Nd"` 或 `"Nh"`），供 dynamic 窗口的 label 使用

## 2. 改寫 parse_window 與 fetch_snapshot

- [ ] 2.1 修改 `parse_window`：讀取 `limit_window_seconds`（缺席退回 `reset_after_seconds`，再退回 `reset_at - now`）取得窗口時長，並改由 `classify_window` 決定 `window_key` 與 `label`；移除呼叫端傳入的固定 `key`/`label` 參數
- [ ] 2.2 修改 `fetch_snapshot` 的窗口建構：對 `primary_window`、`secondary_window` 各別解析，null 或缺席者不產生窗口（不再硬綁 primary→5h、secondary→7d）
- [ ] 2.3 確認兩窗口皆為 null / 缺席時 `windows` 為 `None`（維持既有行為）

## 3. 測試

- [ ] 3.1 為 `classify_window` 補 `#[cfg(test)]` 單元測試：18000 秒→five_hour、604800 秒→seven_day、2592000 秒→codex_window+"30d"、以及各邊界容差值
- [ ] 3.2 為 `parse_window` 的 fallback 補測試：只有 `reset_after_seconds`、只有 `reset_at` 兩種情況能正確分類
- [ ] 3.3 執行 `cargo test`（在 `src-tauri/`）確認全數通過

## 4. 驗證

- [ ] 4.1 執行 `cargo build`（`src-tauri/`）確認編譯通過、無新增 warning
- [ ] 4.2 啟動 app 實測：Codex quota 在僅有週/長期限制時顯示於正確窗口（如 30d/7d），不再誤標 5h；5h 窗口無假資料
