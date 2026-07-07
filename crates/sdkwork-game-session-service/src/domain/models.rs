use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameSessionItem {
    pub id: String,
    pub session_code: String,
    pub game_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruleset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_result_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    pub result_version: i32,
    pub metadata: Value,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameSessionParticipantItem {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_no: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name_snapshot: Option<String>,
    pub status: String,
    pub score_delta: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank_no: Option<i32>,
    pub result_payload: Value,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameSessionResultItem {
    pub id: String,
    pub session_id: String,
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub signature_status: String,
    pub validation_status: String,
    pub result_payload: Value,
    pub received_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateGameSessionParticipant {
    pub user_id: String,
    pub team_no: Option<i32>,
    pub display_name_snapshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateGameSessionCommand {
    pub session_code: String,
    pub game_id: String,
    pub mode_id: Option<String>,
    pub ruleset_id: Option<String>,
    pub room_id: Option<String>,
    pub match_result_id: Option<String>,
    pub server_id: Option<String>,
    pub created_by: Option<String>,
    pub metadata: Value,
    pub participants: Vec<CreateGameSessionParticipant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartGameSessionCommand {
    pub session_id: String,
    pub server_id: Option<String>,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubmitSessionResultCommand {
    pub session_id: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub result_payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSessionError {
    code: String,
    message: String,
}

impl GameSessionError {
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

pub type GameSessionResult<T> = Result<T, GameSessionError>;
