## Purpose

提供本機且不需 API 金鑰的指示檔規模統計，讓使用者能比較 Unicode 字元數與模型輸入 token 的近似占用，同時避免將估算值誤認為供應商帳務或精確 tokenizer 結果。

## ADDED Requirements

### Requirement: 指示檔內容指標
系統 SHALL 對可讀取的 UTF-8 指示檔計算 Unicode 字元數與大於或等於零的 token 估算值。token 估算 SHALL 完全在本機執行，不得要求 API key、傳送檔案內容至外部服務或阻斷既有掃描結果。

#### Scenario: 計算中英混合 Markdown
- **WHEN** 掃描到可讀取的 UTF-8 指示檔
- **THEN** 系統回傳該檔案的 Unicode 字元數與 token 估算值
- **AND** 空白、ASCII、CJK 字元、標點與程式碼內容皆納入估算

#### Scenario: 空白檔案
- **WHEN** 指示檔內容為空
- **THEN** 系統回傳 0 字元與 0 estimated tokens

#### Scenario: 無法計算內容指標
- **WHEN** 檔案存在但內容無法以 UTF-8 解碼或在計算時無法讀取
- **THEN** 系統仍回傳既有檔案 fingerprint 與同步狀態
- **AND** 該檔案的內容指標為 unavailable，不得使整體掃描失敗

### Requirement: 估算語意與可替換性
系統 SHALL 將 token 數標示為估算值，且估算行為 MUST 具版本識別，讓未來可在不改變消費端資料語意的情況下替換或新增模型專用估算方式。第一版 SHALL 使用不綁定特定供應商的 mixed-text profile，不得宣稱結果等同 Claude 或 GPT 的精確 token 數。

#### Scenario: 離線估算
- **WHEN** 裝置未連線網路且使用者未設定 OpenAI 或 Anthropic API key
- **THEN** 系統仍能產生相同內容的確定性 token 估算結果

#### Scenario: 重複估算
- **WHEN** 相同內容以相同 estimator 版本重複計算
- **THEN** 系統每次回傳相同的 token 估算值

### Requirement: 精簡 token 顯示格式
系統 SHALL 以 `~` 前綴呈現 token 估算值；小於 1,000 時顯示整數，大於或等於 1,000 時以最多一位小數的 `k` 單位顯示，並保留可取得完整估算值與字元數的方式。

#### Scenario: 小於一千 tokens
- **WHEN** 檔案估算為 850 tokens
- **THEN** 精簡文字顯示為 `~850 tokens`

#### Scenario: 千位 tokens
- **WHEN** 檔案估算為 1,240 tokens
- **THEN** 精簡文字顯示為 `~1.2k tokens`

### Requirement: 指示檔樹列顯示 token 估算
AGENTS.md 頁籤的樹狀清單 SHALL 在每個可選取檔案項目的最右側顯示精簡 token 估算文字，不得取代既有檔名、同步狀態或檔案類型資訊。群組節點與無可用指標的檔案不得顯示虛構數值。

#### Scenario: 一般檔案項目
- **WHEN** 某指示檔具有可用的 token 估算值
- **THEN** 該檔案項目最右側顯示如 `~1.2k tokens` 的次要文字
- **AND** 檔名空間不足時先截斷，token 文字仍保持可讀

#### Scenario: 來源與目標分開顯示
- **WHEN** AGENTS.md 與 CLAUDE.md 內容不同而展開為兩個檔案項目
- **THEN** 兩個項目各自顯示其內容對應的 token 估算值

#### Scenario: 指標 unavailable
- **WHEN** 某指示檔沒有可用內容指標
- **THEN** 該項目不顯示 token 文字
- **AND** 使用者仍可選取與操作該檔案

### Requirement: 選取檔案後顯示完整內容指標
選取 AGENTS.md 或 CLAUDE.md 後，內容標頭 SHALL 在檔案路徑同一列顯示 locale-aware 的完整字元數與 token 估算資訊；未選取檔案時不得顯示內容指標。

#### Scenario: 顯示選取檔案指標
- **WHEN** 使用者選取具有 3,456 個字元且估算為 1,240 tokens 的檔案
- **THEN** 內容標頭顯示格式化後的 3,456 字元與 `~1.2k tokens`

#### Scenario: 切換選取檔案
- **WHEN** 使用者由 AGENTS.md 切換選取同目錄但內容不同的 CLAUDE.md
- **THEN** 內容標頭與樹列反映 CLAUDE.md 自身的指標

### Requirement: 編輯操作整合至檔案操作區
AGENTS.md 頁籤 SHALL 移除內容區上方獨占一列的編輯操作，並將編輯／預覽與儲存操作放入既有右上檔案操作區。這些操作 SHALL 僅在選取檔案時顯示，且 SHALL 維持既有外部開啟、檔案總管顯示與同步操作能力。

#### Scenario: 尚未選取檔案
- **WHEN** 使用者開啟 AGENTS.md 頁籤但尚未選取檔案
- **THEN** 右上操作區不顯示編輯、預覽或儲存操作

#### Scenario: 進入與離開編輯模式
- **WHEN** 使用者選取檔案並點擊右上編輯操作
- **THEN** 內容區切換至既有編輯與預覽介面
- **AND** 右上操作切換為預覽與儲存

#### Scenario: 儲存編輯內容
- **WHEN** 使用者在編輯模式點擊右上儲存
- **THEN** 系統沿用既有安全寫入流程儲存內容並離開編輯模式
- **AND** 重新整理後的內容指標反映新內容

### Requirement: Skills 與 Commands 顯示相同內容指標
Skills 掃描 SHALL 對每列實際預覽的 `SKILL.md` 回傳內容指標，Commands 掃描 SHALL 對每列實際預覽的文字設定檔回傳內容指標。兩個頁籤 SHALL 在清單項目最右側顯示與 AGENTS.md 相同格式的 token 估算，並在開啟預覽後於內容路徑列顯示完整字元數與 token 估算。

#### Scenario: Skill 清單與預覽
- **WHEN** 某 Skill 的 `SKILL.md` 可讀取且具有內容指標
- **THEN** Skill 清單該列最右側顯示精簡 token 估算
- **AND** 開啟該 Skill 後，內容路徑列顯示同一檔案的字元數與 token 估算

#### Scenario: Command 清單與預覽
- **WHEN** 某 Command 的實際預覽檔可讀取且具有內容指標
- **THEN** Commands 清單該列最右側顯示精簡 token 估算
- **AND** 開啟該 Command 後，內容路徑列顯示同一檔案的字元數與 token 估算

#### Scenario: 同名 provider 資源合併
- **WHEN** 多個 provider 提供同名 Skill 或 Command 且 UI 合併為一列
- **THEN** 清單與預覽顯示的指標對應該列實際開啟的預覽檔
- **AND** 系統不得加總重複 provider 檔案的 token 數

#### Scenario: CLI-only 或無法讀取的項目
- **WHEN** Skill 或 Command 沒有可讀取的本機預覽檔
- **THEN** 清單不顯示虛構的 token 數
- **AND** 該項目的既有 discovery、預覽與同步狀態不得因此失敗
