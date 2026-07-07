use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MatchTicketItem {
    pub id: String,
    pub ticket_code: String,
    pub game_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruleset_id: Option<String>,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<String>,
    pub status: String,
    pub priority: i32,
    pub match_attributes: Value,
    pub idempotency_key: String,
    pub queued_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchTicketPage {
    pub items: Vec<MatchTicketItem>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchTicketQuery {
    pub game_id: Option<String>,
    pub mode_id: Option<String>,
    pub status: Option<String>,
    pub user_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl MatchTicketQuery {
    pub fn limit(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchmakingQueueQuery {
    pub game_id: String,
    pub mode_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl MatchmakingQueueQuery {
    pub fn limit(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateMatchTicketCommand {
    pub game_id: String,
    pub mode_id: Option<String>,
    pub ruleset_id: Option<String>,
    pub user_id: String,
    pub party_id: Option<String>,
    pub priority: i32,
    pub match_attributes: Value,
    pub idempotency_key: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CancelMatchTicketCommand {
    pub ticket_id: String,
    pub user_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameMatchmakingError {
    code: String,
    message: String,
}

impl GameMatchmakingError {
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

pub type GameMatchmakingResult<T> = Result<T, GameMatchmakingError>;
