use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_events_service::{
    AppendAuditRecordCommand, AppendGameEngineEventCommand, AuditRecordItem, AuditRecordPage,
    AuditRecordQuery, GameEngineEventItem, GameEngineEventPage, GameEventError, GameEventResult,
    GameEventsRepository, MarkGameEngineEventFailedCommand, MarkGameEngineEventPublishedCommand,
    PendingGameEngineEventQuery,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxGameEventsRepository {
    pool: DatabasePool,
}

impl SqlxGameEventsRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameEventsRepository for SqlxGameEventsRepository {
    async fn append_event(
        &self,
        tenant_id: &str,
        command: &AppendGameEngineEventCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        validate_tenant(tenant_id)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                append_event_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                append_event_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn get_event(
        &self,
        tenant_id: &str,
        event_id: &str,
    ) -> GameEventResult<GameEngineEventItem> {
        validate_tenant(tenant_id)?;
        if is_blank(Some(event_id)) {
            return Err(GameEventError::invalid("event_id is required"));
        }
        match &self.pool {
            DatabasePool::Postgres(pool, _) => get_event_postgres(pool, tenant_id, event_id).await,
            DatabasePool::Sqlite(pool, _) => get_event_sqlite(pool, tenant_id, event_id).await,
        }
    }

    async fn list_pending_events(
        &self,
        tenant_id: &str,
        query: &PendingGameEngineEventQuery,
    ) -> GameEventResult<GameEngineEventPage> {
        validate_tenant(tenant_id)?;
        let limit = query.limit() as i64;
        let offset = query.offset() as i64;
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                list_pending_postgres(pool, tenant_id, &query.due_at, limit, offset).await
            }
            DatabasePool::Sqlite(pool, _) => {
                list_pending_sqlite(pool, tenant_id, &query.due_at, limit, offset).await
            }
        }
    }

    async fn mark_event_published(
        &self,
        tenant_id: &str,
        command: &MarkGameEngineEventPublishedCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        validate_tenant(tenant_id)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                mark_published_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                mark_published_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn mark_event_failed(
        &self,
        tenant_id: &str,
        command: &MarkGameEngineEventFailedCommand,
    ) -> GameEventResult<GameEngineEventItem> {
        validate_tenant(tenant_id)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                mark_failed_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                mark_failed_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn append_audit_record(
        &self,
        tenant_id: &str,
        command: &AppendAuditRecordCommand,
    ) -> GameEventResult<AuditRecordItem> {
        validate_tenant(tenant_id)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                append_audit_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                append_audit_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn search_audit_records(
        &self,
        tenant_id: &str,
        query: &AuditRecordQuery,
    ) -> GameEventResult<AuditRecordPage> {
        validate_tenant(tenant_id)?;
        let limit = query.limit() as i64;
        let offset = query.offset() as i64;
        let params = AuditSearchParams::from_query(tenant_id, query, limit, offset);
        match &self.pool {
            DatabasePool::Postgres(pool, _) => search_audit_postgres(pool, params).await,
            DatabasePool::Sqlite(pool, _) => search_audit_sqlite(pool, params).await,
        }
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: String,
    event_type: String,
    aggregate_type: String,
    aggregate_id: String,
    idempotency_key: String,
    event_payload: String,
    status: String,
    trace_id: String,
    created_at: String,
    published_at: Option<String>,
    next_retry_at: Option<String>,
    version: i64,
}

impl EventRow {
    fn into_item(self) -> GameEventResult<GameEngineEventItem> {
        Ok(GameEngineEventItem {
            id: self.id,
            event_type: self.event_type,
            aggregate_type: self.aggregate_type,
            aggregate_id: self.aggregate_id,
            idempotency_key: self.idempotency_key,
            event_payload: parse_json(&self.event_payload)?,
            status: self.status,
            trace_id: self.trace_id,
            created_at: self.created_at,
            published_at: self.published_at,
            next_retry_at: self.next_retry_at,
            version: self.version,
        })
    }
}

#[derive(sqlx::FromRow)]
struct AuditRow {
    id: String,
    actor_type: String,
    actor_id: Option<String>,
    action: String,
    target_type: String,
    target_id: String,
    reason_code: Option<String>,
    before_snapshot: String,
    after_snapshot: String,
    trace_id: String,
    created_at: String,
    version: i64,
}

impl AuditRow {
    fn into_item(self) -> GameEventResult<AuditRecordItem> {
        Ok(AuditRecordItem {
            id: self.id,
            actor_type: self.actor_type,
            actor_id: self.actor_id,
            action: self.action,
            target_type: self.target_type,
            target_id: self.target_id,
            reason_code: self.reason_code,
            before_snapshot: parse_json(&self.before_snapshot)?,
            after_snapshot: parse_json(&self.after_snapshot)?,
            trace_id: self.trace_id,
            created_at: self.created_at,
            version: self.version,
        })
    }
}

#[derive(Clone, Copy)]
struct AuditSearchParams<'a> {
    tenant_id: &'a str,
    target_type: Option<&'a str>,
    target_id: Option<&'a str>,
    actor_type: Option<&'a str>,
    actor_id: Option<&'a str>,
    action: Option<&'a str>,
    limit: i64,
    offset: i64,
}

impl<'a> AuditSearchParams<'a> {
    fn from_query(
        tenant_id: &'a str,
        query: &'a AuditRecordQuery,
        limit: i64,
        offset: i64,
    ) -> Self {
        Self {
            tenant_id,
            target_type: query.target_type.as_deref(),
            target_id: query.target_id.as_deref(),
            actor_type: query.actor_type.as_deref(),
            actor_id: query.actor_id.as_deref(),
            action: query.action.as_deref(),
            limit,
            offset,
        }
    }
}

const EVENT_COLUMNS_POSTGRES: &str = "id, event_type, aggregate_type, aggregate_id, \
idempotency_key, event_payload::text AS event_payload, status, trace_id, created_at, \
published_at, next_retry_at, version";
const EVENT_COLUMNS_SQLITE: &str = "id, event_type, aggregate_type, aggregate_id, \
idempotency_key, event_payload, status, trace_id, created_at, published_at, next_retry_at, version";
const AUDIT_COLUMNS_POSTGRES: &str = "id, actor_type, actor_id, action, target_type, target_id, \
reason_code, before_snapshot::text AS before_snapshot, after_snapshot::text AS after_snapshot, \
trace_id, created_at, version";
const AUDIT_COLUMNS_SQLITE: &str = "id, actor_type, actor_id, action, target_type, target_id, \
reason_code, before_snapshot, after_snapshot, trace_id, created_at, version";

async fn append_event_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &AppendGameEngineEventCommand,
    timestamp: &str,
) -> GameEventResult<GameEngineEventItem> {
    if let Some(existing) = get_existing_event_postgres(pool, tenant_id, command).await? {
        return Ok(existing);
    }
    let id = uuid();
    let payload = command.event_payload.to_string();
    let row = sqlx::query_as::<_, EventRow>(&format!(
        "INSERT INTO game_engine_event \
         (id, uuid, tenant_id, organization_id, event_type, aggregate_type, aggregate_id, \
          idempotency_key, event_payload, trace_id, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8::jsonb, $9, $10, $10) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING RETURNING {EVENT_COLUMNS_POSTGRES}",
    ))
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.event_type)
    .bind(&command.aggregate_type)
    .bind(&command.aggregate_id)
    .bind(&command.idempotency_key)
    .bind(payload)
    .bind(&command.trace_id)
    .bind(timestamp)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    if let Some(row) = row {
        return row.into_item();
    }
    get_existing_event_postgres(pool, tenant_id, command)
        .await?
        .ok_or_else(|| GameEventError::conflict("engine event idempotency conflict"))
}

async fn append_event_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &AppendGameEngineEventCommand,
    timestamp: &str,
) -> GameEventResult<GameEngineEventItem> {
    if let Some(existing) = get_existing_event_sqlite(pool, tenant_id, command).await? {
        return Ok(existing);
    }
    let id = uuid();
    let result = sqlx::query(
        "INSERT INTO game_engine_event \
         (id, uuid, tenant_id, organization_id, event_type, aggregate_type, aggregate_id, \
          idempotency_key, event_payload, trace_id, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING",
    )
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.event_type)
    .bind(&command.aggregate_type)
    .bind(&command.aggregate_id)
    .bind(&command.idempotency_key)
    .bind(command.event_payload.to_string())
    .bind(&command.trace_id)
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;
    if result.rows_affected() == 0 {
        return get_existing_event_sqlite(pool, tenant_id, command)
            .await?
            .ok_or_else(|| GameEventError::conflict("engine event idempotency conflict"));
    }
    get_event_sqlite(pool, tenant_id, &id).await
}

async fn get_existing_event_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &AppendGameEngineEventCommand,
) -> GameEventResult<Option<GameEngineEventItem>> {
    let row = sqlx::query_as::<_, EventRow>(&format!(
        "SELECT {EVENT_COLUMNS_POSTGRES} FROM game_engine_event \
         WHERE tenant_id = $1 AND idempotency_key = $2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    map_existing_event(row, command)
}

async fn get_existing_event_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &AppendGameEngineEventCommand,
) -> GameEventResult<Option<GameEngineEventItem>> {
    let row = sqlx::query_as::<_, EventRow>(&format!(
        "SELECT {EVENT_COLUMNS_SQLITE} FROM game_engine_event \
         WHERE tenant_id = ?1 AND idempotency_key = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    map_existing_event(row, command)
}

fn map_existing_event(
    row: Option<EventRow>,
    command: &AppendGameEngineEventCommand,
) -> GameEventResult<Option<GameEngineEventItem>> {
    let Some(row) = row else {
        return Ok(None);
    };
    let item = row.into_item()?;
    ensure_event_idempotent(&item, command)?;
    Ok(Some(item))
}

async fn get_event_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    event_id: &str,
) -> GameEventResult<GameEngineEventItem> {
    let row = sqlx::query_as::<_, EventRow>(&format!(
        "SELECT {EVENT_COLUMNS_POSTGRES} FROM game_engine_event \
         WHERE tenant_id = $1 AND id = $2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameEventError::not_found("engine event not found"))?;
    row.into_item()
}

async fn get_event_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    event_id: &str,
) -> GameEventResult<GameEngineEventItem> {
    let row = sqlx::query_as::<_, EventRow>(&format!(
        "SELECT {EVENT_COLUMNS_SQLITE} FROM game_engine_event \
         WHERE tenant_id = ?1 AND id = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameEventError::not_found("engine event not found"))?;
    row.into_item()
}

async fn list_pending_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    due_at: &str,
    limit: i64,
    offset: i64,
) -> GameEventResult<GameEngineEventPage> {
    let filter = "tenant_id = $1 AND (status = 'pending' OR \
        (status = 'failed' AND next_retry_at IS NOT NULL AND next_retry_at::timestamptz <= $2::timestamptz))";
    let rows = sqlx::query_as::<_, EventRow>(&format!(
        "SELECT {EVENT_COLUMNS_POSTGRES} FROM game_engine_event \
         WHERE {filter} \
         ORDER BY COALESCE(next_retry_at, created_at), created_at LIMIT $3 OFFSET $4",
    ))
    .bind(tenant_id)
    .bind(due_at)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM game_engine_event WHERE {filter}",
    ))
    .bind(tenant_id)
    .bind(due_at)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    event_page_from_rows(rows, total, limit, offset)
}

async fn list_pending_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    due_at: &str,
    limit: i64,
    offset: i64,
) -> GameEventResult<GameEngineEventPage> {
    let filter = "tenant_id = ?1 AND (status = 'pending' OR \
        (status = 'failed' AND next_retry_at IS NOT NULL AND next_retry_at <= ?2))";
    let rows = sqlx::query_as::<_, EventRow>(&format!(
        "SELECT {EVENT_COLUMNS_SQLITE} FROM game_engine_event \
         WHERE {filter} \
         ORDER BY COALESCE(next_retry_at, created_at), created_at LIMIT ?3 OFFSET ?4",
    ))
    .bind(tenant_id)
    .bind(due_at)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM game_engine_event WHERE {filter}",
    ))
    .bind(tenant_id)
    .bind(due_at)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    event_page_from_rows(rows, total, limit, offset)
}

async fn mark_published_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &MarkGameEngineEventPublishedCommand,
    timestamp: &str,
) -> GameEventResult<GameEngineEventItem> {
    let row = if let Some(expected_version) = command.expected_version {
        sqlx::query_as::<_, EventRow>(&format!(
            "UPDATE game_engine_event SET status = 'published', published_at = $4, \
             next_retry_at = NULL, updated_at = $4, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $3 RETURNING {EVENT_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.event_id)
        .bind(expected_version)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_as::<_, EventRow>(&format!(
            "UPDATE game_engine_event SET status = 'published', published_at = $3, \
             next_retry_at = NULL, updated_at = $3, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 RETURNING {EVENT_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.event_id)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    }
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameEventError::conflict("engine event version has changed"))?;
    row.into_item()
}

async fn mark_published_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &MarkGameEngineEventPublishedCommand,
    timestamp: &str,
) -> GameEventResult<GameEngineEventItem> {
    let result = if let Some(expected_version) = command.expected_version {
        sqlx::query(
            "UPDATE game_engine_event SET status = 'published', published_at = ?4, \
             next_retry_at = NULL, updated_at = ?4, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND version = ?3",
        )
        .bind(tenant_id)
        .bind(&command.event_id)
        .bind(expected_version)
        .bind(timestamp)
        .execute(pool)
        .await
    } else {
        sqlx::query(
            "UPDATE game_engine_event SET status = 'published', published_at = ?3, \
             next_retry_at = NULL, updated_at = ?3, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2",
        )
        .bind(tenant_id)
        .bind(&command.event_id)
        .bind(timestamp)
        .execute(pool)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_rows_affected(result.rows_affected(), "engine event version has changed")?;
    get_event_sqlite(pool, tenant_id, &command.event_id).await
}

async fn mark_failed_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &MarkGameEngineEventFailedCommand,
    timestamp: &str,
) -> GameEventResult<GameEngineEventItem> {
    let status = failed_status(command);
    let row = if let Some(expected_version) = command.expected_version {
        sqlx::query_as::<_, EventRow>(&format!(
            "UPDATE game_engine_event SET status = $4, next_retry_at = $5, updated_at = $6, \
             version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $3 RETURNING {EVENT_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.event_id)
        .bind(expected_version)
        .bind(status)
        .bind(&command.next_retry_at)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_as::<_, EventRow>(&format!(
            "UPDATE game_engine_event SET status = $3, next_retry_at = $4, updated_at = $5, \
             version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 RETURNING {EVENT_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.event_id)
        .bind(status)
        .bind(&command.next_retry_at)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    }
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameEventError::conflict("engine event version has changed"))?;
    row.into_item()
}

async fn mark_failed_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &MarkGameEngineEventFailedCommand,
    timestamp: &str,
) -> GameEventResult<GameEngineEventItem> {
    let status = failed_status(command);
    let result = if let Some(expected_version) = command.expected_version {
        sqlx::query(
            "UPDATE game_engine_event SET status = ?4, next_retry_at = ?5, updated_at = ?6, \
             version = version + 1 WHERE tenant_id = ?1 AND id = ?2 AND version = ?3",
        )
        .bind(tenant_id)
        .bind(&command.event_id)
        .bind(expected_version)
        .bind(status)
        .bind(&command.next_retry_at)
        .bind(timestamp)
        .execute(pool)
        .await
    } else {
        sqlx::query(
            "UPDATE game_engine_event SET status = ?3, next_retry_at = ?4, updated_at = ?5, \
             version = version + 1 WHERE tenant_id = ?1 AND id = ?2",
        )
        .bind(tenant_id)
        .bind(&command.event_id)
        .bind(status)
        .bind(&command.next_retry_at)
        .bind(timestamp)
        .execute(pool)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_rows_affected(result.rows_affected(), "engine event version has changed")?;
    get_event_sqlite(pool, tenant_id, &command.event_id).await
}

async fn append_audit_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &AppendAuditRecordCommand,
    timestamp: &str,
) -> GameEventResult<AuditRecordItem> {
    let id = uuid();
    let row = sqlx::query_as::<_, AuditRow>(&format!(
        "INSERT INTO game_audit_record \
         (id, uuid, tenant_id, organization_id, actor_type, actor_id, action, target_type, \
          target_id, reason_code, before_snapshot, after_snapshot, trace_id, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, $9, $10::jsonb, $11::jsonb, $12, $13, $13) \
         RETURNING {AUDIT_COLUMNS_POSTGRES}",
    ))
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.actor_type)
    .bind(&command.actor_id)
    .bind(&command.action)
    .bind(&command.target_type)
    .bind(&command.target_id)
    .bind(&command.reason_code)
    .bind(command.before_snapshot.to_string())
    .bind(command.after_snapshot.to_string())
    .bind(&command.trace_id)
    .bind(timestamp)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    row.into_item()
}

async fn append_audit_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &AppendAuditRecordCommand,
    timestamp: &str,
) -> GameEventResult<AuditRecordItem> {
    let id = uuid();
    sqlx::query(
        "INSERT INTO game_audit_record \
         (id, uuid, tenant_id, organization_id, actor_type, actor_id, action, target_type, \
          target_id, reason_code, before_snapshot, after_snapshot, trace_id, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)",
    )
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.actor_type)
    .bind(&command.actor_id)
    .bind(&command.action)
    .bind(&command.target_type)
    .bind(&command.target_id)
    .bind(&command.reason_code)
    .bind(command.before_snapshot.to_string())
    .bind(command.after_snapshot.to_string())
    .bind(&command.trace_id)
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;
    get_audit_sqlite(pool, tenant_id, &id).await
}

async fn get_audit_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    audit_id: &str,
) -> GameEventResult<AuditRecordItem> {
    let row = sqlx::query_as::<_, AuditRow>(&format!(
        "SELECT {AUDIT_COLUMNS_SQLITE} FROM game_audit_record \
         WHERE tenant_id = ?1 AND id = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(audit_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameEventError::not_found("audit record not found"))?;
    row.into_item()
}

async fn search_audit_postgres(
    pool: &sqlx::PgPool,
    params: AuditSearchParams<'_>,
) -> GameEventResult<AuditRecordPage> {
    let filter = "tenant_id = $1 \
        AND ($2::text IS NULL OR target_type = $2) \
        AND ($3::text IS NULL OR target_id = $3) \
        AND ($4::text IS NULL OR actor_type = $4) \
        AND ($5::text IS NULL OR actor_id = $5) \
        AND ($6::text IS NULL OR action = $6)";
    let rows = sqlx::query_as::<_, AuditRow>(&format!(
        "SELECT {AUDIT_COLUMNS_POSTGRES} FROM game_audit_record \
         WHERE {filter} ORDER BY created_at DESC LIMIT $7 OFFSET $8",
    ))
    .bind(params.tenant_id)
    .bind(params.target_type)
    .bind(params.target_id)
    .bind(params.actor_type)
    .bind(params.actor_id)
    .bind(params.action)
    .bind(params.limit)
    .bind(params.offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM game_audit_record WHERE {filter}",
    ))
    .bind(params.tenant_id)
    .bind(params.target_type)
    .bind(params.target_id)
    .bind(params.actor_type)
    .bind(params.actor_id)
    .bind(params.action)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    audit_page_from_rows(rows, total, params.limit, params.offset)
}

async fn search_audit_sqlite(
    pool: &sqlx::SqlitePool,
    params: AuditSearchParams<'_>,
) -> GameEventResult<AuditRecordPage> {
    let filter = "tenant_id = ?1 \
        AND (?2 IS NULL OR target_type = ?2) \
        AND (?3 IS NULL OR target_id = ?3) \
        AND (?4 IS NULL OR actor_type = ?4) \
        AND (?5 IS NULL OR actor_id = ?5) \
        AND (?6 IS NULL OR action = ?6)";
    let rows = sqlx::query_as::<_, AuditRow>(&format!(
        "SELECT {AUDIT_COLUMNS_SQLITE} FROM game_audit_record \
         WHERE {filter} ORDER BY created_at DESC LIMIT ?7 OFFSET ?8",
    ))
    .bind(params.tenant_id)
    .bind(params.target_type)
    .bind(params.target_id)
    .bind(params.actor_type)
    .bind(params.actor_id)
    .bind(params.action)
    .bind(params.limit)
    .bind(params.offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM game_audit_record WHERE {filter}",
    ))
    .bind(params.tenant_id)
    .bind(params.target_type)
    .bind(params.target_id)
    .bind(params.actor_type)
    .bind(params.actor_id)
    .bind(params.action)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    audit_page_from_rows(rows, total, params.limit, params.offset)
}

fn event_page_from_rows(
    rows: Vec<EventRow>,
    total: i64,
    limit: i64,
    offset: i64,
) -> GameEventResult<GameEngineEventPage> {
    let items = rows
        .into_iter()
        .map(EventRow::into_item)
        .collect::<GameEventResult<Vec<_>>>()?;
    Ok(GameEngineEventPage {
        items,
        total: total as u64,
        page: ((offset / limit) + 1) as u32,
        page_size: limit as u32,
    })
}

fn audit_page_from_rows(
    rows: Vec<AuditRow>,
    total: i64,
    limit: i64,
    offset: i64,
) -> GameEventResult<AuditRecordPage> {
    let items = rows
        .into_iter()
        .map(AuditRow::into_item)
        .collect::<GameEventResult<Vec<_>>>()?;
    Ok(AuditRecordPage {
        items,
        total: total as u64,
        page: ((offset / limit) + 1) as u32,
        page_size: limit as u32,
    })
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

fn ensure_rows_affected(rows_affected: u64, message: &str) -> GameEventResult<()> {
    if rows_affected == 0 {
        return Err(GameEventError::conflict(message));
    }
    Ok(())
}

fn failed_status(command: &MarkGameEngineEventFailedCommand) -> &'static str {
    if command.dead_letter {
        "dead_letter"
    } else {
        "failed"
    }
}

fn parse_json(value: &str) -> GameEventResult<serde_json::Value> {
    serde_json::from_str(value).map_err(|error| GameEventError::invalid(error.to_string()))
}

fn validate_tenant(tenant_id: &str) -> GameEventResult<()> {
    if is_blank(Some(tenant_id)) {
        return Err(GameEventError::invalid("tenant_id is required"));
    }
    Ok(())
}

fn map_sqlx_error(error: sqlx::Error) -> GameEventError {
    GameEventError::invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
    use sdkwork_database_sqlx::create_pool_from_config;
    use serde_json::json;

    use super::*;

    async fn sqlite_repo() -> SqlxGameEventsRepository {
        let pool = create_pool_from_config(DatabaseConfig {
            engine: DatabaseEngine::Sqlite,
            url: "sqlite::memory:".into(),
            max_connections: 1,
            ..Default::default()
        })
        .await
        .unwrap();
        pool.execute_raw(
            "CREATE TABLE game_engine_event (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              event_type TEXT NOT NULL,
              aggregate_type TEXT NOT NULL,
              aggregate_id TEXT NOT NULL,
              idempotency_key TEXT NOT NULL,
              event_payload TEXT NOT NULL DEFAULT '{}',
              status TEXT NOT NULL DEFAULT 'pending',
              trace_id TEXT NOT NULL,
              created_at TEXT NOT NULL,
              published_at TEXT,
              next_retry_at TEXT,
              updated_at TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 0,
              UNIQUE (tenant_id, idempotency_key)
            );
            CREATE TABLE game_audit_record (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              actor_type TEXT NOT NULL,
              actor_id TEXT,
              action TEXT NOT NULL,
              target_type TEXT NOT NULL,
              target_id TEXT NOT NULL,
              reason_code TEXT,
              before_snapshot TEXT NOT NULL DEFAULT '{}',
              after_snapshot TEXT NOT NULL DEFAULT '{}',
              trace_id TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 0
            );",
        )
        .await
        .unwrap();
        SqlxGameEventsRepository::new(pool)
    }

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
    async fn sqlite_append_event_is_idempotent_and_conflict_checked() {
        let repository = sqlite_repo().await;
        let command = event_command("idem-event-sqlite");

        let first = repository.append_event("100001", &command).await.unwrap();
        let replay = repository.append_event("100001", &command).await.unwrap();
        assert_eq!(first.id, replay.id);

        let mut conflict = command;
        conflict.aggregate_id = "session-2".into();
        let error = repository
            .append_event("100001", &conflict)
            .await
            .unwrap_err();
        assert_eq!("conflict", error.code());
    }

    #[tokio::test]
    async fn sqlite_event_outbox_lists_pending_and_publishes() {
        let repository = sqlite_repo().await;
        let event = repository
            .append_event("100001", &event_command("idem-event-pending"))
            .await
            .unwrap();

        let page = repository
            .list_pending_events(
                "100001",
                &PendingGameEngineEventQuery {
                    due_at: "2026-07-07T00:00:00Z".into(),
                    page: Some(1),
                    page_size: Some(20),
                },
            )
            .await
            .unwrap();
        assert_eq!(1, page.total);

        let published = repository
            .mark_event_published(
                "100001",
                &MarkGameEngineEventPublishedCommand {
                    event_id: event.id,
                    expected_version: Some(event.version),
                },
            )
            .await
            .unwrap();

        assert_eq!("published", published.status);
        assert!(published.published_at.is_some());
    }

    #[tokio::test]
    async fn sqlite_audit_append_and_search_filters_by_target() {
        let repository = sqlite_repo().await;
        repository
            .append_audit_record("100001", &audit_command("session-1"))
            .await
            .unwrap();
        repository
            .append_audit_record("100001", &audit_command("session-2"))
            .await
            .unwrap();

        let page = repository
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
