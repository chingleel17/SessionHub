## Context

問題背景與根因查證見 proposal.md - Why。此處僅補充影響設計選擇的現況約束。

- tray panel 由 `toggle_tray_panel`（`lib.rs:188`）動態建立，一般開關走 `show()` / `hide()`，webview 不卸載。但這並非唯一路徑：`EmbeddedTrayPanelApp:64` 的 `onOpenSettings` 呼叫 `getCurrentWindow().close()`，webview 會被銷毀，下次點擊 tray 才重建。因此「webview 剛建立、listener 尚未註冊」的競態不只發生在首次開啟，而是每次由 panel 進入設定後都會再現。
- panel 顯示時呼叫 `set_focus()`（`lib.rs:198`、`lib.rs:237`）；隱藏則由 `on_window_event` 的 `WindowEvent::Focused(false)` 觸發（`lib.rs:418-422`）。此為原生視窗層級的 focus 訊號，與 panel 的顯示／隱藏一一對應。
- `lib.rs:41-62` 三個 emit 函式各自重複「廣播 + 定向送 overlay」的樣板；`QUOTA_OVERLAY_LABEL` 與 `TRAY_PANEL_LABEL` 是目前僅有的兩個動態 webview label。
- `get_quota_snapshots`（`commands/quota.rs:119`）僅回傳快取全量，不做 enabled 過濾；過濾責任在前端。overlay 由 `EmbeddedQuotaOverlayApp` 取 `settings.quotaEnabledProviders` 傳入，panel 則無。

## Goals / Non-Goals

**Goals:**

- 事件送達層面消除 overlay 與 panel 的差別待遇，且此保證不隨日後新增動態視窗而失效。
- panel 具備一層與其顯示週期對齊的兜底重取，涵蓋事件仍然遺漏的殘餘風險。
- panel 的 provider 過濾與 overlay 採同一資料來源與判準。

**Non-Goals:**

- 不改變 quota 刷新的排程與頻率（`QUOTA_REFRESH_INTERVAL_SECS` 不動）。
- 不將 enabled 過濾下推至 Rust 端。`get_quota_snapshots` 目前為多個呼叫端共用的無狀態讀取，改成依設定過濾會擴大影響面；本次維持前端過濾，與 overlay 一致。
- 不重構 `TrayQuotaPanel` 的版面與排序邏輯。

## Decisions

### 決策一：抽出共用的動態 webview 定向送出 helper

新增一個以 label 清單為基礎的內部 helper，對 `QUOTA_OVERLAY_LABEL` 與 `TRAY_PANEL_LABEL` 逐一取得 webview 並送出事件；三個 emit 函式（`emit_quota_snapshots_updated`、`emit_overlay_settings_changed`、`emit_intervention_list_changed`）改為呼叫它。既有的 AppHandle 廣播保留不動——主視窗仍依賴它。

*為何不只在三處各補一行 `TRAY_PANEL_LABEL`：* 那會讓同一個易漏的樣板從三份變成六份。缺陷本質是「動態 webview 清單」這件事沒有單一來源，補行不消除它；helper 則使新增動態視窗時只需改一處。

*為何不取消定向送出、改為只靠廣播：* `lib.rs:48` 的註解記載廣播對動態 webview 不可靠，是先前實測結論。本次不推翻該結論，亦不在此變更中重新驗證。

*事件重複送達的處理：* 廣播與定向送出可能使同一 webview 收到兩次。現有 listener 皆為 idempotent（invalidate / setQueryData），overlay 已在此模式下運作，panel 沿用同樣語意，無需去重。

### 決策二：panel 以 Tauri 原生 `onFocusChanged` 作為兜底，不用常駐輪詢

`EmbeddedTrayPanelApp` 以 `getCurrentWindow().onFocusChanged(...)`（`@tauri-apps/api/window`）監聽原生 focus 事件，在 focused 為 true 時主動重取 snapshot query，並將該 query 的 `staleTime` 設為 `0`，確保重取不被 freshness 抑制。

*為何不用 TanStack Query 的 `refetchOnWindowFocus`：* 該選項由 query 的 `focusManager` 驅動，底層監聽 webview 內的 DOM `visibilitychange`。而本專案 panel 的隱藏是 Rust 端呼叫原生 `window.hide()`，在 Windows / WebView2 上原生隱藏與再顯示不保證翻轉 `document.visibilityState`，DOM 事件可能根本不觸發。兜底機制不能建立在無法確認會發生的訊號上。

*為何 `onFocusChanged` 可靠：* 它接收的正是 Rust 端 `WindowEvent::Focused` 的同一個原生事件——也就是 `lib.rs:418` 用來隱藏 panel 的那個訊號。既有的失焦自動隱藏已在實機運作，即證明該訊號確實送達；顯示時的 `set_focus()` 對應 focused=true，與隱藏共用同一條事件通道。

*為何不從 Rust 端另外 emit 顯示事件：* 原生 focus 事件已足夠且無需新增事件名稱與後端改動；若日後 panel 改為不搶焦點顯示，才需要改用專用的顯示事件。

*為何不照抄 overlay 的 `refetchInterval: 15_000`：* overlay 常駐可見，輪詢有其正當性；panel 絕大多數時間為 hidden，常駐輪詢會在使用者看不到的情況下持續呼叫 IPC 與讀取快取，屬純浪費。

*為何仍需要兜底：* 決策一已修正已知的遺漏路徑，但如 Context 所述，webview 重建後 listener 尚未註冊的競態仍會反覆出現（每次由 panel 進入設定後即再現一次）。focus 重取讓「使用者看到的當下」必定是新資料。

### 決策三：enabled 過濾比照 overlay 於前端進行

`EmbeddedTrayPanelApp` 讀取 settings 取得 `quotaEnabledProviders` 並傳入 `TrayQuotaPanel`；`TrayQuotaPanel` 新增 `enabledProviders` prop，於既有 `sortSnapshots` 之前過濾。預設值與 overlay 同為全部 provider，避免設定尚未載入時清單瞬間清空。

settings query 必須有明確的更新來源，否則會無限期供應過期的 enabled 清單。採兩條並行：一是監聽 `quota-overlay-settings-changed`——決策一使該事件自本次起也送達 panel，panel 收到後直接以 payload 更新快取（與 overlay 同語意）；二是納入決策二的 focus 重取範圍，涵蓋 listener 未及註冊的競態。

不比照 overlay 的 1 秒輪詢：overlay 該輪詢是為了透明視窗樣式即時套用，panel 無此需求，事件加 focus 重取已足夠。

## Risks / Trade-offs

- **廣播 + 定向造成事件重複，若日後有非 idempotent 的 listener 會出錯** → 於 helper 的 doc comment 明確記載此語意，要求 listener 必須 idempotent。
- **兜底依賴 focus 與顯示的耦合關係，若日後 panel 改為不搶焦點顯示，兜底會失效** → 該耦合已寫入 design 與 spec 場景；若未來變更顯示方式，spec 的「panel 顯示時取得最新快照」場景會迫使一併調整，屆時改由 Rust 端 emit 專用顯示事件。
- **新增 settings 讀取使 panel 開啟時多一次 IPC** → 單次成本極低，且以 query 快取避免每次開啟都重打。
- **enabled 過濾上線後，先前誤顯示的 provider 會消失** → 此即回歸既有 spec 定義的正確行為，非退步；但屬使用者可見變化，需在變更說明中提及。

## Migration Plan

無資料遷移。純程式碼變更，不動事件名稱、payload 結構與設定欄位，舊設定檔無需處理。回滾即還原程式碼。
