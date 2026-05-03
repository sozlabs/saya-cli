use std::time::Duration;

use crate::config::OutputFormat;
use crate::contracts::ConversationContext;
use async_trait::async_trait;

pub mod http;

#[derive(Clone, Debug)]
pub struct ChatResult {
    pub conversation_id: String,
    pub text: String,
    /// Recoverable issue: e.g. stream ended without `done`, retries exhausted, or user interrupt.
    pub warning: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ChatRequest {
    pub base_url: String,
    pub message: String,
    pub conversation_id: Option<String>,
    pub context: ConversationContext,
    pub timeout: Duration,
    pub auth_token: Option<String>,
    pub debug: bool,
    pub output_format: OutputFormat,
    pub stream_max_retries: u32,
    pub stream_retry_initial_ms: u64,
    pub stream_retry_max_ms: u64,
    /// `Some(true)` only after explicit opt-in (flag/env) or interactive confirm; otherwise omit/`None` for server default deny.
    pub allow_restricted_tools: Option<bool>,
}

#[async_trait]
pub trait SayaTransport: Send + Sync {
    async fn health(
        &self,
        base_url: &str,
        timeout: Duration,
        debug: bool,
    ) -> Result<String, String>;

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResult, String>;
}
