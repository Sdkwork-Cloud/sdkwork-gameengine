use async_trait::async_trait;
use sdkwork_game_session_service::{
    CreateGameSessionCommand, GameSessionItem, GameSessionParticipantItem, GameSessionRepository,
    GameSessionResult, GameSessionResultItem, StartGameSessionCommand, SubmitSessionResultCommand,
};

#[cfg(any(test, feature = "test-support"))]
use crate::memory::InMemoryGameSessionRepository;
use crate::sqlx::SqlxGameSessionRepository;

pub enum GameSessionRepositoryKind {
    #[cfg(any(test, feature = "test-support"))]
    Memory(InMemoryGameSessionRepository),
    Sqlx(Box<SqlxGameSessionRepository>),
}

#[async_trait]
impl GameSessionRepository for GameSessionRepositoryKind {
    async fn create_session(
        &self,
        tenant_id: &str,
        command: &CreateGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.create_session(tenant_id, command).await,
            Self::Sqlx(repo) => repo.create_session(tenant_id, command).await,
        }
    }

    async fn get_session(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<GameSessionItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.get_session(tenant_id, session_id).await,
            Self::Sqlx(repo) => repo.get_session(tenant_id, session_id).await,
        }
    }

    async fn list_participants(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<Vec<GameSessionParticipantItem>> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_participants(tenant_id, session_id).await,
            Self::Sqlx(repo) => repo.list_participants(tenant_id, session_id).await,
        }
    }

    async fn start_session(
        &self,
        tenant_id: &str,
        command: &StartGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.start_session(tenant_id, command).await,
            Self::Sqlx(repo) => repo.start_session(tenant_id, command).await,
        }
    }

    async fn submit_result(
        &self,
        tenant_id: &str,
        command: &SubmitSessionResultCommand,
    ) -> GameSessionResult<GameSessionResultItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.submit_result(tenant_id, command).await,
            Self::Sqlx(repo) => repo.submit_result(tenant_id, command).await,
        }
    }
}
