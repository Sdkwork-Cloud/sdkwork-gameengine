use async_trait::async_trait;
use sdkwork_game_events_service::{
    AppendAuditRecordCommand, AppendGameEngineEventCommand, AuditRecordItem, AuditRecordPage,
    AuditRecordQuery, GameEngineEventItem, GameEngineEventPage, GameEventResult,
    GameEventsRepository, MarkGameEngineEventFailedCommand, MarkGameEngineEventPublishedCommand,
    PendingGameEngineEventQuery,
};

#[cfg(any(test, feature = "test-support"))]
use crate::memory::InMemoryGameEventsRepository;
use crate::sqlx::SqlxGameEventsRepository;

pub enum GameEventsRepositoryKind {
    #[cfg(any(test, feature = "test-support"))]
    Memory(InMemoryGameEventsRepository),
    Sqlx(Box<SqlxGameEventsRepository>),
}

#[async_trait]
impl GameEventsRepository for GameEventsRepositoryKind {
    async fn append_event(
        &self,
        tenant_id: &str,
        command: &AppendGameEngineEventCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.append_event(tenant_id, command).await,
            Self::Sqlx(repo) => repo.append_event(tenant_id, command).await,
        }
    }

    async fn get_event(
        &self,
        tenant_id: &str,
        event_id: &str,
    ) -> GameEventResult<GameEngineEventItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.get_event(tenant_id, event_id).await,
            Self::Sqlx(repo) => repo.get_event(tenant_id, event_id).await,
        }
    }

    async fn list_pending_events(
        &self,
        tenant_id: &str,
        query: &PendingGameEngineEventQuery,
    ) -> GameEventResult<GameEngineEventPage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_pending_events(tenant_id, query).await,
            Self::Sqlx(repo) => repo.list_pending_events(tenant_id, query).await,
        }
    }

    async fn mark_event_published(
        &self,
        tenant_id: &str,
        command: &MarkGameEngineEventPublishedCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.mark_event_published(tenant_id, command).await,
            Self::Sqlx(repo) => repo.mark_event_published(tenant_id, command).await,
        }
    }

    async fn mark_event_failed(
        &self,
        tenant_id: &str,
        command: &MarkGameEngineEventFailedCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.mark_event_failed(tenant_id, command).await,
            Self::Sqlx(repo) => repo.mark_event_failed(tenant_id, command).await,
        }
    }

    async fn append_audit_record(
        &self,
        tenant_id: &str,
        command: &AppendAuditRecordCommand,
    ) -> GameEventResult<AuditRecordItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.append_audit_record(tenant_id, command).await,
            Self::Sqlx(repo) => repo.append_audit_record(tenant_id, command).await,
        }
    }

    async fn search_audit_records(
        &self,
        tenant_id: &str,
        query: &AuditRecordQuery,
    ) -> GameEventResult<AuditRecordPage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.search_audit_records(tenant_id, query).await,
            Self::Sqlx(repo) => repo.search_audit_records(tenant_id, query).await,
        }
    }
}
