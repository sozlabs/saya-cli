use crate::config::{CliConfig, OutputFormat};
use crate::contracts::ConversationContext;
use crate::credentials::Credentials;
use crate::transport::{ChatRequest, ChatResult, SayaTransport};
use serde::Serialize;

fn render(config: &CliConfig, command: &str, payload: &str) -> String {
    let payload_json = match serde_json::to_string(payload) {
        Ok(value) => value,
        Err(_) => "\"\"".to_string(),
    };
    match config.output_format {
        OutputFormat::Text => payload.to_string(),
        OutputFormat::Json => format!(
            "{{\"command\":\"{}\",\"ok\":true,\"output\":{}}}",
            command, payload_json
        ),
    }
}

#[derive(Serialize)]
struct ChatJsonOutput<'a> {
    command: &'static str,
    ok: bool,
    conversation_id: &'a str,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<&'a str>,
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
    credentials: &mut Credentials,
) -> Result<String, String> {
    if credentials.access_token.is_none() {
        return Err("missing access token: set credentials token before chat".to_string());
    }
    let conversation_id = match &config.conversation_id {
        Some(value) => Some(value.as_str()),
        None => credentials.conversation_id.as_deref(),
    };
    let context = ConversationContext {
        session_id: config.session_id.clone(),
        tenant_id: config.tenant_id.clone(),
        actor_id: config.actor_id.clone(),
        channel_id: config.channel_id.clone(),
    };
    let request = ChatRequest {
        base_url: config.base_url.clone(),
        message: message.to_string(),
        conversation_id: conversation_id.map(|value| value.to_string()),
        context,
        timeout: config.timeout,
        auth_token: credentials.access_token.clone(),
        debug: config.debug,
        output_format: config.output_format,
        stream_max_retries: config.stream_max_retries,
        stream_retry_initial_ms: config.stream_retry_initial_ms,
        stream_retry_max_ms: config.stream_retry_max_ms,
    };
    let out = transport.chat(&request)?;
    credentials.conversation_id = Some(out.conversation_id.clone());
    format_chat_output(config, &out)
}

fn format_chat_output(config: &CliConfig, out: &ChatResult) -> Result<String, String> {
    if let Some(w) = out.warning.as_deref() {
        eprintln!("[saya] {w}");
    }
    match config.output_format {
        OutputFormat::Text => Ok(out.text.clone()),
        OutputFormat::Json => {
            let payload = ChatJsonOutput {
                command: "chat",
                ok: out.warning.is_none(),
                conversation_id: out.conversation_id.as_str(),
                text: out.text.as_str(),
                warning: out.warning.as_deref(),
            };
            serde_json::to_string(&payload).map_err(|e| format!("json output: {e}"))
        }
    }
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
                .map_err(|_| "lock poisoned".to_string())?
                .push(format!("health:{base_url}"));
            Ok("ok".to_string())
        }

        fn chat(&self, request: &ChatRequest) -> Result<crate::transport::ChatResult, String> {
            self.calls
                .lock()
                .map_err(|_| "lock poisoned".to_string())?
                .push(format!(
                    "chat:{}:{}:{}:{}:{}",
                    request.base_url,
                    request.message,
                    request
                        .conversation_id
                        .as_deref()
                        .map_or("none", |value| value),
                    request.context.channel_id,
                    if request.auth_token.is_some() {
                        "auth"
                    } else {
                        "noauth"
                    }
                ));
            Ok(crate::transport::ChatResult {
                conversation_id: "generated-cid".to_string(),
                text: "done".to_string(),
                warning: None,
            })
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
            conversation_id: None,
            session_id: "s".to_string(),
            tenant_id: "t".to_string(),
            actor_id: "u".to_string(),
            channel_id: "terminal".to_string(),
            stream_max_retries: 3,
            stream_retry_initial_ms: 500,
            stream_retry_max_ms: 8000,
        }
    }

    fn test_config_resume() -> CliConfig {
        let mut cfg = test_config();
        cfg.conversation_id = Some("resume-cid".to_string());
        cfg
    }

    #[test]
    fn health_routes_through_transport() {
        let transport = MockTransport::default();
        let result = match run_health(&transport, &test_config()) {
            Ok(value) => value,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(result, "ok");
        let calls = match transport.calls.lock() {
            Ok(value) => value,
            Err(_) => panic!("lock poisoned"),
        };
        assert_eq!(calls.as_slice(), ["health:http://127.0.0.1:3010"]);
    }

    #[test]
    fn chat_routes_through_transport() {
        let transport = MockTransport::default();
        let mut creds = Credentials {
            access_token: Some("secret".to_string()),
            conversation_id: None,
        };
        let result = match run_chat(&transport, &test_config(), "hello", &mut creds) {
            Ok(value) => value,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(result, "done");
        let calls = match transport.calls.lock() {
            Ok(value) => value,
            Err(_) => panic!("lock poisoned"),
        };
        assert_eq!(
            calls.as_slice(),
            ["chat:http://127.0.0.1:3010:hello:none:terminal:auth"]
        );
        assert_eq!(creds.conversation_id.as_deref(), Some("generated-cid"));
    }

    #[test]
    fn chat_uses_conversation_id_from_config_when_present() {
        let transport = MockTransport::default();
        let mut creds = Credentials {
            access_token: Some("secret".to_string()),
            conversation_id: Some("old-cid".to_string()),
        };
        let result = match run_chat(&transport, &test_config_resume(), "hello", &mut creds) {
            Ok(value) => value,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(result, "done");
        let calls = match transport.calls.lock() {
            Ok(value) => value,
            Err(_) => panic!("lock poisoned"),
        };
        assert_eq!(
            calls.as_slice(),
            ["chat:http://127.0.0.1:3010:hello:resume-cid:terminal:auth"]
        );
    }

    #[test]
    fn chat_fails_without_token() {
        let transport = MockTransport::default();
        let mut creds = Credentials {
            access_token: None,
            conversation_id: None,
        };
        let result = run_chat(&transport, &test_config(), "hello", &mut creds);
        assert!(result.is_err());
        let calls = match transport.calls.lock() {
            Ok(value) => value,
            Err(_) => panic!("lock poisoned"),
        };
        assert_eq!(calls.len(), 0);
    }
}
