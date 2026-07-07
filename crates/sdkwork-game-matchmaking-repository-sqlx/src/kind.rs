use async_trait::async_trait;
use sdkwork_game_matchmaking_service::{
    CancelMatchTicketCommand, CreateMatchTicketCommand, GameMatchmakingRepository,
    GameMatchmakingResult, MatchTicketItem, MatchTicketPage, MatchTicketQuery,
    MatchmakingQueueQuery,
};

#[cfg(any(test, feature = "test-support"))]
use crate::memory::InMemoryGameMatchmakingRepository;
use crate::sqlx::SqlxGameMatchmakingRepository;

pub enum GameMatchmakingRepositoryKind {
    #[cfg(any(test, feature = "test-support"))]
    Memory(InMemoryGameMatchmakingRepository),
    Sqlx(Box<SqlxGameMatchmakingRepository>),
}

#[async_trait]
impl GameMatchmakingRepository for GameMatchmakingRepositoryKind {
    async fn create_ticket(
        &self,
        tenant_id: &str,
        command: &CreateMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.create_ticket(tenant_id, command).await,
            Self::Sqlx(repo) => repo.create_ticket(tenant_id, command).await,
        }
    }

    async fn get_ticket(
        &self,
        tenant_id: &str,
        ticket_id: &str,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.get_ticket(tenant_id, ticket_id).await,
            Self::Sqlx(repo) => repo.get_ticket(tenant_id, ticket_id).await,
        }
    }

    async fn cancel_ticket(
        &self,
        tenant_id: &str,
        command: &CancelMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.cancel_ticket(tenant_id, command).await,
            Self::Sqlx(repo) => repo.cancel_ticket(tenant_id, command).await,
        }
    }

    async fn list_tickets(
        &self,
        tenant_id: &str,
        query: &MatchTicketQuery,
    ) -> GameMatchmakingResult<MatchTicketPage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_tickets(tenant_id, query).await,
            Self::Sqlx(repo) => repo.list_tickets(tenant_id, query).await,
        }
    }

    async fn list_queue(
        &self,
        tenant_id: &str,
        query: &MatchmakingQueueQuery,
    ) -> GameMatchmakingResult<MatchTicketPage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_queue(tenant_id, query).await,
            Self::Sqlx(repo) => repo.list_queue(tenant_id, query).await,
        }
    }
}
