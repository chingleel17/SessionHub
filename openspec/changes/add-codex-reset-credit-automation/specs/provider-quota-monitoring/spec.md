## MODIFIED Requirements

### Requirement: 可於 App 內消耗 Codex 手動重置額度

QuotaOverview 的 Codex 重置額度標題前 SHALL 顯示使用重置按鈕，僅在 snapshot 有 `reset_credits` 時出現。可用次數為 0 或操作進行中時 SHALL 停用按鈕。按下按鈕 SHALL 先經應用程式共用確認對話框二次確認，不得單次點擊即送出消耗請求；Tauri IPC 由 App 元件發起。手動與自動使用 SHALL 共用同一帳號的消耗協調與結果刷新；自動使用依獨立設定授權，不套用此手動按鈕的逐次確認。

#### Scenario: 確認後使用一筆重置額度

- **WHEN** 使用者確認操作，且 Codex 額度監控已啟用、具有 ChatGPT 帳戶 ID 與可用重置額度
- **THEN** 後端先重新查詢可用重置額度，再以相同 Codex 憑證呼叫 `POST https://chatgpt.com/backend-api/wham/rate-limit-reset-credits/consume`
- **AND** 請求 JSON 包含新產生的 `redeem_request_id`，若有 account ID 則附帶 `ChatGPT-Account-Id` header
- **AND** 前端顯示後端回傳的 `reset`、`nothing_to_reset`、`no_credit` 或 `already_redeemed` 對應結果，完成後重新整理 Codex 額度與可用次數
- **AND** 取得帳號執行權後仍須重新驗證帳號與可用額度，不得因等待期間登入帳號改變而消耗另一帳號

#### Scenario: 取消確認或額度不可用

- **WHEN** 使用者取消確認，或可用重置額度為 0
- **THEN** 不送出消耗請求

#### Scenario: 重置請求失敗或結果不明

- **WHEN** 憑證無效、請求失敗、回應格式未知或網路逾時
- **THEN** 顯示錯誤或提示使用者重新整理確認結果
- **AND** 操作按鈕恢復可用狀態，並重新整理 Codex snapshot；不得自行宣稱已完成重置
- **AND** 結果不明尚未解除時，後端不得接受同一帳號任何新消耗；按鈕可回報需重新整理確認的狀態。僅在重新查詢確認原觀察券皆不可用，或其到期時間皆已過且沒有未知到期券時解除

#### Scenario: 自動使用進行中遇到手動使用

- **WHEN** 同一帳號已有自動消耗請求，使用者同時確認手動使用
- **THEN** 不並行送出第二筆消耗，回報處理中或等待既有請求完成
- **AND** 等待後若仍要執行，重新查詢並確認有可用券；已無可用券則不送出

#### Scenario: 僅有 OpenCode 的 OpenAI 登入可供查詢

- **WHEN** 目前 Codex 用量查詢採用既有 OpenCode OpenAI OAuth 備援來源
- **THEN** 手動與自動消耗皆使用相同憑證解析結果及同一 ChatGPT 帳號
- **AND** 無法取得可驗證的 ChatGPT 帳戶 ID 時拒絕消耗，顯示需登入的原因，不改用另一帳號的 Codex CLI 憑證
