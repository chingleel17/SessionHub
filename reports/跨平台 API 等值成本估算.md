# 用本機紀錄估算 API 等值成本

SessionHub 可用本機 session 持久化的 token、usage counter 或成本欄位，配合版本化的公開 API 單價，計算「若這些用量按公開 API 價格計費」的美元等值估算。**這個估算不是 Claude、OpenAI/Codex、GitHub Copilot 或 OpenCode 使用者的月費訂閱實際帳單，也不會把月費分攤到單一 session**；供應商方案、折扣、用量抵免、服務路由與實際計價條件均可能不同。實作原則是保存靜態價格版本、模型價格未知就不猜，並依各平台持久化資料能力設定 coverage；四者中 Copilot checkpoint、Codex 累積 counter 與 OpenCode persisted cost 最適合作為事後來源，而 Claude 必須逐筆檢查 usage 完整性。

## 四種本機來源的可用欄位決定估算上限

| 平台／來源 | 可讀欄位與估價方式 | 目前限制 |
|---|---|---|
| Claude Code JSONL | assistant `message.usage.input_tokens`、`output_tokens`、`cache_creation_input_tokens`、`cache_read_input_tokens`；`message.model`、`message.id`、外層 `timestamp`。按模型公開單價與可用的 cache 寫入 TTL 細分估價。 | 單筆估價至少需有效 message usage、input/output tokens、timestamp 與 message ID；模型缺失仍可保存 token 用量，但不能估美元。缺 cache 5m/1h breakdown 時可用 cache creation 合計估算，但 TTL 價格拆分不精確。([Claude pricing](https://platform.claude.com/docs/en/about-claude/pricing)、[Prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching)) |
| OpenAI Codex rollout JSONL | `session_meta` 的 `model_provider`、`turn_context` 的 `model`、`event_msg` 內 `token_count.info.total_token_usage`，包括 input、cached input、output、reasoning output、total。將累積 counter 轉成增量後乘 OpenAI 公開單價。 | Counter 是累積值，需要相鄰去重與處理 reset；只有已知模型且所有使用類別可計價才估美元，未知模型不估。Session rollout 的 model/provider 會隨上下文更新，解析時須保留當時有效的值。([Codex protocol](https://github.com/openai/codex/blob/53446f90a56692dede3c8f413e8d486a6adb77b5/codex-rs/protocol/src/protocol.rs)、[OpenAI pricing](https://platform.openai.com/docs/pricing)) |
| GitHub Copilot SDK | 持久化 `session.usage_checkpoint.data.totalNanoAiu` 或每模型 `modelMetrics[*].totalNanoAiu`，可直接按 `nano / 1e9 * US$0.01` 換算。checkpoint 中亦有 `totalPremiumRequests`，不是美元價格。 | `assistant.usage` 是 ephemeral，resume 不會重播；`requests.cost` 表示 Premium Request multiplier/cost，不是美元，也不是 AI Credits。缺少 `totalNanoAiu` 的舊 session 無法從 token count 或 Premium Request 數倒推同一筆 AI Credit 消耗。([Usage and billing](https://github.com/github/copilot-sdk/blob/main/docs/features/usage-and-billing.md)、[Streaming events](https://github.com/github/copilot-sdk/blob/main/docs/features/streaming-events.md)、[GitHub billing](https://docs.github.com/en/copilot/concepts/billing-and-usage/individuals/billing)) |
| OpenCode session message | 持久化 assistant message 已帶 token usage 與 `cost`；採用該筆 persisted cost 作為估算，不重新套價。保存 model ID 與 `providerID`，可辨識同名模型在不同 provider 下的差異。 | 數值是 OpenCode 寫入的 persisted estimate，仍不是供應商結算帳單；其準確度受當時模型/provider 定價資料與 provider 回報方式影響。([OpenCode session schema](https://github.com/anomalyco/opencode/blob/dev/packages/schema/src/v1/session.ts)) |

一般 token 估價公式為各類 token 數乘每百萬 token 單價後除以 1,000,000；有快取價格時必須將一般 input 與 cached input 分開，並將 cache write/read、output 各自按其價格計算。SessionHub 本輪靜態快照版本為 `anthropic-api-pricing-2026-09-24` 與 `openai-api-pricing-2026-09-24`；價格表應帶版本／查價日期，既有 event 使用其價格版本，避免公開價格更新後回溯改寫歷史數字。OpenAI 價格依模型、context 長度與服務模式而異，Anthropic 亦可能受 fast mode、data residency 等因素影響；本機若未保存相應計價條件，只能呈現標準公開價格對照的估算。([OpenAI API pricing](https://platform.openai.com/docs/pricing)、[Claude pricing](https://platform.claude.com/docs/en/about-claude/pricing))

## Claude partial 必須按缺漏欄位區分原因

Claude assistant record 的確切 incomplete 條件為：缺少 `message.usage`、`message.usage.input_tokens`、`message.usage.output_tokens`、外層 `timestamp`，或 `message.id`。以上任一缺漏，該筆無法構成可去重且可計價的完整 usage event，coverage 標示 partial；不能把缺欄當 0。`message.model` 缺失則只影響估價：token 數與來源事件若齊全仍可保存，但因無法匹配可靠單價，美元估算留空。

`cache_creation_input_tokens` 與 `cache_read_input_tokens` 是分開計價的 usage；若有 `cache_creation.ephemeral_5m_input_tokens`／`ephemeral_1h_input_tokens`，應各按 5 分鐘與 1 小時 cache write 單價估算。若只有 cache creation 合計而沒有 TTL breakdown，總 token 用量仍可估，但只能以有記錄的分項或明確的近似策略計價，並標記估價不精確；缺 breakdown 本身不構成 usage identity 缺漏。Anthropic 文件說明 cache write 價格依 5m/1h 時長不同，因此無法從合計反推出各自數量。([Claude pricing](https://platform.claude.com/docs/en/about-claude/pricing)、[Prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching))

Claude thinking token 若由 `output_tokens_details.thinking_tokens` 取得，它是 `output_tokens` 的子集合，**不得再額外加到 output 計價**。assistant message 的多次更新依 `message.id` 去重，保留同一訊息最新且資訊完整的 usage，不能把 streaming／增量式重複紀錄逐列累加。既有 JSONL 可重掃，補回目前仍存在於檔案中的欄位、重建 event、更新 parser 或使用新增價格版本；但舊檔若原本就沒有 message usage、input/output 數、timestamp 或 message ID，重掃不能創造原始紀錄中不存在的事實，也不能補回被截斷或已刪除的內容。缺 model 可重新檢查原始 message 是否存在；若原始檔也沒有 model，則無法恢復該筆美元估價。

## Codex rollout 用累積 counter 去重再套 API 價格

Codex rollout 的辨識流程以 `session_meta` 保存 `model_provider`，以 `turn_context.model` 更新當下模型，再讀 `event_msg` 且 payload type 為 `token_count` 的 `info.total_token_usage`。官方 protocol 對 `TokenUsage` 提供 `input_tokens`、`cached_input_tokens`、`output_tokens`、`reasoning_output_tokens`、`total_tokens` 等欄位；`last_token_usage` 是前次增量資訊，而 session 紀錄的累積 total counter 需按順序差分。相同 counter 的重複快照產生零差額而略過；若新的 `total_tokens` 小於前次值，視為 counter reset，從新的 counter 起點納入新用量，不可做負增量。([Codex protocol](https://github.com/openai/codex/blob/53446f90a56692dede3c8f413e8d486a6adb77b5/codex-rs/protocol/src/protocol.rs))

價格計算時，cached input 是 input 的子集合：一般 input 數量應扣除 cached input，再將 cached input 乘快取輸入單價，避免重複計價。`reasoning_output_tokens` 是 `output_tokens` 的子集合，僅供分析，不能加在 output 之外再次收費。快取寫入欄位若存在但沒有對應價格支援，也不應默認套用普通 input 價格。當 model 不在版本化價格表時保留 token 與 provider 資訊、估算美元為空；不能用最相近型號代替或以猜測單價補值。([OpenAI API pricing](https://platform.openai.com/docs/pricing))

## Copilot 用 checkpoint 換美元，不用 Premium Request 推價

Copilot SDK 文件區分 ephemeral per-call `assistant.usage` 與持久化 session usage：前者有 model、input/output tokens、`cost` 等，但只即時發送且 resume 不 replay；其中 `cost` 是 Premium Request multiplier/cost。SDK 另提供累積 `totalNanoAiu` 與每模型 `totalNanoAiu`；依 GitHub AI Credit 1 credit = US$0.01，估算式為 `totalNanoAiu / 1,000,000,000 * 0.01`，對應 checkpoint 中現存的 AI Credit 成本，並非按 OpenAI/Anthropic 等公開 API token 單價重估。`requests.cost` / `totalPremiumRequests` 均是請求計量，不等於美元或 AI Credits。([Usage and billing](https://github.com/github/copilot-sdk/blob/main/docs/features/usage-and-billing.md)、[GitHub billing](https://docs.github.com/en/copilot/concepts/billing-and-usage/individuals/billing))

本機應優先將持久化 `session.usage_checkpoint.data.totalNanoAiu` 作為 session 合計來源；若合計缺失但 per-model `totalNanoAiu` 存在，才按模型彙總，避免總計與分項同時計入。不能將 shutdown 的 `modelMetrics` token summary 疊加到 persisted `assistant.message` 的 `outputTokens`，否則同一產出重複計數；shutdown summary 可供補充診斷，但 checkpoint 才是可續接的累積成本來源。舊 session 若沒有 `totalNanoAiu`，僅有 token 欄位或 requests multiplier 時，無法可靠還原 AI Credit，更不能聲稱是實際帳單金額。([Streaming events](https://github.com/github/copilot-sdk/blob/main/docs/features/streaming-events.md))

## OpenCode 保留原生 persisted cost 與 provider 身分

OpenCode assistant message schema 將 usage/cost 等資訊持久化，因此 SessionHub 應採用 message 上已保存的 `cost` 作為 persisted estimate，而非用新價目表重新推算並覆蓋。估算紀錄並列保存 `model` 與 `providerID`：只存模型名稱會遺失實際 provider 的辨識線索，因相同或類似 model ID 可能走不同 provider 與定價。這個來源比只靠 token 重新估價更接近當時 OpenCode 記錄的 estimate，仍需在 UI 與匯出中標示為估算、非發票或訂閱帳單。([OpenCode session schema](https://github.com/anomalyco/opencode/blob/dev/packages/schema/src/v1/session.ts))

## 結論

本輪採用的策略是：**版本化靜態價格表、未知價格不猜、OpenCode persisted cost、Copilot checkpoint、Codex persisted counter、Claude 實際 coverage 檢查**。這讓可估值事件保留來源可追溯性，也讓「有 token 但無法估價」與「缺 usage 資料」成為不同狀態，而不是用假精確美元數填滿報表。

跨平台比較應理解為估算口徑對照，不是平台帳單比較：Claude/Codex 主要依本機 token counter 對照公開 API 單價，Copilot 使用其持久化 AI Credit 成本再換美元，OpenCode 則採原生持久化 estimate。將來源類型、價格版本與 coverage 一併呈現，才能知道數字代表什麼、何時不該被解讀成實際支出。
