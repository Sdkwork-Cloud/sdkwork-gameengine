use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GamePointLedgerEntry {
    pub id: String,
    pub ledger_account_id: String,
    pub game_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season_id: Option<String>,
    pub user_id: String,
    pub direction: String,
    pub points_delta: i64,
    pub points_after: i64,
    pub source_event_id: String,
    pub reason_code: String,
    pub idempotency_key: String,
    pub created_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GamePointBalance {
    pub id: String,
    pub ledger_account_id: String,
    pub game_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season_id: Option<String>,
    pub user_id: String,
    pub points: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ledger_id: Option<String>,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppendPointLedgerCommand {
    pub ledger_account_id: String,
    pub game_id: String,
    pub mode_id: Option<String>,
    pub season_id: Option<String>,
    pub user_id: String,
    pub direction: String,
    pub points_delta: i64,
    pub source_event_id: String,
    pub reason_code: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamePointError {
    code: String,
    message: String,
}

impl GamePointError {
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

pub type GamePointResult<T> = Result<T, GamePointError>;
