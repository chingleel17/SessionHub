## Why

AGENTS.md、CLAUDE.md、Skills 與 Commands 等指示檔會直接占用 AI agent 的輸入脈絡，但目前使用者只能看到內容，無法快速判斷檔案規模或可能消耗的 token。SessionHub 需要一個不依賴 API key、可離線使用且清楚標示為近似值的共用估算能力。

## What Changes

- 掃描 AGENTS.md / CLAUDE.md、Skill 的 `SKILL.md` 與 Command 預覽檔時回傳 Unicode 字元數與本機估算 token 數，避免為每個畫面項目額外讀檔。
- 新增可獨立測試、可替換 profile 的 token 估算模組；第一版以 OpenAI 公開的英文「約 4 字元 = 1 token」經驗值為基準，再針對 Markdown、程式碼與中英混合內容採通用啟發式算法，不宣稱等同 Claude 或 GPT 的精確 tokenizer。
- 在指示檔樹狀清單的檔案項目最右側顯示約略 token 數，使用 `~850 tokens`、`~1.2k tokens` 等精簡格式。
- 選取檔案後，在內容路徑列顯示字元數與較完整的 token 估算資訊。
- 移除內容面板中獨占一列的編輯按鈕，將編輯／預覽與儲存操作移至右上角操作區；未選取檔案時不顯示這些操作。
- Skills 與 Commands 清單在每列最右側顯示相同 token 格式，開啟預覽後在內容路徑列顯示相同字元與 token 指標。
- MCP 不屬於文字指示檔，本階段不加入內容指標。

## Capabilities

### New Capabilities
- `agents-file-metrics`: 定義 AGENTS.md、CLAUDE.md、Skills 與 Commands 的字元統計、本機 token 估算、格式化、掃描資料契約與 UI 呈現。

### Modified Capabilities
- `agents-md-sync`: AGENTS.md / CLAUDE.md 掃描結果新增檔案內容指標。

## Impact

- 後端：`src-tauri/src/agents_config.rs` 的掃描流程、序列化型別與單元測試，並新增或抽出 token 估算模組。
- 前端：`src/types/index.ts`、`src/utils/buildTree.ts`、`src/components/ExplorerTree.tsx`、`src/components/AgentsConfigView.tsx`、i18n 與 Agents 相關 CSS。
- API：`AgentsMdScanResult` 的來源／目標，以及 `SkillEntry`、`CommandEntry` 將新增可選內容指標欄位；既有 Tauri command 名稱不變。
- 相依性：第一版不新增第三方 tokenizer、不呼叫 OpenAI／Anthropic API，也不需要使用者提供金鑰。
