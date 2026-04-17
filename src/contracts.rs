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
    /// When `Some(true)`, soz-saya may execute allowlisted **restricted** tools; omit or `false` denies by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_restricted_tools: Option<bool>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_request_omits_allow_restricted_when_none() {
        let m = MessageRequest {
            role: "user".to_string(),
            content: MessageContent {
                r#type: "text".to_string(),
                text: "hi".to_string(),
            },
            attachments: vec![],
            context: ConversationContext {
                session_id: "s".to_string(),
                tenant_id: "t".to_string(),
                actor_id: "a".to_string(),
                channel_id: "terminal".to_string(),
            },
            allow_restricted_tools: None,
        };
        let v = serde_json::to_value(&m).expect("serialize");
        assert!(v.get("allow_restricted_tools").is_none());
    }

    #[test]
    fn message_request_serializes_allow_restricted_true() {
        let m = MessageRequest {
            role: "user".to_string(),
            content: MessageContent {
                r#type: "text".to_string(),
                text: "hi".to_string(),
            },
            attachments: vec![],
            context: ConversationContext {
                session_id: "s".to_string(),
                tenant_id: "t".to_string(),
                actor_id: "a".to_string(),
                channel_id: "terminal".to_string(),
            },
            allow_restricted_tools: Some(true),
        };
        let v = serde_json::to_value(&m).expect("serialize");
        assert_eq!(
            v.get("allow_restricted_tools").and_then(|x| x.as_bool()),
            Some(true)
        );
    }
}
