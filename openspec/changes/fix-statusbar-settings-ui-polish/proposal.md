## Why

近期新增 Antigravity 支援後，UI 出現四個顯示問題：底部狀態列在視窗縮小或 nav 展開時，多個 provider 的 quota chip（縮寫 + 進度條 + 百分比）過寬導致擁擠溢出；設定頁平台整合管理區的 Claude Code 與 Antigravity provider 標籤缺少專屬 badge 顏色（僅 copilot/opencode/codex 有）；一般設定區的「Claude Hook 腳本路徑」欄位與平台整合管理區功能重複；狀態列 quota tooltip 中 Claude/Codex 的時間視窗仍顯示原始英文標籤（`5h`、`7d`），與 Antigravity 的中文顯示（「5 小時」「每週」）不一致。

## What Changes

- 底部狀態列 quota snapshot chip 改為精簡顯示：移除水平進度條，改為「縮寫 + 依用量變色的百分比數字」（可行時加上小型圓環指示），大幅縮減每個 chip 的寬度，避免多 provider 時擁擠。
- 新增 `--color-provider-claude-bg/text` 與 `--color-provider-antigravity-bg/text` 主題變數（dark/light），並補上 `.provider-tag--claude`、`.provider-tag--antigravity` CSS 規則，使設定頁（與其他使用 provider-tag 之處）的 Claude/Antigravity 標籤有品牌色 badge。
- 補上 Antigravity 的顯示名稱：`getProviderLabel`（`src/App.tsx` 與 `src/utils/providerLabel.ts`）缺 antigravity 分支，設定頁整合卡片與 toast 目前顯示小寫原始字串 `antigravity`，改為顯示「Antigravity」（新增 locale key）。
- 移除設定頁一般設定區的「Claude Hook 腳本路徑」欄位（UI），後端 `hook_scripts_path` 設定與預設值邏輯保留，平台整合管理區為唯一的 hook 路徑管理入口。
- 狀態列 quota tooltip 的時間視窗標籤改用 i18n 對照（沿用 `QuotaOverview` 既有的 `windowLabelKey` 對映：`5h`/`five_hour` →「5 小時」、`7d`/`seven_day`/`weekly` →「1 週」等），非時間視窗的標籤（如 Copilot 的 Premium/Chat）維持原字串。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `global-status-bar`: 狀態列 quota 摘要的顯示規格變更——snapshot chip 由進度條改為精簡百分比（依用量門檻變色）；tooltip 時間視窗標籤須本地化顯示。
- `provider-tag`: provider 標籤須涵蓋 claude 與 antigravity，各有專屬 accent 顏色（dark/light 主題皆定義）。
- `app-settings`: 一般設定 UI 不再顯示 Claude Hook 腳本路徑欄位；`hook_scripts_path` 僅由後端預設值與平台整合管理區管理。

## Impact

- `src/components/StatusBar.tsx` — QuotaSnapshotChip 顯示邏輯與 tooltip 標籤本地化
- `src/components/QuotaOverview.tsx` — 抽出 `windowLabelKey` 供共用（或搬移至共用 util）
- `src/components/SettingsView.tsx` — 移除 hookScriptsPath 欄位列
- `src/App.tsx` — `onBrowseDirectory` 型別中移除 `hookScriptsPath`（若不再被 UI 引用）
- `src/App.css` — provider-tag 新規則、狀態列 quota chip 樣式調整
- `src/styles/themes/dark.css`、`light.css` — 新增 claude/antigravity provider 色票
- 後端（Rust）無需變更：`hook_scripts_path` 欄位、預設值與 quota window `label` 均保留
