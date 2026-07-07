use sdkwork_utils_rust::string::is_blank;

use crate::domain::models::{
    AppendAuditRecordCommand, AppendGameEngineEventCommand, AuditRecordItem, AuditRecordPage,
    AuditRecordQuery, GameEngineEventItem, GameEngineEventPage, GameEventError, GameEventResult,
    MarkGameEngineEventFailedCommand, MarkGameEngineEventPublishedCommand,
    PendingGameEngineEventQuery,
};
use crate::ports::repository::GameEventsRepository;

pub struct GameEventsService<R> {
    repository: R,
}

impl<R> GameEventsService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> GameEventsService<R>
where
    R: GameEventsRepository,
{
    pub async fn append_event(
        &self,
        tenant_id: &str,
        command: AppendGameEngineEventCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("event_type", &command.event_type)?;
        validate_event_type(&command.event_type)?;
        validate_required("aggregate_type", &command.aggregate_type)?;
        validate_required("aggregate_id", &command.aggregate_id)?;
        validate_required("idempotency_key", &command.idempotency_key)?;
        validate_required("trace_id", &command.trace_id)?;
        if command.event_payload.is_null() {
            return Err(GameEventError::invalid("event_payload must not be null"));
        }
        self.repository.append_event(tenant_id, &command).await
    }

    pub async fn get_event(
        &self,
        tenant_id: &str,
        event_id: &str,
    ) -> GameEventResult<GameEngineEventItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("event_id", event_id)?;
        self.repository.get_event(tenant_id, event_id).await
    }

    pub async fn list_pending_events(
        &self,
        tenant_id: &str,
        query: PendingGameEngineEventQuery,
    ) -> GameEventResult<GameEngineEventPage> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("due_at", &query.due_at)?;
        self.repository.list_pending_events(tenant_id, &query).await
    }

    pub async fn mark_event_published(
        &self,
        tenant_id: &str,
        command: MarkGameEngineEventPublishedCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("event_id", &command.event_id)?;
        self.repository
            .mark_event_published(tenant_id, &command)
            .await
    }

    pub async fn mark_event_failed(
        &self,
        tenant_id: &str,
        command: MarkGameEngineEventFailedCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("event_id", &command.event_id)?;
        if !command.dead_letter && command.next_retry_at.is_none() {
            return Err(GameEventError::invalid(
                "next_retry_at is required unless event is dead_letter",
            ));
        }
        self.repository.mark_event_failed(tenant_id, &command).await
    }

    pub async fn append_audit_record(
        &self,
        tenant_id: &str,
        command: AppendAuditRecordCommand,
    ) -> GameEventResult<AuditRecordItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("actor_type", &command.actor_type)?;
        validate_actor_type(&command.actor_type)?;
        validate_required("action", &command.action)?;
        validate_required("target_type", &command.target_type)?;
        validate_required("target_id", &command.target_id)?;
        validate_required("trace_id", &command.trace_id)?;
        if command.before_snapshot.is_null() || command.after_snapshot.is_null() {
            return Err(GameEventError::invalid(
                "audit snapshots must be objects, arrays, or empty json",
            ));
        }
        self.repository
            .append_audit_record(tenant_id, &command)
            .await
    }

    pub async fn search_audit_records(
        &self,
        tenant_id: &str,
        query: AuditRecordQuery,
    ) -> GameEventResult<AuditRecordPage> {
        validate_required("tenant_id", tenant_id)?;
        self.repository
            .search_audit_records(tenant_id, &query)
            .await
    }
}

fn validate_required(field: &str, value: &str) -> GameEventResult<()> {
    if is_blank(Some(value)) {
        return Err(GameEventError::invalid(format!("{field} is required")));
    }
    Ok(())
}

fn validate_event_type(event_type: &str) -> GameEventResult<()> {
    if event_type.contains('.') && !event_type.starts_with('.') && !event_type.ends_with('.') {
        return Ok(());
    }
    Err(GameEventError::invalid(
        "event_type must be a stable dotted event type",
    ))
}

fn validate_actor_type(actor_type: &str) -> GameEventResult<()> {
    if matches!(
        actor_type,
        "user" | "operator" | "system" | "server" | "job"
    ) {
        return Ok(());
    }
    Err(GameEventError::invalid("audit actor_type is not supported"))
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    struct EmptyRepo;

    #[async_trait]
    impl GameEventsRepository for EmptyRepo {
        async fn append_event(
            &self,
            _tenant_id: &str,
            command: &AppendGameEngineEventCommand,
        ) -> GameEventResult<GameEngineEventItem> {
            Ok(event_item(command))
        }

        async fn get_event(
            &self,
            _tenant_id: &str,
            _event_id: &str,
        ) -> GameEventResult<GameEngineEventItem> {
            unreachable!("validation must reject before repository access")
        }

        async fn list_pending_events(
            &self,
            _tenant_id: &str,
            _query: &PendingGameEngineEventQuery,
        ) -> GameEventResult<GameEngineEventPage> {
            unreachable!("validation must reject before repository access")
        }

        async fn mark_event_published(
            &self,
            _tenant_id: &str,
            _command: &MarkGameEngineEventPublishedCommand,
        ) -> GameEventResult<GameEngineEventItem> {
            unreachable!("validation must reject before repository access")
        }

        async fn mark_event_failed(
            &self,
            _tenant_id: &str,
            _command: &MarkGameEngineEventFailedCommand,
        ) -> GameEventResult<GameEngineEventItem> {
            unreachable!("validation must reject before repository access")
        }

        async fn append_audit_record(
            &self,
            _tenant_id: &str,
            command: &AppendAuditRecordCommand,
        ) -> GameEventResult<AuditRecordItem> {
            Ok(audit_item(command))
        }

        async fn search_audit_records(
            &self,
            _tenant_id: &str,
            _query: &AuditRecordQuery,
        ) -> GameEventResult<AuditRecordPage> {
            unreachable!("validation must reject before repository access")
        }
    }

    fn event_command(event_type: &str) -> AppendGameEngineEventCommand {
        AppendGameEngineEventCommand {
            event_type: event_type.into(),
            aggregate_type: "session".into(),
            aggregate_id: "session-1".into(),
            idempotency_key: "event-session-1-completed".into(),
            event_payload: json!({"sessionId": "session-1"}),
            trace_id: "trace-1".into(),
        }
    }

    fn audit_command(actor_type: &str) -> AppendAuditRecordCommand {
        AppendAuditRecordCommand {
            actor_type: actor_type.into(),
            actor_id: Some("operator-1".into()),
            action: "session.result.corrected".into(),
            target_type: "session".into(),
            target_id: "session-1".into(),
            reason_code: Some("manual_review".into()),
            before_snapshot: json!({"score": 1}),
            after_snapshot: json!({"score": 2}),
            trace_id: "trace-1".into(),
        }
    }

    fn event_item(command: &AppendGameEngineEventCommand) -> GameEngineEventItem {
        GameEngineEventItem {
            id: "event-1".into(),
            event_type: command.event_type.clone(),
            aggregate_type: command.aggregate_type.clone(),
            aggregate_id: command.aggregate_id.clone(),
            idempotency_key: command.idempotency_key.clone(),
            event_payload: command.event_payload.clone(),
            status: "pending".into(),
            trace_id: command.trace_id.clone(),
            created_at: "2026-07-07T00:00:00Z".into(),
            published_at: None,
            next_retry_at: None,
            version: 0,
        }
    }

    fn audit_item(command: &AppendAuditRecordCommand) -> AuditRecordItem {
        AuditRecordItem {
            id: "audit-1".into(),
            actor_type: command.actor_type.clone(),
            actor_id: command.actor_id.clone(),
            action: command.action.clone(),
            target_type: command.target_type.clone(),
            target_id: command.target_id.clone(),
            reason_code: command.reason_code.clone(),
            before_snapshot: command.before_snapshot.clone(),
            after_snapshot: command.after_snapshot.clone(),
            trace_id: command.trace_id.clone(),
            created_at: "2026-07-07T00:00:00Z".into(),
            version: 0,
        }
    }

    #[tokio::test]
    async fn append_event_rejects_non_dotted_type() {
        let service = GameEventsService::new(EmptyRepo);

        let error = service
            .append_event("100001", event_command("session_completed"))
            .await
            .expect_err("event type must be dotted");

        assert_eq!("invalid", error.code());
    }

    #[tokio::test]
    async fn append_event_accepts_cloud_events_style_type() {
        let service = GameEventsService::new(EmptyRepo);

        let item = service
            .append_event("100001", event_command("game.session.completed"))
            .await
            .expect("event");

        assert_eq!("game.session.completed", item.event_type);
        assert_eq!("pending", item.status);
    }

    #[tokio::test]
    async fn append_audit_rejects_unsupported_actor_type() {
        let service = GameEventsService::new(EmptyRepo);

        let error = service
            .append_audit_record("100001", audit_command("browser"))
            .await
            .expect_err("unsupported actor must fail");

        assert_eq!("invalid", error.code());
    }

    #[tokio::test]
    async fn audit_query_clamps_page_size() {
        let query = AuditRecordQuery {
            page: Some(2),
            page_size: Some(500),
            ..Default::default()
        };

        assert_eq!(200, query.limit());
        assert_eq!(200, query.offset());
    }
}
