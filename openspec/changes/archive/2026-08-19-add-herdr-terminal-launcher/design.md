## Context

動機見 `proposal.md` — Why。此處只列影響設計的現況與限制。

**現況的啟動契約**：`terminal_path` 被視為「一個 shell 執行檔」，以 `Command::new(term)` + `CREATE_NEW_CONSOLE` 開新視窗，並依 `file_stem` 推導 argv。這個 stem-switch 在後端重複約 8 次：

- `sessions/copilot.rs` — `open_terminal_internal`（`cmd` → `/K`；`bash|sh` → `-i`；其餘 → `-NoExit -Command`）
- `commands/tools.rs` — `open_in_tool_internal` 中 terminal / opencode / claude / codex / copilot / gemini / vscode shim 共 7 個分支，以及 `resume_session_in_terminal_internal`

每個分支的差異僅在於「要送出的指令字串」，其餘（stem 判斷、`current_dir`、`creation_flags`、`configure_msys_stackdump_suppression`）完全相同。

**herdr 的實測行為**（0.8.0-preview，本機 `C:\Users\User\AppData\Local\Programs\Herdr\bin\herdr.exe`，`herdr status server` 回報 running 且 `compatible: yes`）：

- `herdr tab create --cwd <PATH> --label <TEXT> [--focus|--no-focus]` 回傳單行 JSON，形如
  `{"id":"cli:tab:create","result":{"root_pane":{"pane_id":"w1:pQ",...},"tab":{"tab_id":"w1:t7",...},"type":"tab_created"}}`
- `herdr pane run <PANE_ID> <COMMAND>...` 將指令送入該 pane（實測以 `herdr pane read` 確認指令確實執行完成）
- 因此 herdr 路徑必須 **擷取 stdout 並解析 JSON**，與現行 fire-and-forget 的 `spawn()` 不同，需改用 `output()`

**兩處既有機制與 herdr 不相容**：

1. `commands/tools.rs` 的 `focus_terminal_window_internal` 依視窗 class 與標題比對定位 console 視窗；herdr 是單一視窗承載 N 個 pane，標題比對必然找錯。
2. `settings.rs` 的 `validate_terminal_path_internal` 同時要求「檔案存在」與「file_stem 屬於 `VALID_TERMINAL_STEMS`（pwsh/powershell/cmd/bash/sh）」。`herdr` 兩項皆不符 — 它通常由 PATH 解析，且不在白名單內。

## Goals / Non-Goals

**Goals:**

- 以「啟動器種類」為軸心分派，讓 shell 與 herdr 兩條路徑共用同一個決策點
- 收斂重複的 stem-switch，使新增啟動器不需再改動 8 個分支
- shell 為預設值，既有使用者行為與設定檔完全不變（零遷移成本）
- herdr 失敗時錯誤訊息可指出成因（未安裝 / server 未執行 / 解析失敗）

**Non-Goals:**

- 不支援 tmux 或其他多工器（見 proposal 範圍限制）
- 不做 herdr 的 session／workspace 管理（`--session`、`workspace create` 等），僅用 tab + pane
- 不追蹤已建立的 pane 生命週期，也不在 SessionHub 內嵌 herdr 畫面
- 不改動 `explorer` 與 `vscode` 的啟動方式（兩者本就不經終端）

## Decisions

### D1: 新增 `terminal_launcher` 欄位，而非重用 `terminal_path`

**選擇**：`AppSettings` 新增 `terminal_launcher: Option<String>`（`"shell"` | `"herdr"`），以 `#[serde(default)]` 標註，缺漏時視為 `"shell"`。

**理由**：若把 `herdr` 直接填進 `terminal_path`，其 stem 不在白名單內會落入 `_ =>` 分支被餵 `-NoExit -Command`，產生無效呼叫；且 `validate_terminal_path_internal` 會先擋下儲存。兩者語意也不同 — `terminal_path` 是「哪個 shell」，launcher 是「用什麼方式承載」。在 herdr 模式下 `terminal_path` 仍有意義（herdr pane 內部仍跑某個 shell，由 herdr 自身設定決定），故兩欄位並存而非互斥。

**替代方案**：以 `terminal_path` 的 stem 是否為 `herdr` 隱式判斷。捨棄 — 隱式行為難以在 UI 表達，且與既有白名單驗證衝突。

### D2: 抽出 `TerminalLaunchSpec` + 單一分派函式

**選擇**：定義一個描述「要在終端裡做什麼」的結構，例如：

```
struct TerminalLaunchSpec<'a> {
    cwd: &'a str,
    command: Option<&'a str>,  // None = 純開終端
    label: &'a str,            // herdr tab 標籤 / 診斷用
}
```

由單一函式 `launch_terminal(launcher, terminal_path, spec) -> Result<(), String>` 依 launcher 分派到 `launch_via_shell` 或 `launch_via_herdr`。`open_in_tool_internal` 的 7 個分支退化為「組出 command 字串 + 呼叫 launch_terminal」。

**理由**：現有 8 處差異只在 command 字串，其餘皆同。不收斂的話新增 herdr 會讓重複量翻倍（8 → 16）。這不是順手重構 — 是本變更的前置條件。

**替代方案**：在每個分支各加一個 `if launcher == herdr`。捨棄 — 重複量翻倍，且 herdr 的兩段式流程會被複製 8 次。

**邊界**：`vscode` 分支中「以終端執行 shim 腳本」屬於 `CREATE_NO_WINDOW` 的一次性呼叫，不是給使用者互動的終端，維持既有 shell 行為、不走 launcher 分派。`explorer` 同理不受影響。

### D3: herdr 以 `output()` 擷取 JSON，只取所需欄位

**選擇**：新增 `sessions/herdr.rs`（或 `commands/` 下的對應模組，依實作時的相依方向決定），內含：

- `herdr_tab_create(cwd, label, focus) -> Result<HerdrPane, String>` — 執行 `tab create`，以 `output()` 取得 stdout，用 `serde_json::Value` 取 `result.root_pane.pane_id`
- `herdr_pane_run(pane_id, command) -> Result<(), String>`

以 `serde_json::Value` 逐層取值，而非為完整回應定義 struct。

**理由**：herdr 處於 preview 階段（版本字串 `0.8.0-preview.2026-08-04`），完整回應 schema 可能變動；只依賴 `result.root_pane.pane_id` 一個欄位，可將破壞面縮到最小。專案已有 `serde_json` 相依，不新增套件。

**替代方案**：改用 herdr socket API（`herdr api`）。捨棄 — CLI 已足夠且已實測驗證，socket 協定（protocol 19）另有版本相容性負擔，違反 YAGNI。

### D4: 聚焦改以 tab 識別碼定位，不改動既有 Win32 邏輯

**選擇**：herdr 模式下 `tab create` 帶 `--focus`；並保存建立時取得的 `result.tab.tab_id`，於使用者觸發聚焦時以 `herdr tab focus <tab_id>` 定位。`focus_terminal_window_internal` 在 herdr 模式不進入 EnumWindows 比對。

**理由**：herdr 單視窗多 pane，標題比對會聚焦到錯誤 pane —— 比「不聚焦」更糟，因為使用者以為切換成功。已實測 `herdr tab focus <tab_id>` 可用：對 `wB:t3` 執行後 `tab list` 顯示該 tab `focused: true`，故無須將聚焦列為不支援。既有 Win32 程式碼保持不動，只在入口加 launcher 判斷。

**tab_id 保存範圍**：以應用程式執行期記憶體內的 session → tab_id 對應表保存（例如 AppState 中的 map），不寫入 SQLite。理由是 herdr tab 的生命週期不長於 herdr server，跨應用程式重啟後的舊 tab_id 無保證仍有效；持久化只會帶來失效資料的處理成本。對應不存在時回傳明確錯誤（見 specs）。

**替代方案**：以 tab label 反查 `tab list`。捨棄 — label 不保證唯一，且多開同一專案時會誤判。

### D5: 驗證依 launcher 分流

**選擇**：`validate_terminal_path` command 增加 launcher 參數（或新增一個對應 command）。launcher 為 `herdr` 時：跳過 `VALID_TERMINAL_STEMS` 白名單，改以「PATH 解析或檔案存在」判定，可重用既有的 `which_exists` 類 helper。

**理由**：`validate_terminal_path_internal` 目前同時卡「檔案存在」與「stem 白名單」，`herdr` 兩項皆不符。不分流的話設定頁會拒絕合法設定。既有 `validate_terminal_path_returns_true_for_existing_file` 測試針對 shell 行為，需維持通過。

### D7: launcher 由後端自 settings 讀取，不透過 IPC 參數傳遞

**選擇**：`open_in_tool`、`resume_session_in_terminal`、`focus_terminal_window` 三個 command 於後端以 `load_settings_internal` 讀取 `terminal_launcher`，不新增 IPC 參數。唯獨 `validate_terminal_path` 以參數接收 launcher。

**理由**：後端本就能讀設定，透過 IPC 傳遞會讓三個 command 簽章各多一個參數，且前端傳來的值可能與已儲存的設定不同步（例如設定頁編輯中尚未儲存）。`validate_terminal_path` 是例外 —— 它驗證的正是「尚未儲存的表單值」，必須由前端傳入當前選取的 launcher 與路徑。

**替代方案**：四個 command 一律由前端傳入。捨棄 — 三個啟動／聚焦入口應以「已儲存的設定」為準，前端傳值會引入漂移。

### D8: 未偵測到 herdr 時停用並標示，不隱藏選項

**選擇**：沿用 `ProjectView.tsx:592-598` 的既有慣例（`disabled` + `(未安裝)` 標示），launcher 選項在 herdr 不可用時停用並標示「未偵測到」；且當前已選取的值一律照常渲染。

**理由**：關鍵不在一致性，而在**可復原性**。若隱藏選項，使用者選了 herdr 後再移除 herdr，`settings.json` 中 `terminal_launcher` 仍是 `"herdr"`，UI 卻沒有任何路徑可切回 shell —— 應用程式會卡在「開不了終端也改不掉設定」的狀態。始終渲染當前選取值可避免此死角。

**替代方案**：完全隱藏。捨棄 — 需另加保護邏輯（偵測到不可用時自動改寫設定）才能避免上述死角，複雜度更高且會靜默更動使用者設定。

### D9: 區分「未安裝」與「服務未執行」兩種狀態

**選擇**：可用性偵測分兩層 —— 先以 `which_exists("herdr")` 判定是否安裝，已安裝再以 `herdr status server` 的輸出判定服務是否執行（實測輸出首行為 `status: running`，exit code 0）。

**理由**：兩種狀態的補救方式不同（安裝 vs 啟動）。若合併為單一「不可用」，使用者會依錯誤訊息去重新安裝一個其實已安裝的工具。

**快取更新**：`check_tool_availability` 目前是 `App.tsx:1095` 的快取查詢，新裝的 herdr 不會即時反映。於儲存設定時使該查詢失效，並比照終端機路徑欄位既有的「自動偵測」按鈕（`SettingsView.tsx:239-245`）提供手動重新偵測。

### D10: Provider 勾選區以資料根目錄存在與否作為偵測訊號

**選擇**：provider 勾選區顯示「資料根目錄是否存在」的狀態提示（可重用既有的 `directory_exists` / `check_directory_exists`），勾選框維持可用。**不**以 `which_exists` 判定 CLI 是否安裝來停用勾選。

**理由**：`enabled_providers` 控制的是**掃描資料目錄**，與 CLI 是否在 PATH 上是兩件事。CLI 可能已移除但歷史 session 仍在，或以 shim 安裝而 `where` 解析不到（`vscode` 即為此例，故其偵測改用 `resolve_vscode_command()`）。若以 CLI 存在與否停用勾選，會使這些 provider 無法啟用，**使用者自己的 session 歷史被靜默隱藏**。

**附帶影響**：`SettingsView.tsx:202-204` 的 `onChange` 在勾選時會觸發 `onProviderAction(id, "install")`，停用勾選框亦會連帶影響整合安裝流程 —— 這是另一個不應停用勾選的理由。

### D6: 前端沿用既有 IPC 集中慣例

**選擇**：`SettingsView` 以受控元件呈現 launcher 選擇（props 驅動），實際 `invoke()` 仍集中在 `App.tsx`；文案透過 `t("key")`，同步新增 zh-TW 與 en-US 字串。

**理由**：遵循專案既有慣例（子元件不得直接 invoke、JSX 不得硬編中文）。

### D11: tab 標籤格式與「總是開新 tab」

**選擇**：標籤以專案目錄名稱為主，該次啟動對應特定工具時附加工具識別（例如 `session_hub · claude`）。每次啟動一律建立新 tab，不重用既有 tab。

**理由**：標籤只承擔「使用者辨識」職責，定位一律靠 tab_id（見 D4），因此標籤不需唯一。「總是開新」與 shell 啟動器每次開新視窗的語意一致，行為可預期；重用則需處理「該 tab 內已有程式在跑」的情境，複雜度不成比例。

**已知取捨**：重複啟動同一專案會累積 tab，由使用者自行以 herdr 關閉（見 Risks）。

## Risks / Trade-offs

- **[herdr 為 preview 版，CLI 輸出格式可能變動]** → 只依賴 `result.root_pane.pane_id` 單一欄位；解析失敗時回傳含原始 stderr 片段的錯誤，便於診斷而非靜默失敗。

- **[tab_id 僅存於記憶體，應用程式重啟後既有 tab 無法聚焦]** → 屬已知取捨（見 D4）。此情境回傳「無對應 tab 識別碼」的明確錯誤並提示手動切換，不聚焦到錯誤 tab。

- **[`pane run` 送出後無法確認指令是否真的成功執行]** → herdr 僅回報「指令已送出」。設計上接受此語意（與現行 `spawn()` 同樣不等待 CLI 就緒），不額外輪詢 `pane read`，避免引入不確定的等待邏輯。

- **[抽出共用函式需改動 8 處既有啟動路徑，回歸風險集中]** → 每個分支的 command 字串在重構前後逐一比對保持一致；shell 路徑的行為（含 `creation_flags` 與 MSYS 環境）不變，可由既有測試與手動驗證覆蓋。

- **[使用者選了 herdr 但未安裝]** → 儲存設定時即以 D5 的驗證擋下並提示；執行期再次失敗時回傳明確錯誤，並依 D9 區分「未安裝」與「服務未執行」。不自動回退到 shell —— 靜默回退會讓使用者以為 herdr 生效。

- **[重複啟動同一專案會累積 herdr tab]** → 依 D11 為刻意取捨，與 shell 模式累積視窗的既有行為一致。使用者可於 herdr 內自行關閉；若日後成為困擾，可依已有的 session → tab_id 對應表加入重用邏輯，屬後續變更。

- **[已選 herdr 後移除 herdr 導致無法切回 shell]** → 依 D8，設定頁一律渲染當前選取值並標示不可用，保留切回 shell 的路徑。

## Migration Plan

- `terminal_launcher` 以 `#[serde(default)]` 加入，舊 settings.json 缺該欄位時讀為 `"shell"`，行為與現況完全一致，無需資料遷移。
- 回退策略：使用者於設定頁將 launcher 切回 `shell` 即恢復原行為；程式碼層面此變更為單一提交，可整體 revert。
