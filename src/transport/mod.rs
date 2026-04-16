use std::time::Duration;

pub mod http;

pub trait SayaTransport {
    fn health(&self, base_url: &str, timeout: Duration, debug: bool) -> Result<String, String>;
    fn chat(
        &self,
        base_url: &str,
        message: &str,
        timeout: Duration,
        auth_token: Option<&str>,
        debug: bool,
    ) -> Result<String, String>;
}
