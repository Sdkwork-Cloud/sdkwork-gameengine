use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameSettlementJobItem {
    pub id: String,
    pub session_id: String,
    pub session_result_id: String,
    pub status: String,
    pub attempt_count: i32,
    pub idempotency_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<String>,
    pub job_payload: Value,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_retry_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameRewardIntentItem {
    pub id: String,
    pub settlement_job_id: String,
    pub user_id: String,
    pub reward_type: String,
    pub external_owner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_reference_id: Option<String>,
    pub intent_payload: Value,
    pub status: String,
    pub idempotency_key: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameSettlementJobPage {
    pub items: Vec<GameSettlementJobItem>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateSettlementJobCommand {
    pub session_id: String,
    pub session_result_id: String,
    pub idempotency_key: String,
    pub job_payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartSettlementJobCommand {
    pub job_id: String,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordSettlementFailureCommand {
    pub job_id: String,
    pub error_code: String,
    pub error_detail: Option<String>,
    pub next_retry_at: Option<String>,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompleteSettlementJobCommand {
    pub job_id: String,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateRewardIntentCommand {
    pub settlement_job_id: String,
    pub user_id: String,
    pub reward_type: String,
    pub external_owner: String,
    pub intent_payload: Value,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettlementDueJobQuery {
    pub due_at: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl SettlementDueJobQuery {
    pub fn limit(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> u32 {
        self.page.unwrap_or(1).saturating_sub(1) * self.limit()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSettlementError {
    code: String,
    message: String,
}

impl GameSettlementError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid".into(),
            message: message.into(),
        }
    }

    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_parameter".into(),
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

pub type GameSettlementResult<T> = Result<T, GameSettlementError>;
