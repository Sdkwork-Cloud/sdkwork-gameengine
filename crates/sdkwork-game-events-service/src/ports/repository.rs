use async_trait::async_trait;

use crate::domain::models::{
    AppendAuditRecordCommand, AppendGameEngineEventCommand, AuditRecordItem, AuditRecordPage,
    AuditRecordQuery, GameEngineEventItem, GameEngineEventPage, GameEventResult,
    MarkGameEngineEventFailedCommand, MarkGameEngineEventPublishedCommand,
    PendingGameEngineEventQuery,
};

#[async_trait]
pub trait GameEventsRepository: Send + Sync {
    async fn append_event(
        &self,
        tenant_id: &str,
        command: &AppendGameEngineEventCommand,
    ) -> GameEventResult<GameEngineEventItem>;

    async fn get_event(
        &self,
        tenant_id: &str,
        event_id: &str,
    ) -> GameEventResult<GameEngineEventItem>;

    async fn list_pending_events(
        &self,
        tenant_id: &str,
        query: &PendingGameEngineEventQuery,
    ) -> GameEventResult<GameEngineEventPage>;

    async fn mark_event_published(
        &self,
        tenant_id: &str,
        command: &MarkGameEngineEventPublishedCommand,
    ) -> GameEventResult<GameEngineEventItem>;

    async fn mark_event_failed(
        &self,
        tenant_id: &str,
        command: &MarkGameEngineEventFailedCommand,
    ) -> GameEventResult<GameEngineEventItem>;

    async fn append_audit_record(
        &self,
        tenant_id: &str,
        command: &AppendAuditRecordCommand,
    ) -> GameEventResult<AuditRecordItem>;

    async fn search_audit_records(
        &self,
        tenant_id: &str,
        query: &AuditRecordQuery,
    ) -> GameEventResult<AuditRecordPage>;
}
