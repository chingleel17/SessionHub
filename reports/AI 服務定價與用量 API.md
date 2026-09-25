# 釐清 AI 定價與 Session 用量

SessionHub 若要估算單一 session 的美元成本，最可行的方法是**在本機記錄模型與輸入、輸出及快取 token 數，再套用有版本與生效日期的官方公開價格表**。供應商的模型清單 API 不含價格；組織用量／成本 API 主要提供組織或時間區間彙總，無法直接對應 SessionHub 的本機 session ID。Copilot 官方明示 AI Credit 每點為 US$0.01，但這不是 token 單價；ChatGPT 個人訂閱與 Claude Pro／Max 也沒有公開的精確單一 session 成本 API。

## 靜態價格可供估算，模型清單 API 不提供價格

| 供應商 | 靜態公開定價 | 模型清單 API | 組織用量／成本 API | 個人或本機單一 session 可取得性 |
|---|---|---|---|---|
| GitHub Copilot | 個人帳單說明為 **1 AI Credit = US$0.01**，US$10 對應 1,000 點；這是點數金額換算，不是模型 token 單價。`[0.035, 0.055]` 未找到官方定義，不應作為換算率或價格。([個人帳單](https://docs.github.com/en/copilot/concepts/billing-and-usage/individuals/billing)) | 本次提供的官方來源未列可供定價的模型清單 API。 | Copilot Usage Metrics API 提供組織／企業使用情況報表，不等於個人點數或 session 成本。([REST API](https://docs.github.com/en/rest/copilot/copilot-usage-metrics)、[使用指標說明](https://docs.github.com/en/copilot/reference/copilot-usage-metrics)) | 未找到個人 AI Credit 餘額、session 用量或成本 REST／GraphQL API。Premium Request 模型乘數是配額消耗機制，不是美元換算率。([Premium Requests](https://docs.github.com/en/copilot/concepts/billing-and-usage/copilot-requests)) |
| Anthropic／Claude | 官方價格頁列 Claude Sonnet 5 為每百萬 token **輸入 US$2、輸出 US$10**；Claude Opus 5 為**輸入 US$5、輸出 US$25**。價格可能隨官方頁面更新，估算應保存查價日期與價格版本。([定價](https://platform.claude.com/docs/en/about-claude/pricing)、[模型總覽](https://platform.claude.com/docs/en/about-claude/models/overview)) | `GET /v1/models` 回傳模型識別、名稱與能力等資訊，**不含價格**；未找到公開的動態單價 endpoint。 | Claude Code Analytics API `GET /v1/organizations/usage_report/claude_code` 需 Anthropic 組織管理權限／`org:admin`，提供每日彙總的 actor、model、token、session 數與 estimated cost；另有 API Organization Usage／Cost 組織彙總 API。([Claude Code Analytics](https://platform.claude.com/docs/en/manage-claude/claude-code-analytics-api)、[API 參考](https://platform.claude.com/docs/en/api/admin/usage_report/retrieve_claude_code)、[Usage／Cost API](https://platform.claude.com/docs/en/manage-claude/usage-cost-api)) | Analytics 的 session 數並非本機 session ID 的逐筆帳單；個人 Pro／Max 不可使用組織 Analytics。Claude Pro／Max 訂閱與 API 結帳分開；若經 Bedrock、Vertex 或 Foundry 使用，帳務由雲端供應商處理。([Claude Code 費用說明](https://code.claude.com/docs/en/costs)) |
| OpenAI | 官方價格頁所列 `gpt-6-luna` Standard API 價格為每百萬 token **輸入 US$0.10、快取輸入 US$0.01、輸出 US$0.50**。此處只引用該列，不能推廣為其他模型價格。([API 定價](https://platform.openai.com/docs/pricing)) | `GET /v1/models` 提供 model ID 與基本 metadata，**不含價格**；未找到官方動態模型定價 endpoint。([模型清單](https://platform.openai.com/docs/api-reference/models/list)) | Admin API 提供 `GET /v1/organization/usage/completions` 與 `GET /v1/organization/costs`，依時間 buckets／組織維度彙總，不帶本機 SessionHub session ID。([Usage API](https://platform.openai.com/docs/api-reference/usage)、[API 參考](https://platform.openai.com/docs/api-reference)) | ChatGPT Plus／Pro 等訂閱沒有公開的 per-session token 成本 API；ChatGPT 訂閱與 API Platform 用量分開計費。([ChatGPT Plus 說明](https://help.openai.com/en/articles/6950777-what-is-chatgpt-plus)、[帳務說明](https://help.openai.com/en/articles/10421635)) |

## 組織彙總不能取代本機 session 帳本

組織 API 適合管理者核對期間總量、趨勢或費用，但資料粒度與識別方式不足以直接還原某個本機 session 的精確成本。Anthropic 的組織分析雖含 session 數與 estimated cost，仍不是可用本機 session ID 查詢的逐 session 帳單；OpenAI 的用量與成本 API 也是組織及時間 bucket 彙總。Copilot Usage Metrics 同樣屬組織／企業報表，而非個人 AI Credit 餘額或單一 session 成本。

## SessionHub 可用本機 token 與版本化價格估算

每個 session 至少記錄供應商、模型 ID、輸入 token、輸出 token、快取輸入 token（若有）、記錄時間，以及資料來源。以官方公開單價按相應 token 類別計算，再保留價格表版本、生效日期與查詢日期；若價格頁更新，舊 session 應繼續使用當時保存的價格版本，避免回溯重算造成帳目變動。只在來源明確提供相應 token 類別與價格時才計價；不要從模型清單 API 推導價格，也不要把 Copilot Premium Request 乘數當作美元單價。

Copilot 若可取得實際 AI Credit 消耗紀錄，可依官方的 **US$0.01／credit** 換算金額；目前研究結果未找到個人或 session 層級的公開 API，因此不能假設 SessionHub 能自動讀取該筆數。組織 Usage／Cost API 可作為總量校對，不能當作單一本機 session 的來源帳本。ChatGPT 個人訂閱與 Claude Pro／Max 則應標示為「無公開精確 session 成本 API」，避免將訂閱費錯配到特定 session。

## 結論

SessionHub 若要呈現單一 session 美元粗估，應以本機 token 計數和帶版本、生效日期的官方靜態價格為核心；組織 API 僅作彙總校對。Copilot 的 AI Credit 換算只適用於已取得的實際點數消耗，不能以 Premium Request 乘數或未定義的 `[0.035, 0.055]` 代替。個人訂閱方案目前不具備公開的精確 session 成本介面，產品介面應清楚標示估算或資料不可得，而非呈現看似精確的實際帳單。
