use super::{ChatRequest, ChatResult, SayaTransport};
use crate::config::OutputFormat;
use crate::contracts::{
    Attachment, ConversationCreateRequest, ConversationCreateResponse, MessageContent,
    MessageRequest,
};
use crate::debug::debug_log;
use crate::sse_parse::SseDecoder;
use crate::stream_contract::StreamEvent;
use crate::stream_ux::StreamUx;
use crate::terminal_guard::TerminalGuard;
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::redirect::Policy;
use std::io::{IsTerminal, Write};
use std::time::{Duration, Instant};

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
        let client = reqwest::blocking::Client::builder()
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
        let _guard = TerminalGuard;
        let blocking = reqwest::blocking::Client::builder()
            .timeout(request.timeout)
            .build()
            .map_err(|e| format!("failed to build http client: {e}"))?;
        let auth_value = format!("Bearer {token}");
        debug_log(
            request.debug,
            &format!(
                "chat stream base_url={} connect_timeout_ms={} authorization={} message_len={} conversation_id={}",
                request.base_url,
                request.timeout.as_millis(),
                "Bearer token-present",
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
                let response = blocking
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

        let stream_url = format!("{root}/v1/conversations/{active_conversation}/messages/stream");
        let message_request = MessageRequest {
            role: "user".to_string(),
            content: MessageContent {
                r#type: "text".to_string(),
                text: request.message.clone(),
            },
            attachments: Vec::<Attachment>::new(),
            context: request.context.clone(),
            allow_restricted_tools: request.allow_restricted_tools,
        };

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("failed to start async runtime: {e}"))?;

        let max_attempts = request.stream_max_retries.saturating_add(1);
        let mut accumulated = String::new();
        let mut last_issue: Option<String> = None;

        for attempt in 0..max_attempts {
            if attempt > 0 && !accumulated.is_empty() {
                eprintln!(
                    "[saya] retrying stream ({}/{}): issuing a new POST; output may duplicate if the server repeats the reply",
                    attempt,
                    max_attempts - 1
                );
            }
            let round = rt.block_on(chat_stream_round(
                &stream_url,
                &message_request,
                &auth_value,
                request.timeout,
                request.debug,
                request.output_format,
                &mut accumulated,
            ));

            match round {
                Ok(StreamRound::Completed) => {
                    return Ok(ChatResult {
                        conversation_id: active_conversation,
                        text: accumulated,
                        warning: None,
                    });
                }
                Ok(StreamRound::Interrupted) => {
                    return Ok(ChatResult {
                        conversation_id: active_conversation,
                        text: accumulated,
                        warning: Some("interrupted (Ctrl+C)".to_string()),
                    });
                }
                Ok(StreamRound::Incomplete) => {
                    last_issue = Some("stream closed before done event".to_string());
                }
                Err(e) => {
                    last_issue = Some(e);
                }
            }

            if attempt + 1 < max_attempts {
                let mult = 1u64.checked_shl(attempt.min(31)).unwrap_or(u64::MAX);
                let sleep_ms = request
                    .stream_retry_initial_ms
                    .saturating_mul(mult)
                    .min(request.stream_retry_max_ms);
                std::thread::sleep(Duration::from_millis(sleep_ms));
            }
        }

        Ok(ChatResult {
            conversation_id: active_conversation,
            text: accumulated,
            warning: last_issue,
        })
    }
}

enum StreamRound {
    Completed,
    Interrupted,
    Incomplete,
}

async fn chat_stream_round(
    stream_url: &str,
    message_request: &MessageRequest,
    auth_header: &str,
    connect_timeout: Duration,
    debug: bool,
    output_format: OutputFormat,
    accum: &mut String,
) -> Result<StreamRound, String> {
    let client = reqwest::Client::builder()
        .connect_timeout(connect_timeout)
        .redirect(Policy::limited(10))
        .build()
        .map_err(|e| format!("failed to build async http client: {e}"))?;

    let response = client
        .post(stream_url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, auth_header)
        .json(message_request)
        .send()
        .await
        .map_err(|e| format!("stream request failed: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(map_http_error(status, body, "send message stream"));
    }

    let use_spinner =
        matches!(output_format, OutputFormat::Text) && std::io::stderr().is_terminal();
    let spinner: Option<indicatif::ProgressBar> = if use_spinner {
        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_message("streaming…");
        pb.enable_steady_tick(Duration::from_millis(120));
        Some(pb)
    } else {
        None
    };

    let mut stream = response.bytes_stream();
    let mut decoder = SseDecoder::new();
    let mut saw_done = false;
    let mut spinner = spinner;
    let mut stream_ux = StreamUx::new();

    loop {
        tokio::select! {
            biased;
            ctrl = tokio::signal::ctrl_c() => {
                let _ = ctrl;
                if let Some(pb) = spinner.take() {
                    pb.finish_and_clear();
                }
                if matches!(output_format, OutputFormat::Text) && !accum.is_empty() {
                    let _ = writeln!(std::io::stdout());
                }
                return Ok(StreamRound::Interrupted);
            }
            next = stream.next() => {
                match next {
                    None => break,
                    Some(Err(e)) => {
                        if let Some(pb) = spinner.take() {
                            pb.finish_and_clear();
                        }
                        return Err(format!("stream read failed: {e}"));
                    }
                    Some(Ok(chunk)) => {
                        let events = decoder.push(&chunk).map_err(|e| e.to_string())?;
                        for ev in events {
                            match ev {
                                StreamEvent::Token(t) => {
                                    if let Some(pb) = spinner.take() {
                                        pb.finish_and_clear();
                                    }
                                    accum.push_str(&t.text);
                                    if matches!(output_format, OutputFormat::Text) {
                                        print!("{}", t.text);
                                        let _ = std::io::stdout().flush();
                                    }
                                }
                                StreamEvent::Error(e) => {
                                    if let Some(pb) = spinner.take() {
                                        pb.finish_and_clear();
                                    }
                                    return Err(format!("stream error: {}: {}", e.code, e.message));
                                }
                                StreamEvent::Done(_) => {
                                    saw_done = true;
                                }
                                StreamEvent::Emotion(ref e) => {
                                    stream_ux.on_emotion(e, Instant::now());
                                    debug_log(
                                        debug,
                                        &format!(
                                            "sse emotion state={:?} seq={}",
                                            e.state, e.seq
                                        ),
                                    );
                                }
                                StreamEvent::Status(ref s) => {
                                    stream_ux.on_status(s);
                                    debug_log(
                                        debug,
                                        &format!(
                                            "sse status {:?} seq={}",
                                            s.status, s.seq
                                        ),
                                    );
                                }
                                StreamEvent::Tool(ref t) => {
                                    debug_log(
                                        debug,
                                        &format!(
                                            "sse tool {} outcome={:?} seq={}",
                                            t.tool_name, t.outcome, t.seq
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(pb) = spinner.take() {
        pb.finish_and_clear();
    }

    if saw_done {
        if matches!(output_format, OutputFormat::Text) {
            println!();
        }
        Ok(StreamRound::Completed)
    } else {
        Ok(StreamRound::Incomplete)
    }
}
