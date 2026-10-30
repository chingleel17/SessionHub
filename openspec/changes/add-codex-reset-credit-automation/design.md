# Design

## Context

動機見 `proposal.md`。目前資料與限制如下：

- `src-tauri/src/types/quota.rs` 的 `ResetCredits` 提供總可用張數與明細；每筆僅有 `granted_at`、`expires_at`、`status`，沒有穩定券 ID。到期時間可缺少，既有轉換精度為 UTC 秒。
- `quota/codex.rs` 已有 GET reset-credits 與 POST consume。POST 僅接受 `redeem_request_id`，不能指定券；目前沒有服務端最早到期優先或重試冪等性的已驗證契約。
- 查詢可退回 OpenCode 的 OpenAI OAuth，但現有消耗僅讀 Codex CLI 憑證，需讓查詢／消耗使用同一個解析結果。
- `app_setup.rs` 現有背景用量刷新每 60 秒檢查、約每 30 分鐘刷新，不足以處理一分鐘的到期區間。
- 設定由 `AppSettings` 持久化；原生通知已有 Tauri plugin。前端 IPC 集中 `App.tsx`，通知與消耗不能依賴前端頁面保持掛載。

## Goals / Non-Goals

**Goals:**

- 將時間判斷與副作用分離，可用假時鐘、假 API 與假通知驗證邊界。
- 對同一帳號的一次消耗使用同一組已解析憑證，並在手動／自動、重啟及併發之間協調。
- 以小型背景協調器及既有 SQLite／通知能力完成，避免建立通用排程系統。

**Non-Goals:**

- 不建立逐券 ID、不模擬使用者產生 Codex 用量來耗完券、不猜測服務端選券順序。
- 不新增跨多帳號管理、雲端排程、OS 常駐服務或退出後執行機制。

## Decisions

### 1. 固定自動使用提前量，提醒使用日曆天

設定欄位與預設值以 `specs/app-settings/spec.md` 為準：兩個開關預設關閉，每日 `09:00`，提前 `0–30` 天。自動使用固定 `60` 秒，以具名常數表示，不增加另一個提前量欄位。

- 提醒採本地日曆日期，區間為 `(now, start_of_local_day(today + days_before + 1))`；「今天」及「未來 D 天」分別計數。跨日計算使用日期／時區轉換，不用 `24h * 天數` 代替，處理日光節約時間。
- 自動使用以 UTC 時刻比較，區間為 `[expires_at - 60s, expires_at)`，恰好到期不送新請求。
- 未知 status 不推定可用；`active` 且缺到期時間可繼續顯示，但不排程。自動使用前核對總可用張數與 active 明細數；不一致即暫停。
- 前後端均驗證 `HH:mm` 與整數天數；載入時使用逐欄位容錯／正規化，避免負數或錯誤型別使整份設定載入失敗。儲存時拒絕無效輸入，不把錯誤資料靜默存成預設值。

**取捨：** 以「每日時間＋提前天數」滿足需求，避免任意 cron、逐券時間及小時／分鐘提醒組合造成設定複雜化。

### 2. Rust 背景協調器處理到期工作

新增 `quota/reset_credit_automation.rs`，由 `app_setup.rs` 啟動一個可被事件喚醒的背景協調器。狀態由後端持有；command 為薄包裝，協調器不依賴 `commands/` 或 React。

觸發來源：

1. 啟動、相關設定儲存、憑證／帳號變更、既有 Codex 快照更新。
2. 最長每 10 秒檢查牆上時鐘、設定版本與到期工作；喚醒後的第一輪自然重算，無需以專用喚醒事件作為唯一依據。
3. 自動使用有效時，最多每 5 分鐘補查一次目前帳號的重置券，用於發現新券／手動兌換；不對所有 provider 執行高頻刷新。
4. 每日提醒與實際消耗前，再執行不使用舊快取的查詢。一般快照只用於發現候選工作，不授權副作用。

以最早下一工作時間與 10 秒上限決定等待時間；網路工作在背景執行，不阻塞 UI 或其他 provider 刷新。每帳號查詢合併，避免提醒、五分鐘更新與到期事件同時觸發重複查詢。設定儲存立即喚醒協調器；在 POST 前再次核對開關、前置條件、帳號與 UTC 到期時間。

GET 的暫時性失敗可有界重試：每日檢查最多 3 次，首次失敗後分別等待 30 秒、120 秒；到期前查詢最多 3 次，失敗後分別等待 10 秒、20 秒，且每次重試都不得超出到期區間。尊重服務端較長的退避要求，若將超過期限則停止。單筆 HTTP 呼叫設定不超過 10 秒的逾時，POST 無自動重送。

**取捨：** 延續程序內背景執行，不使用前端 `setInterval`，也不新增工作排程器。正常喚醒的檢查精度為十秒等級，不保證作業系統負載、休眠或網路延遲下的準點成功。

### 3. 共用帳號解析與消耗協調

將現有憑證解析整理為 quota 內部的 account context，包含憑證來源、ChatGPT 帳號 ID 及僅在記憶體使用的授權資料。GET 與 POST 接收同一 context，保留既有 Codex 優先、OpenCode 備援來源；不得查到 A 的券後重新解析並送往 B。

持久化的 `account_scope` 為 provider 與 ChatGPT 帳號 ID 組成的穩定雜湊，不使用 access token 或憑證路徑作身分。缺少可驗證帳號 ID 時可保留既有唯讀顯示，但禁止消耗。帳號切換取消舊候選工作、清除僅依 provider 建立的自動化暫存，重新查新帳號。舊帳號的歷史紀錄不刪除，也不顯示成新帳號的結果。

手動與自動均進入 quota 內部的同帳號消耗協調器，先取得執行權，再重查資料、寫入請求紀錄、POST、記錄結果並刷新。使用執行中的帳號鎖及資料庫唯一條件避免不同觸發來源重複執行；不得持有全域 DB／快取 mutex 等待網路回應。若有另一筆請求進行中，手動路徑可回報處理中，自動路徑留待下輪重新評估。

**取捨：** 重用現有 API 與 UI 確認流程，不由背景協調器呼叫 Tauri command，也不新增自動模式繞過共用協調。

### 4. 以到期批次處理多張券，不偽造逐券識別

自動批次以 `(account_scope, expires_at_utc)` 識別，同效期相同明細以張數保留，不以內容去重而遺失張數。首筆請求前保存該批次可用數作為本批次上限；批次已有終止紀錄時，不因刷新、重啟或不同設定版本重新產生批次。

目前沒有已驗證的選券契約，因此首版僅允許：

- 只有一張可用券，且該券已進入到期區間。
- 所有可用券均有相同且已知的到期時間，且已進入到期區間。

若未來能以可核對的服務端契約證明最早到期優先，再放寬混合效期；本次不得從回傳排序或單次觀察推論。任何 active 券缺到期時間、混合效期或明細／總數不一致，均顯示對應暫停原因，不盲目消耗較晚到期券。

允許的批次逐次 POST，`reset` 後必須重新查詢，確認該批次張數減少且仍未到期，才繼續。總請求數受最初到期張數限制；`nothing_to_reset`、`no_credit`、`already_redeemed`、錯誤、結果不明或無法確認張數減少皆終止本批次，不在下一個 tick 重新開始。`nothing_to_reset` 只表示此次無需重置，不推論所有剩餘券已使用。

**取捨：** 無法保證所有券都能使用，尤其服務端沒有需要重置的用量時；保留這個實際結果比反覆 POST 或假造成功更符合 API 能力。

### 5. SQLite 保存提醒去重與消耗日誌

在現有 `metadata.db` 進行加法式 migration，業務 helper 置於 `db.rs`／既有 DB 分工：

| 紀錄 | 唯一範圍 | 必要內容 |
| --- | --- | --- |
| `reset_credit_reminder_checks` | `account_scope + local_date` | 檢查狀態、今日／未來張數、檢查與通知提交時間、錯誤分類 |
| `reset_credit_auto_batches` | `account_scope + expires_at` | 初始張數、已嘗試序號、批次狀態、停止原因 |
| `reset_credit_consume_attempts` | `request_id`；自動來源另限 `batch + ordinal` 唯一 | manual／automatic 來源、帳號、觀察到的到期批次、送出前時間、狀態、API 結果、更新時間 |

提醒成功查到零張時寫 `checked_empty`，當天不再重查提醒。通知前先原子保留當日的 `claimed` 紀錄，再呼叫通知；提交後寫 `dispatched` 或 `failed`。通知 API 只能證明提交，不能證明顯示或已讀；崩潰留下的 `claimed` 不再自動重送，採至多一次提交。GET 失敗尚未進入通知保留階段，可依有界策略重試；耗盡後記錄當日檢查失敗。當日修改提醒時間／提前天數不清除已檢查紀錄。

消耗送出前先保存 request ID 與 `pending`，確保不能在沒有可寫日誌時消耗。回應已知時寫結果；逾時、連線中斷、未知格式或重啟遺留的 `pending` 轉 `unknown`。不假定 `redeem_request_id` 具足以安全重送的服務端冪等保證，因此不重送同 ID，也不換新 ID 自動重送。

`unknown` 存在時暫停該帳號後續消耗，重新查詢以對照原先觀察的到期批次。原批次已不存在可用券或全部到期後解除阻擋，但歷史仍保留結果不明，不改標成功。手動不指定券，故其不明紀錄須包含請求前所有可用批次，不能只猜最早一批；無到期資訊的原券仍存在時持續保留阻擋。

若 DB 無法寫入、已存在無法排除的 pending／unknown，或刷新不能核對帳號，禁止新的消耗。只存分類及必要到期資訊，不存 token、授權 header 或原始憑證。

**取捨：** 採持久化「至多一次發送＋可觀察不確定結果」，不宣稱不可能保證的端到端 exactly-once。

### 6. 設定與狀態使用既有 UI 模式

在 SettingsView 的 quota 區新增精簡分區，使用既有 switch、時間與數字輸入及 `sessionhub-minimal-ui` 設計語言。子元件接收 props；儲存、初次狀態讀取與事件訂閱集中於 App。

新增唯讀自動化狀態 IPC，由 App 取得／訂閱後傳給設定頁，內容包括目前帳號的提醒／自動使用有效狀態、原因代碼、下一次提醒檢查時間、最近一次嘗試時間與結果。不將 token 或原始帳號 ID 傳到通知文案。原因及結果以穩定代碼傳遞，前端與 Rust 通知依目前已儲存語系產生文案。

狀態區區分 `disabled`、`ready`、`paused`、`running`，結果區分 `reset`、`nothing_to_reset`、`no_credit`、`already_redeemed`、`failed`、`unknown`、`missed`。消耗完成後重新整理 Codex 快照並發事件，沿用現有刷新管線更新 Dashboard、狀態列及其他用量顯示。

原生到期提醒僅受提醒開關控制；自動使用結果保留在設定狀態及既有應用程式回饋，不額外引入獨立「結果通知」開關或通知中心。通知權限僅影響提醒。沿用 SessionHub 正式通知 identity，以安裝版驗證；使用者沒有權限或 OS 不顯示時，狀態清楚說明，不阻斷自動使用。

## Risks / Trade-offs

- **一分鐘區間短且有網路延遲** → 十秒檢查、事前發現候選、請求逾時及到期前重查；超過期限不補用，明確呈現失敗／錯過。
- **無指定券與選券順序契約** → 首版採單張或全數同效期的可確認範圍，混合效期提供暫停原因，不能宣稱必用掉特定券。
- **程序退出、休眠與關機** → 生命週期補查僅針對仍有效工作，設定頁說明執行範圍；不自動修改開機啟動或最小化至系統匣設定。
- **至多一次通知可能在崩潰時漏送** → 接受比重複提醒更小的副作用，保留提交狀態供檢查。
- **API 成功與 DB 更新無法形成跨系統交易** → 預先日誌及 unknown 狀態，不盲目重送；同一帳號手動與自動都遵守阻擋。
- **既有 quota 查詢為 best-effort，null 不等於零券** → 自動化使用有明確成功／失敗分類的直接查詢，保留既有 Dashboard best-effort 行為。

## Migration Plan

1. 新增設定預設及逐欄位驗證，缺新欄位的舊設定載入後兩個開關皆關閉。
2. 新增 SQLite 表與唯一條件，不修改或刪除既有快照／session 資料。
3. 先接妥共用帳號及手動消耗協調，再啟動提醒／自動批次；背景功能只有在已儲存設定開啟時運作。
4. 以隔離資料目錄、假時鐘／假 HTTP 完成自動驗收；原生通知以 Windows 安裝版驗收，測試不得實際消耗真實帳號的券。
5. 回復時先關閉兩項開關並確認沒有執行中請求。可保留新增表及設定欄位；回復到不認識日誌的舊版本時，不重播未完成工作，舊版手動操作無法提供新版本的不明結果阻擋保證。

## References

- 專案既有契約：`openspec/specs/provider-quota-monitoring/spec.md`、`openspec/specs/app-settings/spec.md`、`openspec/specs/windows-notification-identity/spec.md`。
- Tauri 2 Notifications（2026-10-30 查核）：https://v2.tauri.app/plugin/notification/
- Tauri Process Model：https://v2.tauri.app/concept/process-model/
- Tauri System Tray：https://v2.tauri.app/learn/system-tray/
- Microsoft App Notifications：https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/
