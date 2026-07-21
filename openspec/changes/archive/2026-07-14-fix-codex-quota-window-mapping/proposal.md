## Why

Codex 官方近期暫時移除了 5 小時滾動窗口限制。此時 usage API 只回傳一組限制資料且放在 `rate_limit.primary_window`（`secondary_window` 為 `null`），但 SessionHub 目前硬性地把 `primary_window` 當成 5h、`secondary_window` 當成 7d。結果只剩的長期限制被錯標為「5h」顯示，而 7d 窗口反而空白——與使用者實際額度不符，造成誤導。

實測 usage API 回應（free plan）證實：每個 window 物件都帶有 **`limit_window_seconds`** 欄位標示該窗口的精確時長（例如 `300`=5h、`604800`=7d、`2592000`=30d），因此可完全確定性地依此欄位分類，不需以欄位名或 reset 時間推測。

## What Changes

- Codex quota adapter 改為**依 window 物件的 `limit_window_seconds` 欄位**判定窗口類型與標籤，而非直接以 API 欄位名（primary/secondary）對應到固定的 5h/7d。
- 依 `limit_window_seconds` 選定 `window_key` / `label`：約 5 小時 → `five_hour`/`5h`；約 7 天 → `seven_day`/`7d`；其他時長 → 以實際時長動態產生標籤（如 30 天 → `30d`），不再硬套 5h/7d。
- 當 API 只回傳單一窗口（官方移除 5h 後的常態），該窗口依其真實時長正確顯示，5h 窗口不再被填入假資料。
- `secondary_window` 為 `null` 或缺席時不產生窗口；兩組皆無時維持不填 `windows`（前端顯示無資料）。
- 若某窗口缺 `limit_window_seconds`（相容舊回應），退回以 `reset_after_seconds` 推估時長。
- 更新 `provider-quota-monitoring` spec 中已過時的 Codex 資料來源描述（目前 spec 仍寫「本地掃描、windows 為 null」，與現行「遠端 usage API + rolling windows」實作脫節）。

## Capabilities

### New Capabilities
<!-- 無新增能力 -->

### Modified Capabilities
- `provider-quota-monitoring`: 修正 Codex adapter 的窗口對應規格——由「欄位名固定對應 5h/7d」改為「依 reset 時間判定窗口類型」，並更新 Codex 資料來源描述（遠端 usage API + rolling windows，而非純本地掃描）。

## Impact

- 程式碼：`src-tauri/src/quota/codex.rs`（`parse_window` 呼叫處與窗口分類邏輯）。
- 前端無需改動：`window_key` 仍為 `five_hour`/`seven_day`（`quotaWindowLabel.ts` 已支援 `primary`/`secondary`/`5h`/`7d` 等別名）。
- 規格：`openspec/specs/provider-quota-monitoring/spec.md` 的 Codex scenario。
- 無 breaking change；不影響其他 provider。
