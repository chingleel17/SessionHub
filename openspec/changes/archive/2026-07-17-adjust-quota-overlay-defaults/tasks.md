## 1. 實作（已完成）

- [x] 1.1 `types.rs`：`default_quota_overlay_opacity()` 改為 `0.3`，`OverlayStyle` 預設改為 `Compact`
- [x] 1.2 `settings.rs`、`lib.rs`：`AppSettings::default()` 與啟動失敗 fallback 改用 `default_quota_overlay_opacity()`，不再硬編碼 `0.85`
- [x] 1.3 `lib.rs`：新增 `has_saved_window_state()` 讀取 `.window-state.json` 判斷是否已有該 overlay label 的紀錄
- [x] 1.4 `lib.rs`：新增 `position_window_bottom_right()`，無已存位置時定位到主螢幕右下角（16px 邊距）
- [x] 1.5 `App.tsx`、`SettingsView.tsx`：所有 `quotaOverlayOpacity ?? 0.85` 與 `quotaOverlayStyle ?? "full"` fallback 同步改為 `0.3` / `"compact"`

## 2. 驗證

- [x] 2.1 `cargo check --lib` 通過
- [x] 2.2 `bun run build` 通過
- [x] 2.3 手動驗證：刪除 `.window-state.json` 中 overlay 紀錄後重新啟用 overlay，確認出現在螢幕右下角、精簡版型、透明度明顯偏高
