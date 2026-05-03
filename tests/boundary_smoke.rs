use saya_cli::async_trait;
use saya_cli::commands::{run_chat, run_health};
use saya_cli::config::{CliConfig, OutputFormat};
use saya_cli::credentials::Credentials;
use saya_cli::transport::{ChatRequest, ChatResult, SayaTransport};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Default)]
struct MockTransport {
    calls: Mutex<Vec<String>>,
}

#[async_trait]
impl SayaTransport for MockTransport {
    async fn health(
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

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResult, String> {
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
        Ok(ChatResult {
            conversation_id: "generated".to_string(),
            text: "done".to_string(),
            warning: None,
        })
    }
}

fn test_config() -> CliConfig {
    CliConfig {
        base_url: "http://127.0.0.1:3010".to_string(),
        timeout: Duration::from_secs(2),
        output_format: OutputFormat::Text,
        non_interactive: true,
        debug: false,
        config_dir: PathBuf::from(".saya"),
        conversation_id: None,
        session_id: "s".to_string(),
        tenant_id: "t".to_string(),
        actor_id: "u".to_string(),
        channel_id: "terminal".to_string(),
        stream_max_retries: 3,
        stream_retry_initial_ms: 500,
        stream_retry_max_ms: 8000,
        allow_restricted_tools_opt_in: false,
    }
}

#[tokio::test]
async fn health_uses_saya_transport_shape() {
    let transport = MockTransport::default();
    let out = match run_health(&transport, &test_config()).await {
        Ok(value) => value,
        Err(err) => panic!("{err}"),
    };
    assert_eq!(out, "ok");
    let calls = match transport.calls.lock() {
        Ok(value) => value,
        Err(_) => panic!("lock poisoned"),
    };
    assert_eq!(calls.as_slice(), ["health:http://127.0.0.1:3010"]);
}

#[tokio::test]
async fn chat_uses_transport_instead_of_agent_runtime() {
    let transport = MockTransport::default();
    let mut creds = Credentials {
        access_token: Some("tok".to_string()),
        conversation_id: None,
    };
    let out = match run_chat(&transport, &test_config(), "hello", &mut creds, false).await {
        Ok(value) => value,
        Err(err) => panic!("{err}"),
    };
    assert_eq!(out, "done");
    let calls = match transport.calls.lock() {
        Ok(value) => value,
        Err(_) => panic!("lock poisoned"),
    };
    assert_eq!(
        calls.as_slice(),
        ["chat:http://127.0.0.1:3010:hello:none:terminal:auth"]
    );
    assert_eq!(creds.conversation_id.as_deref(), Some("generated"));
}
