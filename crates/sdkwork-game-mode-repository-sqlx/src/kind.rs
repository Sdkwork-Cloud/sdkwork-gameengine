use async_trait::async_trait;
use sdkwork_game_mode_service::{
    CreateGameModeCommand, GameModeItem, GameModePage, GameModeQuery, GameModeRepository,
    GameModeResult, UpdateGameModeCommand,
};

#[cfg(any(test, feature = "test-support"))]
use crate::memory::InMemoryGameModeRepository;
use crate::sqlx::SqlxGameModeRepository;

pub enum GameModeRepositoryKind {
    #[cfg(any(test, feature = "test-support"))]
    Memory(InMemoryGameModeRepository),
    Sqlx(Box<SqlxGameModeRepository>),
}

#[async_trait]
impl GameModeRepository for GameModeRepositoryKind {
    async fn list_modes(
        &self,
        tenant_id: &str,
        query: &GameModeQuery,
    ) -> GameModeResult<GameModePage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_modes(tenant_id, query).await,
            Self::Sqlx(repo) => repo.list_modes(tenant_id, query).await,
        }
    }

    async fn get_mode(&self, tenant_id: &str, mode_id: &str) -> GameModeResult<GameModeItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.get_mode(tenant_id, mode_id).await,
            Self::Sqlx(repo) => repo.get_mode(tenant_id, mode_id).await,
        }
    }

    async fn create_mode(
        &self,
        tenant_id: &str,
        command: &CreateGameModeCommand,
    ) -> GameModeResult<GameModeItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.create_mode(tenant_id, command).await,
            Self::Sqlx(repo) => repo.create_mode(tenant_id, command).await,
        }
    }

    async fn update_mode(
        &self,
        tenant_id: &str,
        mode_id: &str,
        command: &UpdateGameModeCommand,
    ) -> GameModeResult<GameModeItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.update_mode(tenant_id, mode_id, command).await,
            Self::Sqlx(repo) => repo.update_mode(tenant_id, mode_id, command).await,
        }
    }
}
