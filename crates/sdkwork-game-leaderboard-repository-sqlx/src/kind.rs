use async_trait::async_trait;
use sdkwork_game_leaderboard_service::{
    LeaderboardConfigItem, LeaderboardConfigPage, LeaderboardConfigQuery,
    LeaderboardEntriesRebuildCommand, LeaderboardEntry, LeaderboardEntryUpdateCommand,
    LeaderboardPage, LeaderboardQuery, LeaderboardRepository, LeaderboardResult,
};

#[cfg(any(test, feature = "test-support"))]
use crate::memory::InMemoryLeaderboardRepository;
use crate::sqlx::SqlxLeaderboardRepository;

pub enum LeaderboardRepositoryKind {
    #[cfg(any(test, feature = "test-support"))]
    Memory(InMemoryLeaderboardRepository),
    Sqlx(Box<SqlxLeaderboardRepository>),
}

#[async_trait]
impl LeaderboardRepository for LeaderboardRepositoryKind {
    async fn list_configs(
        &self,
        tenant_id: &str,
        query: &LeaderboardConfigQuery,
    ) -> LeaderboardResult<LeaderboardConfigPage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_configs(tenant_id, query).await,
            Self::Sqlx(repo) => repo.list_configs(tenant_id, query).await,
        }
    }

    async fn get_config(
        &self,
        tenant_id: &str,
        leaderboard_id: &str,
    ) -> LeaderboardResult<LeaderboardConfigItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.get_config(tenant_id, leaderboard_id).await,
            Self::Sqlx(repo) => repo.get_config(tenant_id, leaderboard_id).await,
        }
    }

    async fn list_rankings(
        &self,
        tenant_id: &str,
        query: &LeaderboardQuery,
    ) -> LeaderboardResult<LeaderboardPage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_rankings(tenant_id, query).await,
            Self::Sqlx(repo) => repo.list_rankings(tenant_id, query).await,
        }
    }

    async fn get_user_ranking(
        &self,
        tenant_id: &str,
        user_id: &str,
        game_id: Option<&str>,
    ) -> LeaderboardResult<LeaderboardEntry> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.get_user_ranking(tenant_id, user_id, game_id).await,
            Self::Sqlx(repo) => repo.get_user_ranking(tenant_id, user_id, game_id).await,
        }
    }

    async fn upsert_entry(
        &self,
        tenant_id: &str,
        command: &LeaderboardEntryUpdateCommand,
    ) -> LeaderboardResult<LeaderboardEntry> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.upsert_entry(tenant_id, command).await,
            Self::Sqlx(repo) => repo.upsert_entry(tenant_id, command).await,
        }
    }

    async fn rebuild_entries(
        &self,
        tenant_id: &str,
        command: &LeaderboardEntriesRebuildCommand,
    ) -> LeaderboardResult<LeaderboardPage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.rebuild_entries(tenant_id, command).await,
            Self::Sqlx(repo) => repo.rebuild_entries(tenant_id, command).await,
        }
    }
}
