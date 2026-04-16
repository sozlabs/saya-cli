use crate::config::{CliConfig, OutputFormat};
use crate::credentials::Credentials;
use crate::transport::SayaTransport;

fn render(config: &CliConfig, command: &str, payload: &str) -> String {
    match config.output_format {
        OutputFormat::Text => payload.to_string(),
        OutputFormat::Json => format!(
            "{{\"command\":\"{}\",\"ok\":true,\"output\":{}}}",
            command,
            serde_json::to_string(payload).unwrap_or_else(|_| "\"\"".to_string())
        ),
    }
}

pub fn run_version(config: &CliConfig) -> String {
    render(
        config,
        "version",
        &format!("saya-cli {}", env!("CARGO_PKG_VERSION")),
    )
}

pub fn run_health(transport: &dyn SayaTransport, config: &CliConfig) -> Result<String, String> {
    let out = transport.health(&config.base_url, config.timeout, config.debug)?;
    Ok(render(config, "health", &out))
}

pub fn run_chat(
    transport: &dyn SayaTransport,
    config: &CliConfig,
    message: &str,
    credentials: &Credentials,
) -> Result<String, String> {
    let out = transport.chat(
        &config.base_url,
        message,
        config.timeout,
        credentials.access_token.as_deref(),
        config.debug,
    )?;
    Ok(render(config, "chat", &out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OutputFormat;
    use crate::credentials::Credentials;
    use crate::transport::SayaTransport;
    use std::sync::Mutex;
    use std::time::Duration;

    #[derive(Default)]
    struct MockTransport {
        calls: Mutex<Vec<String>>,
    }

    impl SayaTransport for MockTransport {
        fn health(
            &self,
            base_url: &str,
            _timeout: Duration,
            _debug: bool,
        ) -> Result<String, String> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push(format!("health:{base_url}"));
            Ok("ok".to_string())
        }

        fn chat(
            &self,
            base_url: &str,
            message: &str,
            _timeout: Duration,
            _auth_token: Option<&str>,
            _debug: bool,
        ) -> Result<String, String> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push(format!("chat:{base_url}:{message}"));
            Ok("done".to_string())
        }
    }

    fn test_config() -> CliConfig {
        CliConfig {
            base_url: "http://127.0.0.1:3010".to_string(),
            timeout: Duration::from_secs(5),
            output_format: OutputFormat::Text,
            non_interactive: false,
            debug: false,
            config_dir: std::path::PathBuf::from(".saya"),
        }
    }

    #[test]
    fn health_routes_through_transport() {
        let transport = MockTransport::default();
        let result = run_health(&transport, &test_config()).expect("health should succeed");
        assert_eq!(result, "ok");
        let calls = transport.calls.lock().expect("lock poisoned");
        assert_eq!(calls.as_slice(), ["health:http://127.0.0.1:3010"]);
    }

    #[test]
    fn chat_routes_through_transport() {
        let transport = MockTransport::default();
        let creds = Credentials {
            access_token: Some("secret".to_string()),
        };
        let result =
            run_chat(&transport, &test_config(), "hello", &creds).expect("chat should succeed");
        assert_eq!(result, "done");
        let calls = transport.calls.lock().expect("lock poisoned");
        assert_eq!(calls.as_slice(), ["chat:http://127.0.0.1:3010:hello"]);
    }
}
