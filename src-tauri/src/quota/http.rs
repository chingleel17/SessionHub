//! quota 模組共用的 HTTP 呼叫輔助
//!
//! ureq 3.x 有兩處行為與 2.x 不同，皆於此統一處理：
//!
//! 1. 預設將 4xx/5xx 視為 Error::StatusCode(code)，且錯誤中不再攜帶 response，
//!    導致無法讀取 429 回應的 Retry-After header。此處關閉該行為，
//!    改以 ApiOutcome 明確分類狀態碼，讓呼叫端必須處理每一種結果。
//! 2. 預設 TLS provider 為 Rustls、root certs 為 bundled WebPki。
//!    2.x 使用 native-tls 搭配平台憑證庫，為維持企業環境（自訂 CA、MITM proxy）
//!    的連線行為，此處明確指定 NativeTls 與 PlatformVerifier。

use std::sync::OnceLock;
use std::time::Duration;

use ureq::http::Response;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
use ureq::{Agent, Body};

/// HTTP 回應的分類結果
///
/// 呼叫端以 match 處理，避免將 4xx/5xx 的錯誤內容誤當成正常結果解析
pub(crate) enum ApiOutcome {
    /// 2xx 成功回應
    Success(Response<Body>),
    /// 401 或 403，代表憑證無效或被拒絕
    Unauthorized,
    /// 429 限流，附帶 Retry-After 秒數（若伺服器有提供）
    RateLimited { retry_after_seconds: Option<u64> },
    /// 其他非成功狀態碼
    UnexpectedStatus(u16),
}

/// 共用 agent，統一 TLS 與狀態碼處理設定
fn agent() -> &'static Agent {
    static AGENT: OnceLock<Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        let config = Agent::config_builder()
            .http_status_as_error(false)
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::NativeTls)
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .build();
        Agent::new_with_config(config)
    })
}

/// 建立 GET request builder，可選設定整體逾時
pub(crate) fn get(
    url: &str,
    timeout: Option<Duration>,
) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
    let builder = agent().get(url);
    match timeout {
        Some(duration) => builder.config().timeout_global(Some(duration)).build(),
        None => builder,
    }
}

/// 建立 POST request builder，可選設定整體逾時
pub(crate) fn post(
    url: &str,
    timeout: Option<Duration>,
) -> ureq::RequestBuilder<ureq::typestate::WithBody> {
    let builder = agent().post(url);
    match timeout {
        Some(duration) => builder.config().timeout_global(Some(duration)).build(),
        None => builder,
    }
}

/// 依狀態碼將回應分類
pub(crate) fn classify(response: Response<Body>) -> ApiOutcome {
    let status = response.status().as_u16();
    match status {
        200..=299 => ApiOutcome::Success(response),
        401 | 403 => ApiOutcome::Unauthorized,
        429 => {
            let retry_after_seconds = response
                .headers()
                .get("Retry-After")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok());
            ApiOutcome::RateLimited {
                retry_after_seconds,
            }
        }
        other => ApiOutcome::UnexpectedStatus(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 實際連線驗證 native-tls 與狀態碼分流；需要外部網路，故預設不執行
    ///
    /// 執行方式：cargo test -- --ignored tls_and_status_classification
    #[test]
    #[ignore]
    fn tls_and_status_classification_over_real_https() {
        let ok_response = get("https://api.github.com/zen", Some(Duration::from_secs(10)))
            .header("User-Agent", "session-hub-test")
            .call()
            .expect("HTTPS 請求應成功建立連線");
        assert!(
            matches!(classify(ok_response), ApiOutcome::Success(_)),
            "200 回應應分類為 Success"
        );

        let missing_response = get(
            "https://api.github.com/this-path-does-not-exist",
            Some(Duration::from_secs(10)),
        )
        .header("User-Agent", "session-hub-test")
        .call()
        .expect("404 不應回傳 Err，http_status_as_error 需為 false");
        assert!(
            matches!(
                classify(missing_response),
                ApiOutcome::UnexpectedStatus(404)
            ),
            "404 回應應分類為 UnexpectedStatus(404)"
        );
    }
}
