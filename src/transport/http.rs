use super::SayaTransport;
use crate::debug::debug_log;
use reqwest::blocking::Client;
use std::time::Duration;

pub struct HttpTransport;

impl SayaTransport for HttpTransport {
    fn health(&self, base_url: &str, timeout: Duration, debug: bool) -> Result<String, String> {
        let url = format!("{}/health", base_url.trim_end_matches('/'));
        debug_log(
            debug,
            &format!("GET {url} timeout_ms={}", timeout.as_millis()),
        );
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| format!("failed to build http client: {e}"))?;
        let response = client
            .get(&url)
            .send()
            .map_err(|e| format!("health request failed: {e}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|e| format!("failed to read health response: {e}"))?;
        debug_log(
            debug,
            &format!("health_response status={} body={}", status, body),
        );
        if !status.is_success() {
            return Err(format!("health endpoint returned {status}: {body}"));
        }
        Ok(body)
    }

    fn chat(
        &self,
        base_url: &str,
        message: &str,
        timeout: Duration,
        auth_token: Option<&str>,
        debug: bool,
    ) -> Result<String, String> {
        debug_log(
            debug,
            &format!(
                "chat dispatch base_url={} timeout_ms={} authorization={} message_len={}",
                base_url,
                timeout.as_millis(),
                if auth_token.is_some() {
                    "Bearer token-present"
                } else {
                    "none"
                },
                message.len()
            ),
        );
        Ok(format!("chat stub via {base_url}: {message}"))
    }
}
