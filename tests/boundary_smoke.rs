use saya_cli::commands::{run_chat, run_health};
use saya_cli::transport::http::HttpTransport;

#[test]
fn health_uses_saya_transport_shape() {
    let transport = HttpTransport;
    let out = run_health(&transport, "http://127.0.0.1:3010");
    assert!(out.contains("/health"));
    assert!(out.contains("http://127.0.0.1:3010"));
}

#[test]
fn chat_uses_transport_instead_of_agent_runtime() {
    let transport = HttpTransport;
    let out = run_chat(&transport, "http://127.0.0.1:3010", "hello");
    assert!(out.contains("http://127.0.0.1:3010"));
    assert!(!out.to_lowercase().contains("agent"));
    assert!(!out.to_lowercase().contains("rag"));
}
