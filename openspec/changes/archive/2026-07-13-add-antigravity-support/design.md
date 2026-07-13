## Context

SessionHub 已有成熟的 provider 抽象：`sessions/mod.rs` 依 `enabled_providers` 逐一掃描各工具目錄，每個 provider 一個檔案（claude/codex/copilot/opencode）；`quota/mod.rs` 以 `QuotaAdapter` trait + `QuotaManager` 聚合各 provider 的額度快照。前端以 `providerLabel.ts` 集中 provider 顯示名稱、settings 控制開關。

Antigravity 的本機資料經實地探勘與實測驗證（記錄於 `memory/antigravity-integration.md`）：

- **Session**：三形態各有 `brain/<uuid>/` 目錄；可讀的 `*.metadata.json`（`summary`/`updatedAt`）與 `.system_generated/logs/transcript.jsonl` 提供列表所需的一切。`conversations/<id>.db`（SQLite + protobuf blob）為 IDE runtime 用，本 change 不碰。
- **Hook**：`~/.gemini/config/hooks.json`（全域）與 `<repo>/.agents/hooks.json`（專案），schema 近似 Claude Code。
- **Quota**：Antigravity IDE／agy 執行時會跑 `language_server.exe`，在 `127.0.0.1` 動態 port 提供 Connect RPC。`RetrieveUserQuotaSummary` 回傳按模型群組分的 5 小時／每週 bucket，含 `remainingFraction` 與 `resetTime`。已用 `get_quota.js`（現於 scratchpad）實測取得真實數字。

## Goals / Non-Goals

**Goals:**

- 將 Antigravity 納為第五個 session provider，沿用既有掃描／快取／git 補強流程，僅列表 metadata 層級。
- 提供全域與專案 hook 的 CRUD，複用 Claude hook UI 模式。
- 以本機 Language Server 讀取即時 quota，映射為既有 `QuotaWindow`，狀態列顯示 Gemini、Dashboard 顯示兩群組。
- 不可用情境（IDE 未開、目錄缺失、metadata 損毀）優雅降級，絕不拖垮其他 provider。

**Non-Goals:**

- 不解析 protobuf `conversations/<id>.db` / `.pb` 的對話內文（訊息、tool call 明細）。
- 不實作雲端 quota fallback（IDE 關閉時 quota 顯示不可用即可）；雲端管道保留為未來 change。
- quota 的進程／port 探測初期僅 Windows 實作；非 Windows 平台降級為不可用。
- 不追蹤 AI Credits（`availablePromptCredits` 等）明細，初期只呈現 5h／週視窗。

## Decisions

### D1：以 transcript 存在性界定 session；標題/workspace 走 summaries.pb 查表（實測修正）

實測（38 個含 transcript 的 brain 目錄，數目與 `conversations/*.db` 吻合）確立：**brain 下含 `.system_generated/logs/transcript.jsonl` 的目錄才是一個 session**；僅有圖片/產出物的目錄（另 16 個）不計。

原假設「逐目錄讀 `*.metadata.json` 當標題」經實測否決——那些 `*.metadata.json` 是產出檔案的附屬（如 `implementation_plan.md.metadata.json`），且部分 conversation 完全沒有；transcript 本身也無 title/workspace 欄位。**正確來源是 `agyhub_summaries_proto.pb`**：單一 protobuf 檔含全部 conversation 的 `UUID → 標題 → file:/// workspace`。做輕量 protobuf 解析（欄位規律已由實測樣本確認，無需完整 schema）取這兩者，查無標題則 fallback 為 transcript 首則 `USER_REQUEST` 前段，查無 workspace 則留空歸未分類。**替代方案**：解析 `conversations/<id>.db` 的 protobuf blob 取完整對話——否決，成本高於價值且與「僅列表 metadata」衝突。

CLI 形態（`antigravity-cli`）無 summaries.pb，改以 transcript 首則 request 當標題、workspace 查無留空——與 IDE 同 provider、兩套解析函式。

### D2：三個 brain 目錄合併為單一 `antigravity` provider

三形態（antigravity / antigravity-cli / antigravity-ide）目錄結構一致，對使用者是同一個工具。合併為單一 provider key `"antigravity"`，掃描時走訪三個 root。**替代方案**：拆成三個 provider——否決，對使用者造成不必要的分裂，且 CLI 目前多為空。

### D3：Quota 走本機 Language Server RPC，不碰 OAuth

實測本機 RPC 完全可行且認證由 LS 代管，省去 token 刷新的複雜與易碎（雲端 OAuth 在 agy print 模式下無法可靠刷新）。**替代方案**：雲端 `cloudcode-pa.googleapis.com` + oauth_creds.json——否決作為主路徑（需處理刷新、token 易過期），保留為未來 fallback。

### D4：Quota adapter 沿用 `QuotaAdapter` trait，映射進既有 `QuotaWindow`

`RetrieveUserQuotaSummary` 的 `remainingFraction`/`resetTime` 直接對應既有 `QuotaWindow` 的 `utilization`/`resets_at`（`utilization = 1 - remainingFraction`），與 codex adapter 的 5h／7d 視窗語意一致，前端 quota 元件可最大化複用。群組資訊（Gemini vs Claude/GPT）額外攜帶，供狀態列／Dashboard 決定顯示範圍。

### D5：Hook 採安裝式整合，比照 Claude/Codex 的 install/detect/uninstall 模式（實作修正）

原假設「前端沿用 Claude hook 編輯模式、提供使用者手動新增/編輯/刪除任意 hook 的 CRUD 介面」經使用者於實機驗證階段否決——使用者期待 Antigravity 與其他四個 provider 一致，呈現為單一「已安裝／未安裝」狀態卡片，由 SessionHub 自動寫入固定的 marker hook 群組（`~/.gemini/config/hooks.json` 中群組名含 `sessionhub-provider-event-bridge` 標記），使用者僅需安裝／解除安裝／重新檢查，不手動編輯 hook 內容。

沿用 `provider/` 模組既有的 `install_or_update_*_integration` / `detect_*_integration_status` / `uninstall_*_integration` 三函式模式（最接近 `provider/codex.rs`，同為 JSON hooks 檔案），在 `provider/mod.rs` 的三個泛用分派函式（`recheck_provider_integration_status` 等）新增 `ANTIGRAVITY_PROVIDER` match arm。

**範圍限縮**：本 change 不建立即時事件（bridge）管線——marker hook 的 command 僅為佔位識別（`echo sessionhub-provider-event-bridge`），不驅動 activity hint 或即時 session 更新；`provider/bridge.rs` 的 `provider_refresh_event_name` 維持不含 antigravity 分支。Session 列表更新仍透過既有掃描機制。即時事件管線（比照 Claude/Codex 的 Node.js hook 腳本 + bridge JSONL）留待未來 change，因為需要針對 Antigravity hook 呼叫時的 stdin/參數格式另行設計腳本，超出本 change 範圍。

`antigravity_hooks.rs` 保留 hooks.json 的 serde 型別與讀寫 helper（供 `provider/antigravity.rs` 的安裝寫入器使用），移除原先設計的手動 CRUD 對外 command 介面。

### D6：進程／port 探測封裝在平台層

以 `tasklist` + `netstat`（Windows）找 `language_server.exe` 的 PID 與 LISTENING port。封裝成獨立函式，非 Windows 回傳「不支援」，adapter 據此降級。

## Risks / Trade-offs

- **[Quota 依賴 IDE 執行中]** → adapter 明確處理「LS 未執行」狀態並附說明；不可用不視為錯誤，不影響其他 provider。
- **[LS 綁當前 IDE 登入帳號]**（實測帳號與 SessionHub 記錄的 email 不同）→ quota 反映的是 IDE 當前登入者，UI 可顯示 `GetUserStatus` 回傳的帳號名以避免混淆。
- **[動態 port + CSRF 可能隨版本變動]** → 逐一嘗試候選 port、以 regex 撈 csrfToken 並容錯；RPC 失敗回傳錯誤狀態而非崩潰，方便日後定位。
- **[進程／port 探測僅 Windows]** → 平台層封裝，非 Windows 降級為不可用，不阻擋整體功能；未來可補其他平台。
- **[metadata.json 檔名為萬用比對]**（`*.metadata.json` 可能多個）→ 選取代表性檔案（如 conversation 層級 metadata）並在缺失時以目錄 mtime fallback。
- **[protobuf 內文不解析]** → 使用者若期待完整對話瀏覽會落空；本 change 明確界定為列表層級，內文瀏覽留待後續。

## Migration Plan

- 純新增功能，無資料遷移。新增 provider 與 quota adapter 預設可透過 `enabled_providers` / `quota_enabled_providers` 控制；未啟用時零影響。
- 前端 provider 開關預設可選擇是否預設啟用 Antigravity（建議預設啟用 session、quota 依 LS 可用性自然降級）。
- Rollback：移除 provider 註冊與 adapter 註冊即可完全停用，不留殘跡。

## Open Questions

- Antigravity session 是否需要 stats（token 統計）？初期僅列表，stats 可留待後續（transcript.jsonl 具備來源，但格式與既有 provider 不同）。
- 全域 hook 路徑最終為 `~/.gemini/config/hooks.json` 或 `~/.gemini/antigravity-cli/hooks.json`：實作時以實機驗證何者為 Antigravity 實際讀取者為準（機器上目前兩者皆無）。
- Antigravity session 是否需支援 archived 狀態？其他 provider 有 archive 概念，Antigravity 目錄結構尚未見對應，初期可不支援。
