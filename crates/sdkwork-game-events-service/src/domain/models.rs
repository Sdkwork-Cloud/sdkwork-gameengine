use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameEngineEventItem {
    pub id: String,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
    pub event_payload: Value,
    pub status: String,
    pub trace_id: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_retry_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecordItem {
    pub id: String,
    pub actor_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    pub before_snapshot: Value,
    pub after_snapshot: Value,
    pub trace_id: String,
    pub created_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameEngineEventPage {
    pub items: Vec<GameEngineEventItem>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecordPage {
    pub items: Vec<AuditRecordItem>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppendGameEngineEventCommand {
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
    pub event_payload: Value,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkGameEngineEventPublishedCommand {
    pub event_id: String,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkGameEngineEventFailedCommand {
    pub event_id: String,
    pub next_retry_at: Option<String>,
    pub dead_letter: bool,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingGameEngineEventQuery {
    pub due_at: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl PendingGameEngineEventQuery {
    pub fn limit(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> u32 {
        self.page.unwrap_or(1).saturating_sub(1) * self.limit()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppendAuditRecordCommand {
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub reason_code: Option<String>,
    pub before_snapshot: Value,
    pub after_snapshot: Value,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AuditRecordQuery {
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub actor_type: Option<String>,
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl AuditRecordQuery {
    pub fn limit(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> u32 {
        self.page.unwrap_or(1).saturating_sub(1) * self.limit()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEventError {
    code: String,
    message: String,
}

impl GameEventError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid".into(),
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: "not_found".into(),
            message: message.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            code: "conflict".into(),
            message: message.into(),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

pub type GameEventResult<T> = Result<T, GameEventError>;
