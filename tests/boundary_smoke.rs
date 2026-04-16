use saya_cli::commands::{run_chat, run_health};
use saya_cli::config::{CliConfig, OutputFormat};
use saya_cli::credentials::Credentials;
use saya_cli::transport::SayaTransport;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Default)]
struct MockTransport {
    calls: Mutex<Vec<String>>,
}

impl SayaTransport for MockTransport {
    fn health(&self, base_url: &str, _timeout: Duration, _debug: bool) -> Result<String, String> {
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
        timeout: Duration::from_secs(2),
        output_format: OutputFormat::Text,
        non_interactive: true,
        debug: false,
        config_dir: PathBuf::from(".saya"),
    }
}

#[test]
fn health_uses_saya_transport_shape() {
    let transport = MockTransport::default();
    let out = run_health(&transport, &test_config()).expect("health should succeed");
    assert_eq!(out, "ok");
    let calls = transport.calls.lock().expect("lock poisoned");
    assert_eq!(calls.as_slice(), ["health:http://127.0.0.1:3010"]);
}

#[test]
fn chat_uses_transport_instead_of_agent_runtime() {
    let transport = MockTransport::default();
    let creds = Credentials {
        access_token: Some("tok".to_string()),
    };
    let out = run_chat(&transport, &test_config(), "hello", &creds).expect("chat should succeed");
    assert_eq!(out, "done");
    let calls = transport.calls.lock().expect("lock poisoned");
    assert_eq!(calls.as_slice(), ["chat:http://127.0.0.1:3010:hello"]);
}
