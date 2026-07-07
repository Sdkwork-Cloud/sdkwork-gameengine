use async_trait::async_trait;
use sdkwork_game_rules_service::{
    CreateGameRulesetCommand, GameRulesetItem, GameRulesetRepository, GameRulesetResult,
};

#[cfg(any(test, feature = "test-support"))]
use crate::memory::InMemoryGameRulesetRepository;
use crate::sqlx::SqlxGameRulesetRepository;

pub enum GameRulesetRepositoryKind {
    #[cfg(any(test, feature = "test-support"))]
    Memory(InMemoryGameRulesetRepository),
    Sqlx(Box<SqlxGameRulesetRepository>),
}

#[async_trait]
impl GameRulesetRepository for GameRulesetRepositoryKind {
    async fn get_active_ruleset(
        &self,
        tenant_id: &str,
        game_id: &str,
        mode_id: Option<&str>,
    ) -> GameRulesetResult<GameRulesetItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.get_active_ruleset(tenant_id, game_id, mode_id).await,
            Self::Sqlx(repo) => repo.get_active_ruleset(tenant_id, game_id, mode_id).await,
        }
    }

    async fn create_ruleset(
        &self,
        tenant_id: &str,
        command: &CreateGameRulesetCommand,
    ) -> GameRulesetResult<GameRulesetItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.create_ruleset(tenant_id, command).await,
            Self::Sqlx(repo) => repo.create_ruleset(tenant_id, command).await,
        }
    }
}
