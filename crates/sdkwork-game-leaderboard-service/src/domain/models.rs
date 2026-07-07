use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub id: String,
    pub game_id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub score: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank_no: Option<i32>,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardConfigItem {
    pub id: String,
    pub game_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season_id: Option<String>,
    pub leaderboard_code: String,
    pub title: String,
    pub status: String,
    pub ranking_metric: String,
    pub ranking_order: String,
    pub tie_breaker: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderboardPage {
    pub items: Vec<LeaderboardEntry>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderboardConfigPage {
    pub items: Vec<LeaderboardConfigItem>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderboardQuery {
    pub leaderboard_id: Option<String>,
    pub game_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl LeaderboardQuery {
    pub fn limit(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderboardConfigQuery {
    pub game_id: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl LeaderboardConfigQuery {
    pub fn limit(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderboardEntryUpdateCommand {
    pub leaderboard_id: String,
    pub game_id: String,
    pub mode_id: Option<String>,
    pub season_id: Option<String>,
    pub user_id: String,
    pub display_name_snapshot: Option<String>,
    pub score_value: i64,
    pub tie_breaker_value: Option<String>,
    pub last_ledger_id: Option<String>,
    pub recorded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderboardEntriesRebuildCommand {
    pub leaderboard_id: String,
    pub entries: Vec<LeaderboardEntryUpdateCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderboardError {
    code: String,
    message: String,
}

impl LeaderboardError {
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

pub type LeaderboardResult<T> = Result<T, LeaderboardError>;
