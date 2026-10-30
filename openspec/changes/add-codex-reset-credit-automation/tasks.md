# Tasks

## 1. 設定契約與時間判斷

- [x] 1.1 在 Rust／TypeScript AppSettings 及 `src/utils/appSettingsDefaults.ts` 的 DEFAULT_APP_SETTINGS／merge 白名單同步新增四個重置券欄位；測試舊設定經前端表單合併後儲存的真正往返，確認預設皆關閉、09:00、提前 0 天且新舊欄位保留。
- [x] 1.2 實作載入時逐欄位容錯與儲存時嚴格驗證；以無效時間、錯誤型別、負數、非整數及超界天數測試確認只回復壞欄位，無效儲存不覆寫原設定。
- [x] 1.3 建立不含副作用的到期資格、今日／未來張數、提醒日期與 60 秒區間判斷；以假時鐘驗證 Asia/Taipei、跨日／日光節約時間、恰好到期、缺少時間、未知狀態與今天零張但未來有券。

## 2. 共用帳號、日誌與消耗協調

- [x] 2.1 統一既有 Codex CLI／OpenCode OpenAI OAuth 憑證解析，讓查詢與消耗接收相同 account context，加入 Codex snapshot 的選填 accountScope 及內部世代；測試備援帳號一致、缺帳號 ID 拒絕消耗、舊快照僅唯讀及切帳號不誤用。
- [x] 2.2 在 metadata.db 新增提醒檢查、到期批次、消耗嘗試及端點退避紀錄與唯一條件；以隔離 SQLite 驗證 migration 可重入、舊資料保留、同日提醒／同批次序號不可重複認領，停用 provider prune 快照時不刪操作日誌。
- [x] 2.3 建立同帳號消耗協調、送出執行權、pending 日誌及不可改寫的 request ID；測試手動／自動同時到達僅一筆 POST、重複 ID 只回讀結果、不同內容拒絕，DB 寫入失敗時不送出。
- [x] 2.4 實作 pending 重啟轉 unknown、帳號級阻擋與原觀察券核對；測試逾時／中止後不重送同一或新 request ID，包含手動未知效期券持續阻擋及全部原券不可用後解除。
- [x] 2.5 補上所有 Codex 快照 writer 的 scope／世代提交檢查，並調整既有 quota 刷新的網路／DB 鎖定界線；測試 A 回應晚於 B 不覆寫記憶體／DB／UI，慢速其他 provider 不阻塞重置券日誌與期限判斷，其他 provider 原刷新行為保持相容。
- [x] 2.6 將手動使用接上共用協調，確認開啟時捕捉 accountScope 並由 App 傳 expectedAccountScope；測試確認期間換帳號取消不送出、快取未確認時停用、既有取消／四種 API 結果及 unknown 阻擋仍正確。
- [x] 2.7 建立自動化專用有型別查詢結果，區分零張、unsupported、no_auth、暫時錯誤與無效回應；測試缺欄位不默認零值、401 不靜默改帳號或更新 token，憑證更新後重新查詢並維持去重。

## 3. 背景生命週期與每日提醒

- [x] 3.1 建立 Rust 重置券背景協調模組並接入 app setup、設定儲存及 Codex 更新；以假時鐘驗證最長 10 秒判斷、正常情況五分鐘補查、同帳號查詢合併及服務端退避時延後，不阻塞 UI／其他 provider。
- [x] 3.2 實作每日時間觸發、啟動／喚醒補查及持久化至多一次提醒；測試 09:00、中午啟動、同日重啟／改時間、零張記錄、跨日、時區及時鐘倒退不重複提交。
- [x] 3.3 接上既有原生通知與 zh-TW／en-US 到期文案；以假通知測試今日／未來張數、自動使用開／關／暫停及已提交不等於已讀，權限拒絕或提交失敗不影響消耗。
- [x] 3.4 實作每日 GET 有界重試，持久保存次數／下次時間及端點退避；測試三次上限跨重啟不重設、其他背景來源不繞過退避、跨午夜不重送舊日提醒、離線不誤報零券，認證／格式錯誤不無限重試。
- [x] 3.5 接上重置券自身開關、帳號條件及設定版本閘門，獨立於平台監控；測試關閉儲存先完成時拒絕舊版本新請求、進行中仍保存結果，重新啟用只重建未開始工作且不重開終止批次。

## 4. 到期前自動使用

- [x] 4.1 實作到期前 60 秒的最新資料查詢與 POST 前重驗；測試 -61 秒不執行、-60 秒可執行、0 秒不開始新請求、查詢跨過期限及喚醒後已過期不補用。
- [x] 4.2 實作單張／全數同效期的選券資格檢查；以假 API 測試混合效期、未知效期與總數／明細不一致均暫停且回報原因，不從陣列順序推論服務端排序。
- [x] 4.3 實作同批次逐次處理、上限與終止紀錄；測試每次 reset 後重查減少才繼續，nothing_to_reset／no_credit／already_redeemed／錯誤／不明即停止，重啟或下一輪不重開終止批次。
- [x] 4.4 接上持久化到期 GET 預算、退避及顯式最多十秒的整體 HTTP timeout，包含回應本文；以假時鐘測試重試不越過區間、重啟不重設次數及退避超過期限即停止。
- [x] 4.5 為 consume 設定 request-scoped 禁用 redirect 與不重送政策；以隔離 HTTP server 驗證鎖定 ureq 版本下 3xx／逾時／斷線只收到一筆 POST，5xx／解析失敗分類 unknown，明確拒絕與送出前本地失敗不誤標 reset。

## 5. 設定頁、狀態與用量同步

- [x] 5.1 依 sessionhub-minimal-ui 加入重置券設定分區及翻譯與通知語系持久化同步；以設定頁驗收四種開關組合、時間／天數輸入、關閉提醒保留值、儲存前不生效及固定一分鐘說明。
- [x] 5.2 新增唯讀狀態 command 與 scope／世代事件，僅由 App.tsx 呼叫及訂閱；驗證過期回應被丟棄、尚未到觸發點顯示等待而非暫停、下次時間可空、最近結果與登入／重新整理恢復提示正確，UI 不直接接觸憑證。
- [x] 5.3 將消耗完成與失敗接回既有 Codex 快照刷新管線；以假後端整合驗證目前帳號的 Dashboard／狀態列同步更新，舊帳號完成時僅更新原日誌，提醒關閉仍可查看自動結果且無需重新開頁。

## 6. 整合驗收與規格核對

- [x] 6.1 以隔離資料目錄與假 API 完成背景工作流程演練：提醒、到期使用、獨立於前端畫面的執行、重啟去重、切換帳號、休眠時間跳躍及結果不明阻擋；搭配設定頁 DOM 與 loopback HTTP 驗證，過程不消耗真實重置券。實際系統匣及 OS 通知驗收由 6.2 涵蓋。
- [ ] 6.2 在 Windows 安裝版驗證 SessionHub 原生通知識別、通知權限／系統設定及隱藏至系統匣行為；記錄可觀察的提交結果，確認畫面與通知均不承諾退出／休眠期間執行。
- [x] 6.3 執行 `bun run build`、於 `src-tauri` 執行 `cargo test`，以及 `openspec validate add-codex-reset-credit-automation --strict`；確認全部通過、規格與實際行為一致，更新任務完成狀態。

## 驗證紀錄（2026-10-30）

- Rust 全套（使用回饋修正後）：`cargo test --quiet`，358 通過、6 項原有手動／外部服務測試略過，無失敗。
- 前端相關：`bun test tests/resetCreditSettings.test.ts tests/ResetCreditSettings.test.tsx tests/settingsSaveEffects.test.ts tests/codexResetConfirmation.test.ts`，7 通過。
- `bun run build` 與 `openspec validate add-codex-reset-credit-automation --strict` 通過。
- `bun run lint` 通過；僅保留 ThemeProvider 與 AgentsConfigView 的兩項既有 hook dependency 警告。
- 先前額外執行全套 `bun test tests`：57 通過、1 失敗。`PlansSpecsView.test.tsx:60` 在 HEAD `79ed936` 即存在測試未傳 `loadingSection` 卻預期背景載入 status 的不一致；隔離重跑同樣失敗，相關元件與測試均無本次變更。
- 6.2 尚待 Windows 安裝版人工驗收：目前沒有可操作桌面／原生通知的工具。假通知、DOM 與 loopback HTTP 已驗證，未使用真實帳號消耗重置券；不以此代替 OS 安裝版驗收。

## 7. 使用回饋修正

- [x] 7.1 將重置券改為獨立 card、解除平台監控耦合；以 DOM 與假 API 驗證平台監控關閉時提醒／自動使用仍有效，未開啟功能顯示已關閉。
- [x] 7.2 分別顯示提醒與自動使用的暫停原因，修正錯誤 fallback 與過期查詢覆寫新狀態；以狀態與 DOM 測試驗證。
- [x] 7.3 設定相關阻塞工作改為背景執行並序列化保存及後置效果，僅變更相關欄位才重啟 watcher／工具／原生視窗；以效果判斷、通知不持設定鎖及設定持久化測試驗證，完成 lint 與 build。
