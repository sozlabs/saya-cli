use super::{ChatRequest, ChatResult, SayaTransport};
use crate::contracts::{
    Attachment, ConversationCreateRequest, ConversationCreateResponse, MessageContent,
    MessageRequest, MessageResponse,
};
use crate::debug::debug_log;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use std::time::Duration;

pub struct HttpTransport;

fn map_http_error(status: reqwest::StatusCode, body: String, op: &str) -> String {
    if status.as_u16() == 401 {
        return format!("{op} failed (401): unauthorized, check access token");
    }
    if status.as_u16() == 403 {
        return format!("{op} failed (403): forbidden");
    }
    if status.as_u16() == 404 {
        return format!("{op} failed (404): resource not found");
    }
    if status.as_u16() == 429 {
        return format!("{op} failed (429): server busy, retry later");
    }
    if status.is_server_error() {
        return format!("{op} failed ({}): upstream server error", status);
    }
    format!("{op} failed ({status}): {body}")
}

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

    fn chat(&self, request: &ChatRequest) -> Result<ChatResult, String> {
        let token = match request.auth_token.as_deref() {
            Some(value) if !value.trim().is_empty() => value,
            _ => return Err("missing access token: set credentials token before chat".to_string()),
        };
        let client = Client::builder()
            .timeout(request.timeout)
            .build()
            .map_err(|e| format!("failed to build http client: {e}"))?;
        let auth_value = format!("Bearer {token}");
        debug_log(
            request.debug,
            &format!(
                "chat dispatch base_url={} timeout_ms={} authorization={} message_len={} conversation_id={}",
                request.base_url,
                request.timeout.as_millis(),
                if request.auth_token.is_some() {
                    "Bearer token-present"
                } else {
                    "none"
                },
                request.message.len(),
                request.conversation_id.as_deref().map_or("new", |value| value)
            ),
        );
        let root = request.base_url.trim_end_matches('/');
        let active_conversation = match request.conversation_id.as_deref() {
            Some(value) => value.to_string(),
            None => {
                let create_url = format!("{root}/v1/conversations");
                let create_request = ConversationCreateRequest {
                    session_id: request.context.session_id.clone(),
                    tenant_id: request.context.tenant_id.clone(),
                    actor_id: request.context.actor_id.clone(),
                    channel_id: request.context.channel_id.clone(),
                };
                let response = client
                    .post(&create_url)
                    .header(CONTENT_TYPE, "application/json")
                    .header(AUTHORIZATION, &auth_value)
                    .json(&create_request)
                    .send()
                    .map_err(|e| format!("create conversation request failed: {e}"))?;
                let status = response.status();
                if !status.is_success() {
                    let body = response.text().map_err(|e| {
                        format!("failed to read create conversation error body: {e}")
                    })?;
                    return Err(map_http_error(status, body, "create conversation"));
                }
                let payload = response
                    .json::<ConversationCreateResponse>()
                    .map_err(|e| format!("failed to parse create conversation response: {e}"))?;
                payload.conversation_id
            }
        };
        let message_url = format!("{root}/v1/conversations/{active_conversation}/messages");
        let message_request = MessageRequest {
            role: "user".to_string(),
            content: MessageContent {
                r#type: "text".to_string(),
                text: request.message.clone(),
            },
            attachments: Vec::<Attachment>::new(),
            context: request.context.clone(),
        };
        let response = client
            .post(&message_url)
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, &auth_value)
            .json(&message_request)
            .send()
            .map_err(|e| format!("send message request failed: {e}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .map_err(|e| format!("failed to read send message error body: {e}"))?;
            return Err(map_http_error(status, body, "send message"));
        }
        let payload = response
            .json::<MessageResponse>()
            .map_err(|e| format!("failed to parse message response: {e}"))?;
        Ok(ChatResult {
            conversation_id: active_conversation,
            text: payload.content.text,
        })
    }
}
