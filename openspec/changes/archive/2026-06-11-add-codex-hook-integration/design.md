## Context

SessionHub 目前的 provider integration 已能管理 Copilot 與 OpenCode，但 Codex 仍停留在純 session 掃描與檔案 watcher 層，尚未接到 provider bridge。這造成三個 provider 的即時刷新策略不一致：Copilot 可透過 hook 送 targeted 或 activity 事件，OpenCode 可透過 plugin 寫 bridge，Codex 則只能等 `~/.codex/sessions` 的檔案變化再觸發 refresh。

另一個問題是 provider integration 產物目前偏向由各模組內嵌字串直接組裝。當只有兩個 provider 時還可接受，但要加入第三個 provider 後，若 hook / plugin 腳本持續混在同一批模板邏輯中，之後調整事件映射、版本號或平台差異時會更難維護。

外部能力方面，Codex 官方文件支援在 `~/.codex/hooks.json` 或 `~/.codex/config.toml` 內註冊 lifecycle hooks。本次將優先採用 `hooks.json`，因為它較容易由 SessionHub 完整接管、偵測與更新，不需要解析或 merge 使用者既有 TOML 設定層。

## Goals / Non-Goals

**Goals:**

- 讓 SessionHub 可安裝、更新、重新檢查 Codex hook integration，並將事件寫入既有 provider bridge 機制。
- 將 Copilot、OpenCode、Codex 的 integration 產物拆成 provider-specific 腳本或模板檔，避免三者維護邏輯混在同一處。
- 讓設定頁 provider integration 區塊可顯示 Codex 狀態與路徑資訊。
- 讓 watcher 對 Codex 採取與其他 provider 一致的策略：bridge 可用時優先吃 bridge，否則使用 filesystem fallback。

**Non-Goals:**

- 本次不重新設計 bridge record schema；沿用既有 `ProviderBridgeRecord` 欄位。
- 本次不要求 Codex 達到與 Copilot 完全相同的 targeted refresh 或 activity hint 精度；首版以可靠 refresh 為主。
- 本次不處理使用者自訂 Codex hook 的完整 merge 編輯器介面；僅要求 SessionHub 可安全寫入並辨識自己管理的條目。
- 本次不變更 session 掃描模型或 Codex JSONL 解析方式。

## Decisions

### 1. Codex integration 採用 `~/.codex/hooks.json` 作為受管理入口

SessionHub 將為 Codex 寫入受管理的 `hooks.json` 內容，而不是直接修改 `config.toml`。受管理 metadata 需記錄 provider、bridge path、integration version 與 SessionHub 管理標記，供狀態檢查使用。

原因：`hooks.json` 對 hook lifecycle 定義更直接，也比較適合由應用程式生成固定內容與做版本檢查。若改寫 `config.toml`，需要處理更多既有設定與使用者手動編輯的合併情境。

替代方案：

- 使用 `config.toml` inline hooks。缺點是需要更複雜的 TOML merge 與格式保留策略。
- 不做安裝流程，只保留手動設定說明。缺點是無法與現有 provider integration UX 對齊。

### 2. Provider integration 產物改為 provider-specific 腳本 / 模板檔分離管理

Copilot、OpenCode、Codex 各自維護自己的模板來源與必要腳本，避免把三家的 hook / plugin 內容寫在單一大型字串組裝流程中。共同邏輯只保留在 metadata 驗證、bridge diagnostics、狀態建構與檔案寫入 helper。

原因：使用者已明確要求拆開管理，且這樣能降低後續 provider 擴充時的耦合。對 Rust module 結構而言，也更符合 `provider/<provider>.rs` 各自管理該 provider integration 的邊界。

替代方案：

- 保留單一 integration renderer，內部分支三種 provider。缺點是 provider 專屬事件映射會持續膨脹。
- 完全抽成共用 DSL。缺點是目前只有三個 provider，抽象成本過高。

### 3. Codex bridge 先以完整 refresh 為主，不先做 targeted refresh

Codex hook 首版應至少發出可驅動 `codex-sessions-updated` 的標準 bridge record。可使用 `SessionStart`、`PostToolUse`、`Stop` 或等效事件作為 refresh trigger，但後端先不新增 Codex 專屬 targeted payload。

原因：Codex 已有穩定的 session 檔案掃描器，bridge 只要能更快觸發重新查詢就足夠改善 UX。若直接追求 targeted refresh，需要額外驗證 Codex hook 提供的 `cwd`、`session_id` 與 lifecycle 時點是否足夠穩定。

替代方案：

- 一開始就做 Codex targeted refresh。缺點是風險高，且沒有現成需求要求必須做到與 Copilot 同級。
- 僅安裝 hook 但後端仍忽略 bridge。缺點是沒有實際產品價值。

### 4. watcher 啟動策略改為三個 provider 都先檢查 bridge integration 狀態

`restart_session_watcher_internal` 應針對 Copilot、OpenCode、Codex 分別判斷 integration 是否為 `installed`。若已安裝，則該 provider 的主要刷新路徑改由 provider bridge watcher 驅動；若未安裝或狀態異常，才啟用其 filesystem watcher。

原因：目前 Codex 已有 filesystem watcher，但沒有 bridge 狀態切換邏輯。補齊後可與既有設計一致，也能避免 bridge 與 filesystem 同時大量重複刷新。

替代方案：

- Codex 永遠同時開 bridge 與 filesystem watcher。缺點是容易重複刷新且增加事件噪音。

### 5. 設定頁 integration 管理文案與資料模型擴充為三 provider

`provider_integrations` 回傳資料需包含 Codex，並在設定頁與相關 toast / action 流程一體適用。文案不能再只描述「Copilot hook 與 OpenCode plugin」，需改成包含 Codex hook 的平台整合管理。

原因：若後端支援 Codex integration 但 UI 不顯示，使用者就無法安裝、重新檢查或開啟目標設定檔。

替代方案：

- 先只做後端，不在設定頁顯示。缺點是功能可用性低，且違反既有 provider integration 產品模式。

## Risks / Trade-offs

- [Codex 使用者已存在自訂 hooks.json] -> 需要定義受管理區塊或 merge 規則，避免直接覆蓋非 SessionHub 條目。
- [Codex lifecycle event 與 session 真正落盤時間不同步] -> 首版以 refresh trigger 為主，保留 filesystem fallback 補強一致性。
- [新增第三個 integration 後，前後端 provider 列表容易漏改] -> 將 provider 順序與支援清單集中，避免各處硬編碼。
- [腳本拆檔後檔案數變多] -> 以 provider 模組邊界管理，降低單檔複雜度，整體可維護性仍較高。
- [bridge 與 fallback watcher 狀態切換錯誤] -> 以 integration status 為唯一判斷來源，並補測試驗證 watcher 啟停矩陣。

## Migration Plan

1. 新增 Codex integration 偵測 / 安裝流程與 provider-specific 模板來源。
2. 擴充 provider integration 狀態聚合與設定頁顯示，讓 Codex 出現在 UI 中。
3. 調整 watcher 啟動邏輯，讓 Codex bridge installed 時改走 provider bridge watcher。
4. 補上 bridge refresh event 名稱與相關測試，避免 Codex bridge 寫入後沒有前端刷新事件。
5. 若使用者已有自訂 Codex hooks 設定，保留非受管理條目，只更新 SessionHub 管理的條目。

## Open Questions

- Codex hook metadata 要放在 hooks.json 的哪種可穩定辨識位置，才能兼顧管理與相容性？
- Codex 是否需要在首版就送出 `cwd` 與 `sessionId` 以支援未來 targeted refresh，還是先允許部分事件只有 provider / timestamp？
- Copilot 與 OpenCode 的現有 integration 產物要拆成獨立模板檔，還是先維持獨立 renderer function 即可視為「分開管理」？
