use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameRoomItem {
    pub id: String,
    pub game_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruleset_id: Option<String>,
    pub room_code: String,
    pub host_user_id: String,
    pub visibility: String,
    pub join_policy: String,
    pub max_players: i32,
    pub current_players: i32,
    pub status: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameRoomSeatItem {
    pub id: String,
    pub room_id: String,
    pub seat_no: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_no: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name_snapshot: Option<String>,
    pub status: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameRoomPage {
    pub items: Vec<GameRoomItem>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameRoomQuery {
    pub game_id: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl GameRoomQuery {
    pub fn limit(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateGameRoomCommand {
    pub game_id: String,
    pub mode_id: Option<String>,
    pub ruleset_id: Option<String>,
    pub room_code: String,
    pub host_user_id: String,
    pub visibility: String,
    pub join_policy: String,
    pub max_players: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JoinGameRoomCommand {
    pub room_id: String,
    pub user_id: String,
    pub display_name_snapshot: Option<String>,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaveGameRoomCommand {
    pub room_id: String,
    pub user_id: String,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadyGameRoomCommand {
    pub room_id: String,
    pub user_id: String,
    pub ready: bool,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartGameRoomCommand {
    pub room_id: String,
    pub host_user_id: String,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloseGameRoomCommand {
    pub room_id: String,
    pub operator_user_id: String,
    pub reason: Option<String>,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRoomError {
    code: String,
    message: String,
}

impl GameRoomError {
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

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            code: "forbidden".into(),
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

pub type GameRoomResult<T> = Result<T, GameRoomError>;
