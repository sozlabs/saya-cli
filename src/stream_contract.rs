//! SSE stream payloads: mirror `soz-saya` `StreamEventSchema` (schemas.ts) and
//! `buildStreamJsonSchema` (contracts.ts). Update this module when those change.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmotionState {
    Focus,
    Confused,
    Happy,
    Dormant,
    Supportive,
    WaitingFocus,
    DegradedConfused,
    OfflineSafe,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusState {
    AnalysisStarted,
    StreamStalled,
    StreamCancelled,
    CoreSoftTimeout,
    CoreHardTimeout,
    CoreUnavailable,
    DraftInsightReady,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolTier {
    Safe,
    Restricted,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolOutcome {
    Ok,
    Error,
    Denied,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StreamTokenData {
    pub text: String,
    pub seq: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StreamEmotionData {
    pub state: EmotionState,
    pub source: EmotionSource,
    pub priority: i64,
    pub ttl_ms: u64,
    pub event_id: String,
    pub conversation_id: String,
    pub ts: String,
    pub seq: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum EmotionSource {
    #[serde(rename = "semantic")]
    Semantic,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StreamStatusData {
    pub status: StatusState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_id: Option<String>,
    pub event_id: String,
    pub conversation_id: String,
    pub ts: String,
    pub seq: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StreamToolData {
    pub tool_name: String,
    pub tier: ToolTier,
    pub outcome: ToolOutcome,
    pub event_id: String,
    pub conversation_id: String,
    pub seq: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StreamErrorData {
    pub code: String,
    pub message: String,
    pub seq: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StreamDoneData {
    pub seq: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamEvent {
    Token(StreamTokenData),
    Emotion(StreamEmotionData),
    Status(StreamStatusData),
    Tool(StreamToolData),
    Error(StreamErrorData),
    Done(StreamDoneData),
}

impl StreamEvent {
    /// Deserialize `data` JSON for a given SSE `event` name (wire format uses split event/data).
    pub fn from_wire(event: &str, data_json: &str) -> Result<Self, String> {
        match event.trim() {
            "token" => serde_json::from_str::<StreamTokenData>(data_json)
                .map(Self::Token)
                .map_err(|e| format!("token data: {e}")),
            "emotion" => serde_json::from_str::<StreamEmotionData>(data_json)
                .map(Self::Emotion)
                .map_err(|e| format!("emotion data: {e}")),
            "status" => serde_json::from_str::<StreamStatusData>(data_json)
                .map(Self::Status)
                .map_err(|e| format!("status data: {e}")),
            "tool" => serde_json::from_str::<StreamToolData>(data_json)
                .map(Self::Tool)
                .map_err(|e| format!("tool data: {e}")),
            "error" => serde_json::from_str::<StreamErrorData>(data_json)
                .map(Self::Error)
                .map_err(|e| format!("error data: {e}")),
            "done" => serde_json::from_str::<StreamDoneData>(data_json)
                .map(Self::Done)
                .map_err(|e| format!("done data: {e}")),
            other => Err(format!("unknown SSE event type: {other}")),
        }
    }
}
