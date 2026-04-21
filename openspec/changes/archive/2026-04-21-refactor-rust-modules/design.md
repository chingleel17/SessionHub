## Context

`src-tauri/src/lib.rs` 目前為 7263 行的單一大檔，包含：
- 型別定義（structs/enums）~350 行
- 路徑解析與工具函式 ~250 行
- FS Watcher（sessions + opencode + provider bridge）~370 行
- Provider 整合（安裝/偵測/更新）~750 行
- OpenCode session 解析 ~500 行
- SQLite DB 初始化與 CRUD helpers ~700 行
- Session 掃描邏輯（copilot + opencode 增量掃描）~700 行
- 統計計算（copilot events.jsonl + opencode messages）~450 行
- 活動狀態偵測 ~200 行
- Sisyphus/OpenSpec 資料掃描 ~400 行
- Tauri commands（26 個） ~300 行
- win32 FFI ~80 行
- 單元測試 ~1200 行

Rust 慣用的模組化方式為在 `src/` 下建立子目錄或獨立 `.rs` 檔，透過 `mod` 宣告引入，並以 `pub(crate)` 控制跨模組 visibility。

## Goals / Non-Goals

**Goals:**
- 將 `lib.rs` 依職責邊界拆分為多個子模組，每個模組控制在 ~500 行以內
- `lib.rs` 本身最終只保留 `mod` 宣告與 `pub fn run()`（目標 < 70 行）
- 所有現有 `#[tauri::command]` 公開簽名保持不變
- `cargo test` 全數通過
- 拆分過程採增量方式，每步驟皆可獨立編譯與測試

**Non-Goals:**
- 重構業務邏輯或修改 API 行為
- 修改前端 `src/` 代碼
- 修改 `Cargo.toml` 依賴
- 新增或刪除任何 Tauri command

## Decisions

### 決策 1：模組目錄結構

採用「平層 + 兩層子目錄」混合方式，依內聚性決定是否建立子目錄：

```
src-tauri/src/
├── lib.rs                      # mod 宣告 + run()
├── types.rs                    # 所有公用 struct/enum（無邏輯）
├── db.rs                       # SQLite 連線、init、migrate、CRUD
├── settings.rs                 # load/save settings, detect_terminal/vscode
├── watcher.rs                  # WatcherState + 所有 create_*_watcher
├── stats.rs                    # parse_session_stats, calculate_opencode_stats
├── activity.rs                 # get_*_activity_status
├── plan.rs                     # read/write/watch plan file helpers
├── sisyphus.rs                 # scan_sisyphus_internal + related structs
├── openspec_scan.rs            # scan_openspec_internal + related structs
├── sessions/
│   ├── mod.rs                  # get_sessions_internal, shared helpers
│   ├── copilot.rs              # scan_copilot_incremental_internal
│   └── opencode.rs             # scan_opencode_*, OpencodeMessage structs
├── provider/
│   ├── mod.rs                  # shared helpers, ProviderBridgeDiagnostics
│   ├── bridge.rs               # process_provider_bridge_event, read_last_bridge_record
│   ├── copilot.rs              # detect/install copilot integration
│   └── opencode.rs             # detect/install opencode integration
├── commands/
│   ├── mod.rs                  # re-export all commands
│   ├── sessions.rs             # get_sessions, archive, delete, stats commands
│   ├── settings.rs             # get_settings, save_settings, detect_* commands
│   ├── plan.rs                 # read_plan, write_plan, open_plan_external
│   ├── tools.rs                # open_terminal, open_in_tool, check_tool_availability
│   └── provider.rs             # install/update/recheck provider integration
└── platform/
    ├── mod.rs
    └── win32_focus.rs          # WIN32 FFI focus helpers（已是 mod，直接搬移）
```

**理由**：`sessions/`、`provider/`、`commands/` 各有 3+ 個內聚子集，值得子目錄；其他模組為獨立職責，平層即可。

**替代方案考量**：全部平層（過多檔案，失去分組語意）；全部子目錄（過度分層，小模組不值得）。

### 決策 2：Visibility 策略

- 型別定義在 `types.rs` 用 `pub(crate)` 或直接 `pub`（因 Tauri command 需要序列化，struct 本身為 `pub`）
- 內部 helper 函式改用 `pub(crate)` 而非 `pub`
- `#[tauri::command]` 函式維持 `pub fn`（Tauri macro 需要）
- `commands/mod.rs` 用 `pub use` re-export 所有 commands，`lib.rs` 只引用 `commands`

**理由**：最小化公開 surface，同時讓跨模組呼叫順暢。

### 決策 3：增量拆分順序

依依賴關係由底層往上拆，確保每步都可編譯：

1. `types.rs`（零依賴，純搬移）
2. `db.rs`（依賴 types）
3. `settings.rs`（依賴 types, db）
4. `platform/win32_focus.rs`（獨立 WIN32 mod，直接搬）
5. `stats.rs`（依賴 types, db）
6. `activity.rs`（依賴 types）
7. `plan.rs`（無特殊依賴）
8. `sisyphus.rs` + `openspec_scan.rs`（獨立掃描邏輯）
9. `sessions/`（依賴 types, db, stats）
10. `provider/`（依賴 types, settings）
11. `watcher.rs`（依賴 sessions, provider, types）
12. `commands/`（依賴所有上層模組）
13. `lib.rs` 清理（只留 mod + run）

## Risks / Trade-offs

- **[風險] 循環依賴**：若 sessions 依賴 provider 或反之，會造成編譯錯誤 → 緩解：拆分前先在 lib.rs 中確認各函式的呼叫圖，確保依賴單向
- **[風險] pub(crate) 邊界錯誤**：某些 helper 被多個模組呼叫，visibility 設錯會編譯失敗 → 緩解：每步拆分後立即 `cargo build` 確認
- **[Trade-off] 拆分步驟數多**：12 個步驟，但每步獨立可驗證，比一次大拆安全
- **[Trade-off] `use` 宣告增加**：各模組頂部需明確 `use crate::types::*`，略增行數，但換來清晰的依賴表達

## Migration Plan

1. 建立新模組檔案（空白或含最小內容）
2. 從 `lib.rs` 搬移對應程式碼至新模組
3. 在 `lib.rs` 加入 `mod <module>; use crate::<module>::*;`（過渡期）
4. 執行 `cargo build` 確認無編譯錯誤
5. 執行 `cargo test` 確認測試全數通過
6. 重複至所有模組完成
7. 最終清理 `lib.rs` 中的過渡 `use` 宣告，改為精確引用

**Rollback**：每步驟皆為純搬移，git revert 任一 commit 可恢復。

## Open Questions

- 無。拆分策略與邊界已明確，可直接執行。
