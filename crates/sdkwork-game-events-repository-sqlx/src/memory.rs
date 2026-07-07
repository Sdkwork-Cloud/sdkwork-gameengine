use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sdkwork_game_events_service::{
    AppendAuditRecordCommand, AppendGameEngineEventCommand, AuditRecordItem, AuditRecordPage,
    AuditRecordQuery, GameEngineEventItem, GameEngineEventPage, GameEventError, GameEventResult,
    GameEventsRepository, MarkGameEngineEventFailedCommand, MarkGameEngineEventPublishedCommand,
    PendingGameEngineEventQuery,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;

#[derive(Clone, Default)]
pub struct InMemoryGameEventsRepository {
    store: Arc<RwLock<EventStore>>,
}

#[derive(Default)]
struct EventStore {
    events: Vec<StoredEvent>,
    audits: Vec<StoredAudit>,
}

#[derive(Clone)]
struct StoredEvent {
    tenant_id: String,
    item: GameEngineEventItem,
}

#[derive(Clone)]
struct StoredAudit {
    tenant_id: String,
    item: AuditRecordItem,
}

#[async_trait]
impl GameEventsRepository for InMemoryGameEventsRepository {
    async fn append_event(
        &self,
        tenant_id: &str,
        command: &AppendGameEngineEventCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        if let Some(existing) = store.events.iter().find(|stored| {
            stored.tenant_id == tenant_id && stored.item.idempotency_key == command.idempotency_key
        }) {
            ensure_event_idempotent(&existing.item, command)?;
            return Ok(existing.item.clone());
        }
        let item = GameEngineEventItem {
            id: uuid(),
            event_type: command.event_type.clone(),
            aggregate_type: command.aggregate_type.clone(),
            aggregate_id: command.aggregate_id.clone(),
            idempotency_key: command.idempotency_key.clone(),
            event_payload: command.event_payload.clone(),
            status: "pending".into(),
            trace_id: command.trace_id.clone(),
            created_at: now().to_rfc3339(),
            published_at: None,
            next_retry_at: None,
            version: 0,
        };
        store.events.push(StoredEvent {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });
        Ok(item)
    }

    async fn get_event(
        &self,
        tenant_id: &str,
        event_id: &str,
    ) -> GameEventResult<GameEngineEventItem> {
        let store = self.store.read().map_err(lock_error)?;
        find_event(&store, tenant_id, event_id)
            .map(|stored| stored.item.clone())
            .ok_or_else(|| GameEventError::not_found("engine event not found"))
    }

    async fn list_pending_events(
        &self,
        tenant_id: &str,
        query: &PendingGameEngineEventQuery,
    ) -> GameEventResult<GameEngineEventPage> {
        let store = self.store.read().map_err(lock_error)?;
        let mut items: Vec<GameEngineEventItem> = store
            .events
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id && is_due(&stored.item, &query.due_at))
            .map(|stored| stored.item.clone())
            .collect();
        items.sort_by(|left, right| {
            event_due_sort_key(left)
                .cmp(event_due_sort_key(right))
                .then_with(|| left.created_at.cmp(&right.created_at))
        });
        let total = items.len() as u64;
        let limit = query.limit() as usize;
        let offset = query.offset() as usize;
        Ok(GameEngineEventPage {
            items: items.into_iter().skip(offset).take(limit).collect(),
            total,
            page: query.page.unwrap_or(1),
            page_size: query.limit(),
        })
    }

    async fn mark_event_published(
        &self,
        tenant_id: &str,
        command: &MarkGameEngineEventPublishedCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let stored = find_event_mut(&mut store, tenant_id, &command.event_id)
            .ok_or_else(|| GameEventError::not_found("engine event not found"))?;
        ensure_expected_version(&stored.item, command.expected_version)?;
        stored.item.status = "published".into();
        stored.item.published_at = Some(now().to_rfc3339());
        stored.item.next_retry_at = None;
        stored.item.version += 1;
        Ok(stored.item.clone())
    }

    async fn mark_event_failed(
        &self,
        tenant_id: &str,
        command: &MarkGameEngineEventFailedCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let stored = find_event_mut(&mut store, tenant_id, &command.event_id)
            .ok_or_else(|| GameEventError::not_found("engine event not found"))?;
        ensure_expected_version(&stored.item, command.expected_version)?;
        stored.item.status = if command.dead_letter {
            "dead_letter".into()
        } else {
            "failed".into()
        };
        stored.item.next_retry_at = command.next_retry_at.clone();
        stored.item.version += 1;
        Ok(stored.item.clone())
    }

    async fn append_audit_record(
        &self,
        tenant_id: &str,
        command: &AppendAuditRecordCommand,
    ) -> GameEventResult<AuditRecordItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let item = AuditRecordItem {
            id: uuid(),
            actor_type: command.actor_type.clone(),
            actor_id: command.actor_id.clone(),
            action: command.action.clone(),
            target_type: command.target_type.clone(),
            target_id: command.target_id.clone(),
            reason_code: command.reason_code.clone(),
            before_snapshot: command.before_snapshot.clone(),
            after_snapshot: command.after_snapshot.clone(),
            trace_id: command.trace_id.clone(),
            created_at: now().to_rfc3339(),
            version: 0,
        };
        store.audits.push(StoredAudit {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });
        Ok(item)
    }

    async fn search_audit_records(
        &self,
        tenant_id: &str,
        query: &AuditRecordQuery,
    ) -> GameEventResult<AuditRecordPage> {
        let store = self.store.read().map_err(lock_error)?;
        let mut items: Vec<AuditRecordItem> = store
            .audits
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id && audit_matches(&stored.item, query))
            .map(|stored| stored.item.clone())
            .collect();
        items.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        let total = items.len() as u64;
        let limit = query.limit() as usize;
        let offset = query.offset() as usize;
        Ok(AuditRecordPage {
            items: items.into_iter().skip(offset).take(limit).collect(),
            total,
            page: query.page.unwrap_or(1),
            page_size: query.limit(),
        })
    }
}

fn find_event<'a>(
    store: &'a EventStore,
    tenant_id: &str,
    event_id: &str,
) -> Option<&'a StoredEvent> {
    store
        .events
        .iter()
        .find(|stored| stored.tenant_id == tenant_id && stored.item.id == event_id)
}

fn find_event_mut<'a>(
    store: &'a mut EventStore,
    tenant_id: &str,
    event_id: &str,
) -> Option<&'a mut StoredEvent> {
    store
        .events
        .iter_mut()
        .find(|stored| stored.tenant_id == tenant_id && stored.item.id == event_id)
}

fn is_due(item: &GameEngineEventItem, due_at: &str) -> bool {
    item.status == "pending"
        || (item.status == "failed"
            && item
                .next_retry_at
                .as_deref()
                .map(|next_retry_at| next_retry_at <= due_at)
                .unwrap_or(false))
}

fn event_due_sort_key(item: &GameEngineEventItem) -> &str {
    item.next_retry_at.as_deref().unwrap_or(&item.created_at)
}

fn audit_matches(item: &AuditRecordItem, query: &AuditRecordQuery) -> bool {
    query
        .target_type
        .as_ref()
        .map(|value| item.target_type == *value)
        .unwrap_or(true)
        && query
            .target_id
            .as_ref()
            .map(|value| item.target_id == *value)
            .unwrap_or(true)
        && query
            .actor_type
            .as_ref()
            .map(|value| item.actor_type == *value)
            .unwrap_or(true)
        && query
            .actor_id
            .as_ref()
            .map(|value| item.actor_id.as_deref() == Some(value.as_str()))
            .unwrap_or(true)
        && query
            .action
            .as_ref()
            .map(|value| item.action == *value)
            .unwrap_or(true)
}

fn ensure_event_idempotent(
    existing: &GameEngineEventItem,
    command: &AppendGameEngineEventCommand,
) -> GameEventResult<()> {
    let same_payload = existing.event_type == command.event_type
        && existing.aggregate_type == command.aggregate_type
        && existing.aggregate_id == command.aggregate_id
        && existing.event_payload == command.event_payload
        && existing.trace_id == command.trace_id;
    if !same_payload {
        return Err(GameEventError::conflict(
            "idempotency_key already belongs to a different engine event payload",
        ));
    }
    Ok(())
}

fn ensure_expected_version(
    item: &GameEngineEventItem,
    expected: Option<i64>,
) -> GameEventResult<()> {
    if let Some(expected) = expected {
        if item.version != expected {
            return Err(GameEventError::conflict("engine event version has changed"));
        }
    }
    Ok(())
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> GameEventError {
    GameEventError::invalid("events repository lock is poisoned")
}

#[cfg(test)]
mod tests {
    use sdkwork_game_events_service::{
        AppendAuditRecordCommand, AppendGameEngineEventCommand, AuditRecordQuery,
        GameEventsRepository, MarkGameEngineEventPublishedCommand, PendingGameEngineEventQuery,
    };
    use serde_json::json;

    use super::InMemoryGameEventsRepository;

    fn event_command(idempotency_key: &str) -> AppendGameEngineEventCommand {
        AppendGameEngineEventCommand {
            event_type: "game.session.completed".into(),
            aggregate_type: "session".into(),
            aggregate_id: "session-1".into(),
            idempotency_key: idempotency_key.into(),
            event_payload: json!({"sessionId": "session-1"}),
            trace_id: "trace-1".into(),
        }
    }

    fn audit_command(target_id: &str) -> AppendAuditRecordCommand {
        AppendAuditRecordCommand {
            actor_type: "operator".into(),
            actor_id: Some("operator-1".into()),
            action: "session.result.corrected".into(),
            target_type: "session".into(),
            target_id: target_id.into(),
            reason_code: Some("manual_review".into()),
            before_snapshot: json!({"score": 1}),
            after_snapshot: json!({"score": 2}),
            trace_id: "trace-1".into(),
        }
    }

    #[tokio::test]
    async fn append_event_is_idempotent_and_can_be_published() {
        let repo = InMemoryGameEventsRepository::default();
        let command = event_command("idem-event-1");

        let first = repo.append_event("100001", &command).await.unwrap();
        let replay = repo.append_event("100001", &command).await.unwrap();
        assert_eq!(first.id, replay.id);

        let published = repo
            .mark_event_published(
                "100001",
                &MarkGameEngineEventPublishedCommand {
                    event_id: first.id,
                    expected_version: Some(first.version),
                },
            )
            .await
            .unwrap();

        assert_eq!("published", published.status);
        assert!(published.published_at.is_some());
    }

    #[tokio::test]
    async fn pending_events_are_paginated() {
        let repo = InMemoryGameEventsRepository::default();
        repo.append_event("100001", &event_command("idem-event-1"))
            .await
            .unwrap();
        repo.append_event("100001", &event_command("idem-event-2"))
            .await
            .unwrap();

        let page = repo
            .list_pending_events(
                "100001",
                &PendingGameEngineEventQuery {
                    due_at: "2026-07-07T00:00:00Z".into(),
                    page: Some(1),
                    page_size: Some(1),
                },
            )
            .await
            .unwrap();

        assert_eq!(2, page.total);
        assert_eq!(1, page.items.len());
    }

    #[tokio::test]
    async fn audit_search_filters_by_target() {
        let repo = InMemoryGameEventsRepository::default();
        repo.append_audit_record("100001", &audit_command("session-1"))
            .await
            .unwrap();
        repo.append_audit_record("100001", &audit_command("session-2"))
            .await
            .unwrap();

        let page = repo
            .search_audit_records(
                "100001",
                &AuditRecordQuery {
                    target_type: Some("session".into()),
                    target_id: Some("session-2".into()),
                    page: Some(1),
                    page_size: Some(20),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(1, page.total);
        assert_eq!("session-2", page.items[0].target_id);
    }
}
