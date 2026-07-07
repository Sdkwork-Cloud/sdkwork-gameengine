use async_trait::async_trait;

use crate::domain::models::{
    LeaderboardConfigItem, LeaderboardConfigPage, LeaderboardConfigQuery,
    LeaderboardEntriesRebuildCommand, LeaderboardEntry, LeaderboardEntryUpdateCommand,
    LeaderboardPage, LeaderboardQuery, LeaderboardResult,
};

#[async_trait]
pub trait LeaderboardRepository: Send + Sync {
    async fn list_configs(
        &self,
        tenant_id: &str,
        query: &LeaderboardConfigQuery,
    ) -> LeaderboardResult<LeaderboardConfigPage>;

    async fn get_config(
        &self,
        tenant_id: &str,
        leaderboard_id: &str,
    ) -> LeaderboardResult<LeaderboardConfigItem>;

    async fn list_rankings(
        &self,
        tenant_id: &str,
        query: &LeaderboardQuery,
    ) -> LeaderboardResult<LeaderboardPage>;

    async fn get_user_ranking(
        &self,
        tenant_id: &str,
        user_id: &str,
        game_id: Option<&str>,
    ) -> LeaderboardResult<LeaderboardEntry>;

    async fn upsert_entry(
        &self,
        tenant_id: &str,
        command: &LeaderboardEntryUpdateCommand,
    ) -> LeaderboardResult<LeaderboardEntry>;

    async fn rebuild_entries(
        &self,
        tenant_id: &str,
        command: &LeaderboardEntriesRebuildCommand,
    ) -> LeaderboardResult<LeaderboardPage>;
}
