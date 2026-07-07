use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameRulesetItem {
    pub id: String,
    pub game_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    pub ruleset_code: String,
    pub version_no: i32,
    pub status: String,
    pub config_schema: Value,
    pub config_values: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateGameRulesetCommand {
    pub game_id: String,
    pub mode_id: Option<String>,
    pub ruleset_code: String,
    pub version_no: i32,
    pub status: String,
    pub config_schema: Value,
    pub config_values: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRulesetError {
    code: String,
    message: String,
}

impl GameRulesetError {
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

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

pub type GameRulesetResult<T> = Result<T, GameRulesetError>;
