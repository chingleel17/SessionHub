use serde::{Deserialize, Serialize};

pub(crate) const MIXED_TEXT_ESTIMATOR_VERSION: &str = "mixed-text-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileContentMetrics {
    pub(crate) character_count: u64,
    pub(crate) estimated_tokens: u64,
    pub(crate) estimator_version: String,
}

/// 以 Unicode scalar value 計算字元數，並估算混合 Markdown 內容的 token 規模。
pub(crate) fn estimate_mixed_text(content: &str) -> FileContentMetrics {
    let character_count = content.chars().count() as u64;
    let mut estimated_tokens = 0u64;
    let mut non_whitespace_run = 0u64;

    for character in content.chars() {
        if is_cjk(character) {
            estimated_tokens += 1;
            estimated_tokens += ceil_divide_by_four(&mut non_whitespace_run);
        } else if character.is_whitespace() {
            estimated_tokens += ceil_divide_by_four(&mut non_whitespace_run);
            estimated_tokens += 1;
        } else {
            non_whitespace_run += 1;
        }
    }
    estimated_tokens += ceil_divide_by_four(&mut non_whitespace_run);

    FileContentMetrics {
        character_count,
        estimated_tokens,
        estimator_version: MIXED_TEXT_ESTIMATOR_VERSION.to_string(),
    }
}

fn ceil_divide_by_four(value: &mut u64) -> u64 {
    let result = (*value + 3) / 4;
    *value = 0;
    result
}

fn is_cjk(character: char) -> bool {
    matches!(
        character as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2FA1F
            | 0x30000..=0x323AF
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_empty_content_as_zero() {
        let metrics = estimate_mixed_text("");

        assert_eq!(metrics.character_count, 0);
        assert_eq!(metrics.estimated_tokens, 0);
        assert_eq!(metrics.estimator_version, MIXED_TEXT_ESTIMATOR_VERSION);
    }

    #[test]
    fn estimates_english_by_four_unicode_scalars() {
        let metrics = estimate_mixed_text("abcdefgh");

        assert_eq!(metrics.character_count, 8);
        assert_eq!(metrics.estimated_tokens, 2);
    }

    #[test]
    fn counts_traditional_chinese_as_one_token_per_character() {
        let metrics = estimate_mixed_text("繁體中文");

        assert_eq!(metrics.character_count, 4);
        assert_eq!(metrics.estimated_tokens, 4);
    }

    #[test]
    fn includes_markdown_whitespace_and_code() {
        let metrics = estimate_mixed_text("# 標題\nconst value = 42;");

        assert_eq!(metrics.character_count, 22);
        assert!(metrics.estimated_tokens > 0);
    }

    #[test]
    fn counts_emoji_as_unicode_scalars() {
        let metrics = estimate_mixed_text("Hi 👋");

        assert_eq!(metrics.character_count, 4);
        assert_eq!(metrics.estimated_tokens, 3);
    }

    #[test]
    fn repeated_estimation_is_deterministic() {
        let content = "# AGENTS\n請先閱讀 README.md";

        assert_eq!(estimate_mixed_text(content), estimate_mixed_text(content));
    }
}
