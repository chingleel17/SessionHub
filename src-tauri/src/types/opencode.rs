use serde::Deserialize;

// ── OpenCode JSON 儲存格式（session/*.json / project/*.json）────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeProjectJson {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) worktree: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeSessionJsonTime {
    #[serde(default)]
    pub(crate) created: Option<i64>,
    #[serde(default)]
    pub(crate) updated: Option<i64>,
    #[serde(default)]
    pub(crate) archived: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeSessionJsonSummary {
    #[serde(default)]
    pub(crate) additions: Option<i64>,
    #[serde(default)]
    pub(crate) deletions: Option<i64>,
    #[serde(default)]
    pub(crate) files: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeSessionJson {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) directory: Option<String>,
    #[serde(default)]
    pub(crate) time: Option<OpencodeSessionJsonTime>,
    #[serde(default)]
    pub(crate) summary: Option<OpencodeSessionJsonSummary>,
}

// ── OpenCode Stats 解析相關型別 ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeTokens {
    #[serde(default)]
    pub(crate) input: Option<u64>,
    #[serde(default)]
    pub(crate) output: Option<u64>,
    #[serde(default)]
    pub(crate) reasoning: Option<u64>,
    #[serde(default, rename = "inputTokens")]
    pub(crate) input_tokens: Option<u64>,
    #[serde(default, rename = "outputTokens")]
    pub(crate) output_tokens: Option<u64>,
}

impl OpencodeTokens {
    pub(crate) fn effective_input(&self) -> u64 {
        self.input.or(self.input_tokens).unwrap_or(0)
    }
    pub(crate) fn effective_output(&self) -> u64 {
        self.output.or(self.output_tokens).unwrap_or(0)
    }
    pub(crate) fn effective_reasoning(&self) -> u64 {
        self.reasoning.unwrap_or(0)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeMessageTime {
    #[serde(default)]
    pub(crate) created: Option<i64>,
    #[serde(default)]
    pub(crate) completed: Option<i64>,
}

/// metadata.assistant 子物件（modelID、tokens）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeAssistantMeta {
    #[serde(default, alias = "modelID")]
    pub(crate) model_id: Option<String>,
    #[serde(default)]
    pub(crate) tokens: Option<OpencodeTokens>,
}

/// metadata 子物件
#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeMessageMetadata {
    #[serde(default)]
    pub(crate) time: Option<OpencodeMessageTime>,
    #[serde(default)]
    pub(crate) assistant: Option<OpencodeAssistantMeta>,
    /// 有些版本的 token 統計直接放在 metadata.usage
    #[serde(default)]
    pub(crate) usage: Option<OpencodeTokens>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeMessage {
    pub(crate) id: String,
    #[serde(default, alias = "sessionID")]
    pub(crate) session_id: Option<String>,
    #[serde(default)]
    pub(crate) role: String,
    #[serde(default)]
    pub(crate) time: Option<OpencodeMessageTime>,
    #[serde(default, alias = "modelID")]
    pub(crate) model_id: Option<String>,
    #[serde(default)]
    pub(crate) tokens: Option<OpencodeTokens>,
    #[serde(default)]
    pub(crate) metadata: Option<OpencodeMessageMetadata>,
}

impl OpencodeMessage {
    pub(crate) fn time(&self) -> Option<&OpencodeMessageTime> {
        self.time
            .as_ref()
            .or_else(|| self.metadata.as_ref()?.time.as_ref())
    }
    pub(crate) fn model_id(&self) -> Option<&str> {
        self.model_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                self.metadata
                    .as_ref()?
                    .assistant
                    .as_ref()?
                    .model_id
                    .as_deref()
                    .filter(|s| !s.is_empty())
            })
    }
    pub(crate) fn tokens(&self) -> Option<&OpencodeTokens> {
        self.tokens.as_ref().or_else(|| {
            self.metadata.as_ref().and_then(|m| {
                m.assistant
                    .as_ref()
                    .and_then(|a| a.tokens.as_ref())
                    .or(m.usage.as_ref())
            })
        })
    }
}

// ── Activity 相關型別 ────────────────────────────────────────────────────────

/// OpenCode message 檔案結構（只解析需要的欄位）
#[derive(Debug, Deserialize)]
pub(crate) struct OpenCodeMessageFile {
    pub(crate) role: Option<String>,
    pub(crate) finish: Option<String>,
    pub(crate) time: Option<OpenCodeMessageTime2>,
}

/// Activity 模組專用的 time 型別（與 stats 的 OpencodeMessageTime 不同）
#[derive(Debug, Deserialize)]
pub(crate) struct OpenCodeMessageTime2 {
    pub(crate) created: Option<i64>,
    pub(crate) completed: Option<i64>,
}
