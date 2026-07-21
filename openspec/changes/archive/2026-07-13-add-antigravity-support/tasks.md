## 1. 後端基礎：provider 常數與設定

- [x] 1.1 在 `types.rs` 新增 `ANTIGRAVITY_PROVIDER: &str = "antigravity"` 常數
- [x] 1.2 在 `default_enabled_providers_all()` 與（視預設策略）`default_enabled_providers()` 加入 antigravity
- [x] 1.3 在 `AppSettings` 視需要新增 `antigravity_root`（預設 `~/.gemini`）與其 serde default 函式
- [x] 1.4 在 `settings.rs` 新增 `resolve_antigravity_root` / `default_antigravity_root`，供掃描與 quota 共用

## 2. Session Provider

- [x] 2.1 新增 `src-tauri/src/sessions/antigravity.rs`，掃描三個 brain root（antigravity / antigravity-cli / antigravity-ide），沿符號連結讀取；只把含 `.system_generated/logs/transcript.jsonl` 的目錄視為 session
- [x] 2.2 實作 `agyhub_summaries_proto.pb` 輕量解析（UUID → 標題 → `file:///` workspace），供 IDE 形態查表；URL-decode workspace 路徑
- [x] 2.3 IDE 形態：標題優先取 summaries.pb，查無則 fallback 為 transcript 首則 `USER_REQUEST` 前段；workspace 取自 summaries，查無留空
- [x] 2.4 CLI 形態（無 summaries.pb）：標題取 transcript 首則 request，workspace 查無留空
- [x] 2.5 時間：取 transcript 首/末則 `created_at`，不可得則目錄 mtime；映射為 `SessionInfo`（provider=antigravity），解析失敗標記 parse_error 不中斷
- [x] 2.6 在 `sessions/mod.rs` 加入 antigravity 掃描分支，接入既有 ProviderCache／增量掃描／git 補強流程
- [x] 2.7 在 `sessions/mod.rs` 的 `pub mod` 與 `pub(crate) use` 註冊 antigravity 模組

## 3. Quota Adapter

- [x] 3.1 新增平台層函式：以 `tasklist` + `netstat`（Windows）找 `language_server.exe` PID 與 LISTENING port；非 Windows 回傳不支援
- [x] 3.2 實作 CSRF 流程：GET `http://127.0.0.1:<port>/` 解析 `csrfToken`
- [x] 3.3 實作 RPC 呼叫：POST `/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary`（帶 `x-codeium-csrf-token`、`Origin`），逐一嘗試候選 port
- [x] 3.4 新增 `src-tauri/src/quota/antigravity.rs`，實作 `QuotaAdapter`：解析 `groups[].buckets[]`，映射 `utilization = 1 - remainingFraction`、`resets_at = resetTime`，保留群組區分
- [x] 3.5 處理不可用狀態：LS 未執行→回傳 `no_auth`/不可用 snapshot（附說明）；RPC 失敗→error snapshot；不中斷其他 provider
- [x] 3.6 在 `quota/mod.rs` 的 `QuotaManager::new` 註冊 `AntigravityAdapter`，受 `quota_enabled_providers` 控制
- [x] 3.7 為 adapter 加單元測試：LS 不可用時回傳不可用狀態、provider key 正確

## 4. Hook 管理（後端）—— 安裝式整合（比照 Claude/Codex，非手動 CRUD，見 design.md D5 修正）

- [x] 4.1 `antigravity_hooks.rs` 定義 Antigravity hook 的 serde 型別（群組→enabled + 事件鍵→matcher 陣列→hooks{type,command,timeout}），供安裝寫入器使用
- [x] 4.2 實作全域 `~/.gemini/config/hooks.json` 讀寫 helper；不存在時回傳空清單，安裝時自動建立
- [x] 4.3 新增 `provider/antigravity.rs`，實作 `install_or_update_antigravity_integration` / `detect_antigravity_integration_status` / `uninstall_antigravity_integration`，寫入／偵測／移除 SessionHub marker hook 群組，保留其餘既有群組不受影響
- [x] 4.4 在 `provider/mod.rs` 的三個泛用分派函式（`recheck_provider_integration_status` 等）與 `settings.rs` 的 `collect_provider_integration_statuses` 新增 antigravity 分支

## 5. 前端整合

- [x] 5.1 `src/utils/providerLabel.ts` 新增 `case "antigravity": return "Antigravity"`
- [x] 5.2 `types/index.ts` 補上 antigravity provider／quota 群組相關型別（`AppSettings.antigravityRoot`、`QuotaWindow.group`）
- [x] 5.3 `SettingsView.tsx` 新增 Antigravity 的 session provider 與 quota 開關
- [x] 5.4 quota 顯示：底部狀態列（`global-status-bar`）只顯示 Gemini 群組
- [x] 5.5 quota 顯示：Dashboard 顯示 Gemini 與 Claude/GPT 兩群組（依 group 分組呈現）
- [x] 5.6 hook 整合 UI：Antigravity 併入既有 provider 安裝狀態卡片清單，複用 install/update/recheck/uninstall 按鈕與狀態徽章（與 Claude/Codex 同一元件），不建立獨立 CRUD 表單
- [x] 5.7 在 `App.tsx` 集中新增對應 invoke 呼叫（session/quota），子元件以 props 驅動
- [x] 5.8 所有新增文案透過 `t()` 加入 localization，無硬編中文

## 6. 驗證

- [x] 6.1 `cargo build` 與 `cargo test` 通過（128 passed）
- [x] 6.2 實機驗證：Antigravity session 出現在列表、標題／時間正確（實測 42 筆真實 session，含正確 Traditional Chinese 標題與 URL-decode workspace）
- [x] 6.3 實機驗證：IDE 開啟時 quota 顯示真實 5h／週數字；關閉時優雅降級為不可用（實測取得 Gemini Models 與 Claude and GPT models 兩群組的真實 5h/weekly utilization 與 resetsAt）
- [x] 6.4 實機驗證：安裝／偵測／解除安裝後 hooks.json 內容正確（marker 群組寫入/移除），且不影響既有其他群組
- [x] 6.5 前端 `npm run build` 通過（專案未設定 lint script，以 `tsc` 型別檢查作為靜態檢查基準，通過）
