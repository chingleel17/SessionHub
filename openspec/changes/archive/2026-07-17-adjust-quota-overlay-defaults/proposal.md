## Why

`tray-quota-widget` spec 目前記載的 overlay 預設值（不透明度 0.85、`full` 版型、無首次定位邏輯）與使用者期望的開箱體驗不符：新使用者啟用 overlay 後畫面偏大、偏不透明，且首次出現位置不可預期（落在系統預設位置，非螢幕右下角）。已依使用者要求調整為更精簡、更透明的預設樣式，並補上首次定位邏輯，需同步更新 spec 反映新行為。

## What Changes

- Overlay 預設版型由 `full`（進度條列表）改為 `compact`（圓環一列 chips）。
- Overlay 預設不透明度由 `0.85` 改為 `0.3`（背景更透明）。
- Overlay 首次啟用（尚無 `tauri-plugin-window-state` 已存位置紀錄）時，改為自動定位到主螢幕右下角（保留 16px 邊距），而非落在系統預設位置；已拖曳過的使用者不受影響，仍還原其自訂位置。

## Capabilities

### Modified Capabilities
- `tray-quota-widget`: Overlay 的預設版型、預設不透明度與首次顯示時的預設定位行為變更。

## Impact

- 後端：`src-tauri/src/types.rs`（`default_quota_overlay_opacity`、`OverlayStyle` 預設值）、`src-tauri/src/settings.rs`、`src-tauri/src/lib.rs`（新增 `has_saved_window_state`、`position_window_bottom_right`）。
- 前端：`src/App.tsx`、`src/components/SettingsView.tsx` 的預設值 fallback 同步調整。
- 僅影響全新安裝或尚未儲存過 overlay 設定／位置的使用者；既有使用者已保存的 `settings.json` 與 `.window-state.json` 不受影響。
