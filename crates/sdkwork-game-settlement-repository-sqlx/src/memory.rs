use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sdkwork_game_settlement_service::{
    CompleteSettlementJobCommand, CreateRewardIntentCommand, CreateSettlementJobCommand,
    GameRewardIntentItem, GameSettlementError, GameSettlementJobItem, GameSettlementJobPage,
    GameSettlementRepository, GameSettlementResult, RecordSettlementFailureCommand,
    SettlementDueJobQuery, StartSettlementJobCommand,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;

#[derive(Clone, Default)]
pub struct InMemoryGameSettlementRepository {
    store: Arc<RwLock<SettlementStore>>,
}

#[derive(Default)]
struct SettlementStore {
    jobs: Vec<StoredSettlementJob>,
    intents: Vec<StoredRewardIntent>,
}

#[derive(Clone)]
struct StoredSettlementJob {
    tenant_id: String,
    item: GameSettlementJobItem,
}

#[derive(Clone)]
struct StoredRewardIntent {
    tenant_id: String,
    item: GameRewardIntentItem,
}

#[async_trait]
impl GameSettlementRepository for InMemoryGameSettlementRepository {
    async fn create_job(
        &self,
        tenant_id: &str,
        command: &CreateSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        if let Some(existing) = store.jobs.iter().find(|stored| {
            stored.tenant_id == tenant_id && stored.item.idempotency_key == command.idempotency_key
        }) {
            ensure_job_idempotent(&existing.item, command)?;
            return Ok(existing.item.clone());
        }

        let timestamp = now().to_rfc3339();
        let item = GameSettlementJobItem {
            id: uuid(),
            session_id: command.session_id.clone(),
            session_result_id: command.session_result_id.clone(),
            status: "pending".into(),
            attempt_count: 0,
            idempotency_key: command.idempotency_key.clone(),
            error_code: None,
            error_detail: None,
            job_payload: command.job_payload.clone(),
            created_at: timestamp,
            started_at: None,
            completed_at: None,
            next_retry_at: None,
            version: 0,
        };
        store.jobs.push(StoredSettlementJob {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });
        Ok(item)
    }

    async fn get_job(
        &self,
        tenant_id: &str,
        job_id: &str,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        let store = self.store.read().map_err(lock_error)?;
        find_job(&store, tenant_id, job_id)
            .map(|stored| stored.item.clone())
            .ok_or_else(|| GameSettlementError::not_found("settlement job not found"))
    }

    async fn list_due_jobs(
        &self,
        tenant_id: &str,
        query: &SettlementDueJobQuery,
    ) -> GameSettlementResult<GameSettlementJobPage> {
        let store = self.store.read().map_err(lock_error)?;
        let mut items: Vec<GameSettlementJobItem> = store
            .jobs
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id && is_due(&stored.item, &query.due_at))
            .map(|stored| stored.item.clone())
            .collect();
        items.sort_by(|left, right| {
            due_sort_key(left)
                .cmp(due_sort_key(right))
                .then_with(|| left.created_at.cmp(&right.created_at))
        });
        let total = items.len() as u64;
        let limit = query.limit() as usize;
        let offset = query.offset() as usize;
        let page_items = items.into_iter().skip(offset).take(limit).collect();
        Ok(GameSettlementJobPage {
            items: page_items,
            total,
            page: query.page.unwrap_or(1),
            page_size: query.limit(),
        })
    }

    async fn start_job(
        &self,
        tenant_id: &str,
        command: &StartSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let stored = find_job_mut(&mut store, tenant_id, &command.job_id)
            .ok_or_else(|| GameSettlementError::not_found("settlement job not found"))?;
        ensure_expected_version(&stored.item, command.expected_version)?;
        if !matches!(stored.item.status.as_str(), "pending" | "retrying") {
            return Err(GameSettlementError::conflict(
                "settlement job can only start from pending or retrying",
            ));
        }
        stored.item.status = "running".into();
        stored.item.attempt_count += 1;
        stored.item.started_at = Some(now().to_rfc3339());
        stored.item.next_retry_at = None;
        stored.item.version += 1;
        Ok(stored.item.clone())
    }

    async fn record_failure(
        &self,
        tenant_id: &str,
        command: &RecordSettlementFailureCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let stored = find_job_mut(&mut store, tenant_id, &command.job_id)
            .ok_or_else(|| GameSettlementError::not_found("settlement job not found"))?;
        ensure_expected_version(&stored.item, command.expected_version)?;
        stored.item.status = if command.next_retry_at.is_some() {
            "retrying".into()
        } else {
            "failed".into()
        };
        stored.item.error_code = Some(command.error_code.clone());
        stored.item.error_detail = command.error_detail.clone();
        stored.item.next_retry_at = command.next_retry_at.clone();
        stored.item.version += 1;
        Ok(stored.item.clone())
    }

    async fn complete_job(
        &self,
        tenant_id: &str,
        command: &CompleteSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let stored = find_job_mut(&mut store, tenant_id, &command.job_id)
            .ok_or_else(|| GameSettlementError::not_found("settlement job not found"))?;
        ensure_expected_version(&stored.item, command.expected_version)?;
        stored.item.status = "succeeded".into();
        stored.item.completed_at = Some(now().to_rfc3339());
        stored.item.next_retry_at = None;
        stored.item.version += 1;
        Ok(stored.item.clone())
    }

    async fn create_reward_intent(
        &self,
        tenant_id: &str,
        command: &CreateRewardIntentCommand,
    ) -> GameSettlementResult<GameRewardIntentItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        if find_job(&store, tenant_id, &command.settlement_job_id).is_none() {
            return Err(GameSettlementError::not_found("settlement job not found"));
        }
        if let Some(existing) = store.intents.iter().find(|stored| {
            stored.tenant_id == tenant_id && stored.item.idempotency_key == command.idempotency_key
        }) {
            ensure_reward_idempotent(&existing.item, command)?;
            return Ok(existing.item.clone());
        }
        let item = GameRewardIntentItem {
            id: uuid(),
            settlement_job_id: command.settlement_job_id.clone(),
            user_id: command.user_id.clone(),
            reward_type: command.reward_type.clone(),
            external_owner: command.external_owner.clone(),
            external_reference_id: None,
            intent_payload: command.intent_payload.clone(),
            status: "pending".into(),
            idempotency_key: command.idempotency_key.clone(),
            created_at: now().to_rfc3339(),
            submitted_at: None,
            completed_at: None,
            version: 0,
        };
        store.intents.push(StoredRewardIntent {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });
        Ok(item)
    }
}

fn find_job<'a>(
    store: &'a SettlementStore,
    tenant_id: &str,
    job_id: &str,
) -> Option<&'a StoredSettlementJob> {
    store
        .jobs
        .iter()
        .find(|stored| stored.tenant_id == tenant_id && stored.item.id == job_id)
}

fn find_job_mut<'a>(
    store: &'a mut SettlementStore,
    tenant_id: &str,
    job_id: &str,
) -> Option<&'a mut StoredSettlementJob> {
    store
        .jobs
        .iter_mut()
        .find(|stored| stored.tenant_id == tenant_id && stored.item.id == job_id)
}

fn is_due(item: &GameSettlementJobItem, due_at: &str) -> bool {
    item.status == "pending"
        || (item.status == "retrying"
            && item
                .next_retry_at
                .as_deref()
                .map(|next_retry_at| next_retry_at <= due_at)
                .unwrap_or(false))
}

fn due_sort_key(item: &GameSettlementJobItem) -> &str {
    item.next_retry_at.as_deref().unwrap_or(&item.created_at)
}

fn ensure_expected_version(
    item: &GameSettlementJobItem,
    expected: Option<i64>,
) -> GameSettlementResult<()> {
    if let Some(expected) = expected {
        if item.version != expected {
            return Err(GameSettlementError::conflict(
                "settlement job version has changed",
            ));
        }
    }
    Ok(())
}

fn ensure_job_idempotent(
    existing: &GameSettlementJobItem,
    command: &CreateSettlementJobCommand,
) -> GameSettlementResult<()> {
    let same_payload = existing.session_id == command.session_id
        && existing.session_result_id == command.session_result_id
        && existing.job_payload == command.job_payload;
    if !same_payload {
        return Err(GameSettlementError::conflict(
            "idempotency_key already belongs to a different settlement job payload",
        ));
    }
    Ok(())
}

fn ensure_reward_idempotent(
    existing: &GameRewardIntentItem,
    command: &CreateRewardIntentCommand,
) -> GameSettlementResult<()> {
    let same_payload = existing.settlement_job_id == command.settlement_job_id
        && existing.user_id == command.user_id
        && existing.reward_type == command.reward_type
        && existing.external_owner == command.external_owner
        && existing.intent_payload == command.intent_payload;
    if !same_payload {
        return Err(GameSettlementError::conflict(
            "idempotency_key already belongs to a different reward intent payload",
        ));
    }
    Ok(())
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> GameSettlementError {
    GameSettlementError::invalid("settlement repository lock is poisoned")
}

#[cfg(test)]
mod tests {
    use sdkwork_game_settlement_service::{
        CreateRewardIntentCommand, CreateSettlementJobCommand, GameSettlementRepository,
        RecordSettlementFailureCommand, SettlementDueJobQuery, StartSettlementJobCommand,
    };
    use serde_json::json;

    use super::InMemoryGameSettlementRepository;

    fn job_command(idempotency_key: &str) -> CreateSettlementJobCommand {
        CreateSettlementJobCommand {
            session_id: "session-1".into(),
            session_result_id: "result-1".into(),
            idempotency_key: idempotency_key.into(),
            job_payload: json!({"source": "session_result"}),
        }
    }

    fn reward_command(job_id: &str, idempotency_key: &str) -> CreateRewardIntentCommand {
        CreateRewardIntentCommand {
            settlement_job_id: job_id.into(),
            user_id: "user-1".into(),
            reward_type: "points".into(),
            external_owner: "game".into(),
            intent_payload: json!({"points": 30}),
            idempotency_key: idempotency_key.into(),
        }
    }

    #[tokio::test]
    async fn create_job_is_idempotent() {
        let repo = InMemoryGameSettlementRepository::default();
        let command = job_command("idem-job-1");

        let first = repo.create_job("100001", &command).await.unwrap();
        let replay = repo.create_job("100001", &command).await.unwrap();

        assert_eq!(first.id, replay.id);
    }

    #[tokio::test]
    async fn failed_job_with_retry_is_returned_by_due_query() {
        let repo = InMemoryGameSettlementRepository::default();
        let job = repo
            .create_job("100001", &job_command("idem-retry"))
            .await
            .unwrap();
        let running = repo
            .start_job(
                "100001",
                &StartSettlementJobCommand {
                    job_id: job.id,
                    expected_version: Some(job.version),
                },
            )
            .await
            .unwrap();

        repo.record_failure(
            "100001",
            &RecordSettlementFailureCommand {
                job_id: running.id,
                error_code: "downstream_timeout".into(),
                error_detail: None,
                next_retry_at: Some("2026-07-07T00:05:00Z".into()),
                expected_version: Some(running.version),
            },
        )
        .await
        .unwrap();

        let page = repo
            .list_due_jobs(
                "100001",
                &SettlementDueJobQuery {
                    due_at: "2026-07-07T00:10:00Z".into(),
                    page: Some(1),
                    page_size: Some(20),
                },
            )
            .await
            .unwrap();

        assert_eq!(1, page.total);
        assert_eq!("retrying", page.items[0].status);
    }

    #[tokio::test]
    async fn reward_intent_is_idempotent_and_conflict_checked() {
        let repo = InMemoryGameSettlementRepository::default();
        let job = repo
            .create_job("100001", &job_command("idem-reward-job"))
            .await
            .unwrap();
        let command = reward_command(&job.id, "idem-reward-1");

        let first = repo.create_reward_intent("100001", &command).await.unwrap();
        let replay = repo.create_reward_intent("100001", &command).await.unwrap();
        assert_eq!(first.id, replay.id);

        let mut conflict = command;
        conflict.user_id = "user-2".into();
        let error = repo
            .create_reward_intent("100001", &conflict)
            .await
            .unwrap_err();
        assert_eq!("conflict", error.code());
    }
}
