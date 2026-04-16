use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationContext {
    pub session_id: String,
    pub tenant_id: String,
    pub actor_id: String,
    pub channel_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationCreateRequest {
    pub session_id: String,
    pub tenant_id: String,
    pub actor_id: String,
    pub channel_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConversationCreateResponse {
    pub conversation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageContent {
    pub r#type: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageRequest {
    pub role: String,
    pub content: MessageContent,
    pub attachments: Vec<Attachment>,
    pub context: ConversationContext,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Attachment {
    pub asset_id: String,
    pub data_base64: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageResponse {
    pub content: MessageContent,
}
