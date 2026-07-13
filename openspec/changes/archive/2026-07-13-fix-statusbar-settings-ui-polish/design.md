## Context

四個 UI 問題皆為前端顯示層修正，不涉及後端資料變更：

1. `StatusBar.tsx` 的 `QuotaSnapshotChip` / `QuotaChip` 目前渲染「縮寫 + 水平進度條 + 百分比」，多 provider（CX/GH/AG/CC）同時顯示時總寬度過大，視窗縮小或 nav 展開時擁擠溢出。
2. `App.css` 只有 `.provider-tag--copilot/opencode/codex` 三條規則；`provider-tag--claude`、`provider-tag--antigravity` 無對應 CSS，theme 檔也沒有 `--color-provider-claude-*`、`--color-provider-antigravity-*` 變數，導致設定頁整合卡片的標籤無底色。
3. `SettingsView.tsx` 一般設定區有「Claude Hook 腳本路徑」欄位（`hookScriptsPath`），與平台整合管理區的路徑編輯功能重複。
4. 狀態列 tooltip 直接顯示後端 `QuotaWindow.label` 原始字串：Claude/Codex 在 Rust 端寫死 `"5h"`/`"7d"`（`quota/claude.rs:322`、`quota/codex.rs:230`），而 Antigravity 在 Rust 端寫死中文（`quota/antigravity.rs:218`）。`QuotaOverview.tsx` 已有 `windowLabelKey()` 將 windowKey 對映到 i18n key，但 StatusBar tooltip 未使用。

## Goals / Non-Goals

**Goals**
- 狀態列 quota chip 寬度大幅縮減，多 provider 同時顯示不擁擠。
- claude / antigravity provider 標籤取得與其他 provider 一致的品牌色 badge（dark/light 皆定義）。
- 一般設定移除重複的 hook 路徑欄位。
- 狀態列 tooltip 時間視窗標籤全面中文化（跟隨 i18n locale）。

**Non-Goals**
- 不修改 Rust 端 `QuotaWindow.label` 的產生邏輯（維持 `5h`/`7d` 原始值，本地化在前端處理）。
- 不移除 `AppSettings.hook_scripts_path` 欄位或其後端預設值 / fallback 邏輯。
- 不改動 Dashboard `QuotaOverview` 的視覺呈現（已使用本地化標籤與完整進度條，空間充足）。

## Decisions

### D1：狀態列 quota chip 改為「縮寫 + SVG 圓環 + 變色百分比」

- 移除 `QuotaSnapshotChip` 與 `QuotaChip` 內的水平進度條（`global-status-bar-quota-bar`）。
- 以小型 SVG 圓環（約 12–14px，`stroke-dasharray` 依 pct 繪製，stroke 顏色沿用 `quotaBarColor()` 門檻：≥90% danger、≥70% warning、其餘 ok）+ 百分比數字（文字同色）取代。
- 百分比數字為主要資訊，圓環為輔助視覺；若百分比不可得（無 limit / 無 window），僅顯示縮寫（現行為）。
- `QuotaChip`（本地彙總）有 cost 時保留 `$x.xx` 文字，有 limit 時同樣改用圓環 + 百分比。
- **替代方案**：僅顯示變色百分比數字（無圓環）— 使用者已接受作為 fallback；SVG 圓環實作成本低（單一 `<svg><circle>`），採圓環方案。

### D2：provider 色票以既有品牌色為基準

- `StatusBar.tsx` 已定義品牌色：claude `#D97757`、antigravity `#4285F4`。theme 變數以此為基準產生 bg/text 對比組合（dark：暗底亮字；light：淺底深字），並確保 contrast ratio ≥ 4.5:1（沿用 provider-tag 既有視覺規格）。
- 新增 `.provider-tag--claude`、`.provider-tag--antigravity` 至 `App.css`，模式與既有三條規則一致。

### D3：一般設定僅移除 UI 欄位，資料層不動

- 刪除 `SettingsView.tsx` 中 hookScriptsPath 的 label/input/瀏覽按鈕區塊，並自 `onBrowseDirectory` 的 field union type 移除 `"hookScriptsPath"`。
- `App.tsx` 中 settingsForm 仍保留 `hookScriptsPath` 值的讀取與回寫（儲存設定時原值透傳），確保既有自訂路徑不因 UI 移除而遺失。
- 平台整合管理區的編輯（鉛筆）按鈕為唯一的 hook 路徑調整入口。
- **替代方案**：連同後端欄位一併移除 — 風險高（`provider/claude.rs` 多處依賴），且使用者僅要求移除重複 UI，不採。

### D4：tooltip 標籤本地化 — 前端共用 helper，不改 Rust

- 將 `QuotaOverview.tsx` 的 `windowLabelKey()` 抽至共用模組（如 `src/utils/quotaWindowLabel.ts`），StatusBar tooltip 改用。
- 注意 windowKey 衝突：codex 用 `primary`/`secondary` 代表 5h/7d，copilot 也用 `primary`/`secondary` 但語意是 Premium/Chat（月配額，非時間視窗）。共用 helper 簽名採 `localizedWindowLabel(provider, windowKey, rawLabel, t)`：
  - `provider === "copilot"` → 直接回傳 rawLabel（Premium/Chat）。
  - 其他 provider → 依 windowKey 查 i18n key（`5h`/`five_hour` → `quota.window.fiveHour`；`7d`/`seven_day`/`weekly`/`secondary` → `quota.window.sevenDay`；含 sonnet/opus 變體），查無對映時 fallback 回傳 rawLabel（不再硬 fallback 到 fiveHour）。
- Antigravity 的 Rust 端中文 label 因 windowKey（`5h`/`weekly`）可對映，統一走前端 i18n，顯示結果不變且未來可跟隨 locale 切換。
- **替代方案**：改 Rust 端 label 為中文 — 違反「文字經 t()、不硬編碼中文」慣例（antigravity.rs 現況屬例外），且無法跟隨語言切換，不採。

## Risks / Trade-offs

- [圓環 12px 在低 DPI 下可能不夠清晰] → 百分比數字為主資訊，圓環僅輔助；必要時 CSS 可整體隱藏圓環退回純數字。
- [copilot 以 rawLabel 顯示英文 Premium/Chat] → 該類別非時間視窗，無自然中文對應，維持原字串屬合理範疇；如需翻譯可後續補 i18n key。
- [QuotaOverview 對 copilot 的 primary/secondary 也誤映為 5 小時/1 週] → 既有問題，本次抽共用 helper 時一併讓 QuotaOverview 改用 provider-aware 版本修正。

## Migration Plan

純前端顯示變更，無資料遷移；`hookScriptsPath` 既有設定值原樣保留。

## Open Questions

（無）
