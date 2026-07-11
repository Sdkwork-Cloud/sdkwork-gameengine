use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameModeItem {
    pub id: String,
    pub game_id: String,
    pub mode_code: String,
    pub title: String,
    pub status: String,
    pub min_players: i32,
    pub max_players: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruleset_id: Option<String>,
    pub matchmaking_enabled: bool,
    pub room_enabled: bool,
    pub leaderboard_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameModePage {
    pub items: Vec<GameModeItem>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameModeQuery {
    pub game_id: Option<String>,
    pub status: Option<String>,
    pub q: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl GameModeQuery {
    pub fn limit(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateGameModeCommand {
    pub game_id: String,
    pub mode_code: String,
    pub title: String,
    pub status: String,
    pub min_players: i32,
    pub max_players: i32,
    pub team_size: Option<i32>,
    pub ruleset_id: Option<String>,
    pub matchmaking_enabled: bool,
    pub room_enabled: bool,
    pub leaderboard_enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateGameModeCommand {
    pub title: Option<String>,
    pub status: Option<String>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub team_size: Option<Option<i32>>,
    pub ruleset_id: Option<Option<String>>,
    pub matchmaking_enabled: Option<bool>,
    pub room_enabled: Option<bool>,
    pub leaderboard_enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameModeError {
    code: String,
    message: String,
}

impl GameModeError {
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

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

pub type GameModeResult<T> = Result<T, GameModeError>;
