use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_settlement_service::{
    CompleteSettlementJobCommand, CreateRewardIntentCommand, CreateSettlementJobCommand,
    GameRewardIntentItem, GameSettlementError, GameSettlementJobItem, GameSettlementJobPage,
    GameSettlementRepository, GameSettlementResult, RecordSettlementFailureCommand,
    SettlementDueJobQuery, StartSettlementJobCommand,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxGameSettlementRepository {
    pool: DatabasePool,
}

impl SqlxGameSettlementRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameSettlementRepository for SqlxGameSettlementRepository {
    async fn create_job(
        &self,
        tenant_id: &str,
        command: &CreateSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_tenant(tenant_id)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                create_job_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                create_job_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn get_job(
        &self,
        tenant_id: &str,
        job_id: &str,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_tenant(tenant_id)?;
        if is_blank(Some(job_id)) {
            return Err(GameSettlementError::invalid("job_id is required"));
        }
        match &self.pool {
            DatabasePool::Postgres(pool, _) => get_job_postgres(pool, tenant_id, job_id).await,
            DatabasePool::Sqlite(pool, _) => get_job_sqlite(pool, tenant_id, job_id).await,
        }
    }

    async fn list_due_jobs(
        &self,
        tenant_id: &str,
        query: &SettlementDueJobQuery,
    ) -> GameSettlementResult<GameSettlementJobPage> {
        validate_tenant(tenant_id)?;
        let limit = query.limit() as i64;
        let offset = query.offset() as i64;
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                list_due_jobs_postgres(pool, tenant_id, &query.due_at, limit, offset).await
            }
            DatabasePool::Sqlite(pool, _) => {
                list_due_jobs_sqlite(pool, tenant_id, &query.due_at, limit, offset).await
            }
        }
    }

    async fn start_job(
        &self,
        tenant_id: &str,
        command: &StartSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_tenant(tenant_id)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                start_job_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                start_job_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn record_failure(
        &self,
        tenant_id: &str,
        command: &RecordSettlementFailureCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_tenant(tenant_id)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                record_failure_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                record_failure_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn complete_job(
        &self,
        tenant_id: &str,
        command: &CompleteSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_tenant(tenant_id)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                complete_job_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                complete_job_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn create_reward_intent(
        &self,
        tenant_id: &str,
        command: &CreateRewardIntentCommand,
    ) -> GameSettlementResult<GameRewardIntentItem> {
        validate_tenant(tenant_id)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                create_reward_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                create_reward_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct JobRow {
    id: String,
    session_id: String,
    session_result_id: String,
    status: String,
    attempt_count: i32,
    idempotency_key: String,
    error_code: Option<String>,
    error_detail: Option<String>,
    job_payload: String,
    created_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    next_retry_at: Option<String>,
    version: i64,
}

impl JobRow {
    fn into_item(self) -> GameSettlementResult<GameSettlementJobItem> {
        Ok(GameSettlementJobItem {
            id: self.id,
            session_id: self.session_id,
            session_result_id: self.session_result_id,
            status: self.status,
            attempt_count: self.attempt_count,
            idempotency_key: self.idempotency_key,
            error_code: self.error_code,
            error_detail: self.error_detail,
            job_payload: parse_json(&self.job_payload)?,
            created_at: self.created_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            next_retry_at: self.next_retry_at,
            version: self.version,
        })
    }
}

#[derive(sqlx::FromRow)]
struct RewardRow {
    id: String,
    settlement_job_id: String,
    user_id: String,
    reward_type: String,
    external_owner: String,
    external_reference_id: Option<String>,
    intent_payload: String,
    status: String,
    idempotency_key: String,
    created_at: String,
    submitted_at: Option<String>,
    completed_at: Option<String>,
    version: i64,
}

impl RewardRow {
    fn into_item(self) -> GameSettlementResult<GameRewardIntentItem> {
        Ok(GameRewardIntentItem {
            id: self.id,
            settlement_job_id: self.settlement_job_id,
            user_id: self.user_id,
            reward_type: self.reward_type,
            external_owner: self.external_owner,
            external_reference_id: self.external_reference_id,
            intent_payload: parse_json(&self.intent_payload)?,
            status: self.status,
            idempotency_key: self.idempotency_key,
            created_at: self.created_at,
            submitted_at: self.submitted_at,
            completed_at: self.completed_at,
            version: self.version,
        })
    }
}

const JOB_COLUMNS_POSTGRES: &str = "id, session_id, session_result_id, status, attempt_count, \
idempotency_key, error_code, error_detail, job_payload::text AS job_payload, created_at, \
started_at, completed_at, next_retry_at, version";
const JOB_COLUMNS_SQLITE: &str = "id, session_id, session_result_id, status, attempt_count, \
idempotency_key, error_code, error_detail, job_payload, created_at, started_at, completed_at, \
next_retry_at, version";
const REWARD_COLUMNS_POSTGRES: &str = "id, settlement_job_id, user_id, reward_type, \
external_owner, external_reference_id, intent_payload::text AS intent_payload, status, \
idempotency_key, created_at, submitted_at, completed_at, version";
const REWARD_COLUMNS_SQLITE: &str = "id, settlement_job_id, user_id, reward_type, \
external_owner, external_reference_id, intent_payload, status, idempotency_key, created_at, \
submitted_at, completed_at, version";

async fn create_job_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CreateSettlementJobCommand,
    timestamp: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    if let Some(existing) = get_existing_job_postgres(pool, tenant_id, command).await? {
        return Ok(existing);
    }
    let id = uuid();
    let payload = command.job_payload.to_string();
    let row = sqlx::query_as::<_, JobRow>(&format!(
        "INSERT INTO game_settlement_job \
         (id, uuid, tenant_id, organization_id, session_id, session_result_id, idempotency_key, \
          job_payload, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7::jsonb, $8, $8) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING RETURNING {JOB_COLUMNS_POSTGRES}",
    ))
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.session_id)
    .bind(&command.session_result_id)
    .bind(&command.idempotency_key)
    .bind(payload)
    .bind(timestamp)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    if let Some(row) = row {
        return row.into_item();
    }
    get_existing_job_postgres(pool, tenant_id, command)
        .await?
        .ok_or_else(|| GameSettlementError::conflict("settlement job idempotency conflict"))
}

async fn create_job_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &CreateSettlementJobCommand,
    timestamp: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    if let Some(existing) = get_existing_job_sqlite(pool, tenant_id, command).await? {
        return Ok(existing);
    }
    let id = uuid();
    let result = sqlx::query(
        "INSERT INTO game_settlement_job \
         (id, uuid, tenant_id, organization_id, session_id, session_result_id, idempotency_key, \
          job_payload, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?8) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING",
    )
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.session_id)
    .bind(&command.session_result_id)
    .bind(&command.idempotency_key)
    .bind(command.job_payload.to_string())
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;
    if result.rows_affected() == 0 {
        return get_existing_job_sqlite(pool, tenant_id, command)
            .await?
            .ok_or_else(|| GameSettlementError::conflict("settlement job idempotency conflict"));
    }
    get_job_sqlite(pool, tenant_id, &id).await
}

async fn get_existing_job_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CreateSettlementJobCommand,
) -> GameSettlementResult<Option<GameSettlementJobItem>> {
    let row = sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS_POSTGRES} FROM game_settlement_job \
         WHERE tenant_id = $1 AND idempotency_key = $2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    map_existing_job(row, command)
}

async fn get_existing_job_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &CreateSettlementJobCommand,
) -> GameSettlementResult<Option<GameSettlementJobItem>> {
    let row = sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS_SQLITE} FROM game_settlement_job \
         WHERE tenant_id = ?1 AND idempotency_key = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    map_existing_job(row, command)
}

fn map_existing_job(
    row: Option<JobRow>,
    command: &CreateSettlementJobCommand,
) -> GameSettlementResult<Option<GameSettlementJobItem>> {
    let Some(row) = row else {
        return Ok(None);
    };
    let item = row.into_item()?;
    ensure_job_idempotent(&item, command)?;
    Ok(Some(item))
}

async fn get_job_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    job_id: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    let row = sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS_POSTGRES} FROM game_settlement_job \
         WHERE tenant_id = $1 AND id = $2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameSettlementError::not_found("settlement job not found"))?;
    row.into_item()
}

async fn get_job_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    job_id: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    let row = sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS_SQLITE} FROM game_settlement_job \
         WHERE tenant_id = ?1 AND id = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameSettlementError::not_found("settlement job not found"))?;
    row.into_item()
}

async fn list_due_jobs_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    due_at: &str,
    limit: i64,
    offset: i64,
) -> GameSettlementResult<GameSettlementJobPage> {
    let filter = "tenant_id = $1 AND (status = 'pending' OR \
        (status = 'retrying' AND next_retry_at IS NOT NULL AND next_retry_at::timestamptz <= $2::timestamptz))";
    let rows = sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS_POSTGRES} FROM game_settlement_job \
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
        "SELECT COUNT(*) FROM game_settlement_job WHERE {filter}",
    ))
    .bind(tenant_id)
    .bind(due_at)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    page_from_rows(rows, total, limit, offset)
}

async fn list_due_jobs_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    due_at: &str,
    limit: i64,
    offset: i64,
) -> GameSettlementResult<GameSettlementJobPage> {
    let filter = "tenant_id = ?1 AND (status = 'pending' OR \
        (status = 'retrying' AND next_retry_at IS NOT NULL AND next_retry_at <= ?2))";
    let rows = sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS_SQLITE} FROM game_settlement_job \
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
        "SELECT COUNT(*) FROM game_settlement_job WHERE {filter}",
    ))
    .bind(tenant_id)
    .bind(due_at)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    page_from_rows(rows, total, limit, offset)
}

async fn start_job_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &StartSettlementJobCommand,
    timestamp: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    let row = if let Some(expected_version) = command.expected_version {
        sqlx::query_as::<_, JobRow>(&format!(
            "UPDATE game_settlement_job SET status = 'running', attempt_count = attempt_count + 1, \
             started_at = $4, next_retry_at = NULL, updated_at = $4, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $3 AND status IN ('pending', 'retrying') \
             RETURNING {JOB_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(expected_version)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_as::<_, JobRow>(&format!(
            "UPDATE game_settlement_job SET status = 'running', attempt_count = attempt_count + 1, \
             started_at = $3, next_retry_at = NULL, updated_at = $3, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND status IN ('pending', 'retrying') \
             RETURNING {JOB_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    }
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameSettlementError::conflict("settlement job version has changed"))?;
    row.into_item()
}

async fn start_job_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &StartSettlementJobCommand,
    timestamp: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    let result = if let Some(expected_version) = command.expected_version {
        sqlx::query(
            "UPDATE game_settlement_job SET status = 'running', attempt_count = attempt_count + 1, \
             started_at = ?4, next_retry_at = NULL, updated_at = ?4, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND version = ?3 AND status IN ('pending', 'retrying')",
        )
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(expected_version)
        .bind(timestamp)
        .execute(pool)
        .await
    } else {
        sqlx::query(
            "UPDATE game_settlement_job SET status = 'running', attempt_count = attempt_count + 1, \
             started_at = ?3, next_retry_at = NULL, updated_at = ?3, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND status IN ('pending', 'retrying')",
        )
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(timestamp)
        .execute(pool)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_rows_affected(result.rows_affected(), "settlement job version has changed")?;
    get_job_sqlite(pool, tenant_id, &command.job_id).await
}

async fn record_failure_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &RecordSettlementFailureCommand,
    timestamp: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    let status = failure_status(command);
    let row = if let Some(expected_version) = command.expected_version {
        sqlx::query_as::<_, JobRow>(&format!(
            "UPDATE game_settlement_job SET status = $4, error_code = $5, error_detail = $6, \
             next_retry_at = $7, updated_at = $8, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $3 RETURNING {JOB_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(expected_version)
        .bind(status)
        .bind(&command.error_code)
        .bind(&command.error_detail)
        .bind(&command.next_retry_at)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_as::<_, JobRow>(&format!(
            "UPDATE game_settlement_job SET status = $3, error_code = $4, error_detail = $5, \
             next_retry_at = $6, updated_at = $7, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 RETURNING {JOB_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(status)
        .bind(&command.error_code)
        .bind(&command.error_detail)
        .bind(&command.next_retry_at)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    }
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameSettlementError::conflict("settlement job version has changed"))?;
    row.into_item()
}

async fn record_failure_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &RecordSettlementFailureCommand,
    timestamp: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    let status = failure_status(command);
    let result = if let Some(expected_version) = command.expected_version {
        sqlx::query(
            "UPDATE game_settlement_job SET status = ?4, error_code = ?5, error_detail = ?6, \
             next_retry_at = ?7, updated_at = ?8, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND version = ?3",
        )
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(expected_version)
        .bind(status)
        .bind(&command.error_code)
        .bind(&command.error_detail)
        .bind(&command.next_retry_at)
        .bind(timestamp)
        .execute(pool)
        .await
    } else {
        sqlx::query(
            "UPDATE game_settlement_job SET status = ?3, error_code = ?4, error_detail = ?5, \
             next_retry_at = ?6, updated_at = ?7, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2",
        )
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(status)
        .bind(&command.error_code)
        .bind(&command.error_detail)
        .bind(&command.next_retry_at)
        .bind(timestamp)
        .execute(pool)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_rows_affected(result.rows_affected(), "settlement job version has changed")?;
    get_job_sqlite(pool, tenant_id, &command.job_id).await
}

async fn complete_job_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CompleteSettlementJobCommand,
    timestamp: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    let row = if let Some(expected_version) = command.expected_version {
        sqlx::query_as::<_, JobRow>(&format!(
            "UPDATE game_settlement_job SET status = 'succeeded', completed_at = $4, \
             next_retry_at = NULL, updated_at = $4, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $3 RETURNING {JOB_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(expected_version)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_as::<_, JobRow>(&format!(
            "UPDATE game_settlement_job SET status = 'succeeded', completed_at = $3, \
             next_retry_at = NULL, updated_at = $3, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 RETURNING {JOB_COLUMNS_POSTGRES}",
        ))
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    }
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameSettlementError::conflict("settlement job version has changed"))?;
    row.into_item()
}

async fn complete_job_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &CompleteSettlementJobCommand,
    timestamp: &str,
) -> GameSettlementResult<GameSettlementJobItem> {
    let result = if let Some(expected_version) = command.expected_version {
        sqlx::query(
            "UPDATE game_settlement_job SET status = 'succeeded', completed_at = ?4, \
             next_retry_at = NULL, updated_at = ?4, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND version = ?3",
        )
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(expected_version)
        .bind(timestamp)
        .execute(pool)
        .await
    } else {
        sqlx::query(
            "UPDATE game_settlement_job SET status = 'succeeded', completed_at = ?3, \
             next_retry_at = NULL, updated_at = ?3, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2",
        )
        .bind(tenant_id)
        .bind(&command.job_id)
        .bind(timestamp)
        .execute(pool)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_rows_affected(result.rows_affected(), "settlement job version has changed")?;
    get_job_sqlite(pool, tenant_id, &command.job_id).await
}

async fn create_reward_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CreateRewardIntentCommand,
    timestamp: &str,
) -> GameSettlementResult<GameRewardIntentItem> {
    ensure_job_exists_postgres(pool, tenant_id, &command.settlement_job_id).await?;
    if let Some(existing) = get_existing_reward_postgres(pool, tenant_id, command).await? {
        return Ok(existing);
    }
    let id = uuid();
    let payload = command.intent_payload.to_string();
    let row = sqlx::query_as::<_, RewardRow>(&format!(
        "INSERT INTO game_reward_intent \
         (id, uuid, tenant_id, organization_id, settlement_job_id, user_id, reward_type, \
          external_owner, intent_payload, idempotency_key, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8::jsonb, $9, $10, $10) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING RETURNING {REWARD_COLUMNS_POSTGRES}",
    ))
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.settlement_job_id)
    .bind(&command.user_id)
    .bind(&command.reward_type)
    .bind(&command.external_owner)
    .bind(payload)
    .bind(&command.idempotency_key)
    .bind(timestamp)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    if let Some(row) = row {
        return row.into_item();
    }
    get_existing_reward_postgres(pool, tenant_id, command)
        .await?
        .ok_or_else(|| GameSettlementError::conflict("reward intent idempotency conflict"))
}

async fn create_reward_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &CreateRewardIntentCommand,
    timestamp: &str,
) -> GameSettlementResult<GameRewardIntentItem> {
    ensure_job_exists_sqlite(pool, tenant_id, &command.settlement_job_id).await?;
    if let Some(existing) = get_existing_reward_sqlite(pool, tenant_id, command).await? {
        return Ok(existing);
    }
    let id = uuid();
    let result = sqlx::query(
        "INSERT INTO game_reward_intent \
         (id, uuid, tenant_id, organization_id, settlement_job_id, user_id, reward_type, \
          external_owner, intent_payload, idempotency_key, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING",
    )
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.settlement_job_id)
    .bind(&command.user_id)
    .bind(&command.reward_type)
    .bind(&command.external_owner)
    .bind(command.intent_payload.to_string())
    .bind(&command.idempotency_key)
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;
    if result.rows_affected() == 0 {
        return get_existing_reward_sqlite(pool, tenant_id, command)
            .await?
            .ok_or_else(|| GameSettlementError::conflict("reward intent idempotency conflict"));
    }
    get_reward_sqlite(pool, tenant_id, &id).await
}

async fn ensure_job_exists_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    job_id: &str,
) -> GameSettlementResult<()> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM game_settlement_job WHERE tenant_id = $1 AND id = $2 LIMIT 1",
    )
    .bind(tenant_id)
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    if exists.is_none() {
        return Err(GameSettlementError::not_found("settlement job not found"));
    }
    Ok(())
}

async fn ensure_job_exists_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    job_id: &str,
) -> GameSettlementResult<()> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM game_settlement_job WHERE tenant_id = ?1 AND id = ?2 LIMIT 1",
    )
    .bind(tenant_id)
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    if exists.is_none() {
        return Err(GameSettlementError::not_found("settlement job not found"));
    }
    Ok(())
}

async fn get_existing_reward_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CreateRewardIntentCommand,
) -> GameSettlementResult<Option<GameRewardIntentItem>> {
    let row = sqlx::query_as::<_, RewardRow>(&format!(
        "SELECT {REWARD_COLUMNS_POSTGRES} FROM game_reward_intent \
         WHERE tenant_id = $1 AND idempotency_key = $2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    map_existing_reward(row, command)
}

async fn get_existing_reward_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &CreateRewardIntentCommand,
) -> GameSettlementResult<Option<GameRewardIntentItem>> {
    let row = sqlx::query_as::<_, RewardRow>(&format!(
        "SELECT {REWARD_COLUMNS_SQLITE} FROM game_reward_intent \
         WHERE tenant_id = ?1 AND idempotency_key = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    map_existing_reward(row, command)
}

async fn get_reward_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    reward_id: &str,
) -> GameSettlementResult<GameRewardIntentItem> {
    let row = sqlx::query_as::<_, RewardRow>(&format!(
        "SELECT {REWARD_COLUMNS_SQLITE} FROM game_reward_intent \
         WHERE tenant_id = ?1 AND id = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(reward_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameSettlementError::not_found("reward intent not found"))?;
    row.into_item()
}

fn map_existing_reward(
    row: Option<RewardRow>,
    command: &CreateRewardIntentCommand,
) -> GameSettlementResult<Option<GameRewardIntentItem>> {
    let Some(row) = row else {
        return Ok(None);
    };
    let item = row.into_item()?;
    ensure_reward_idempotent(&item, command)?;
    Ok(Some(item))
}

fn page_from_rows(
    rows: Vec<JobRow>,
    total: i64,
    limit: i64,
    offset: i64,
) -> GameSettlementResult<GameSettlementJobPage> {
    let items = rows
        .into_iter()
        .map(JobRow::into_item)
        .collect::<GameSettlementResult<Vec<_>>>()?;
    Ok(GameSettlementJobPage {
        items,
        total: total as u64,
        page: ((offset / limit) + 1) as u32,
        page_size: limit as u32,
    })
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

fn ensure_rows_affected(rows_affected: u64, message: &str) -> GameSettlementResult<()> {
    if rows_affected == 0 {
        return Err(GameSettlementError::conflict(message));
    }
    Ok(())
}

fn failure_status(command: &RecordSettlementFailureCommand) -> &'static str {
    if command.next_retry_at.is_some() {
        "retrying"
    } else {
        "failed"
    }
}

fn parse_json(value: &str) -> GameSettlementResult<serde_json::Value> {
    serde_json::from_str(value).map_err(|error| GameSettlementError::invalid(error.to_string()))
}

fn validate_tenant(tenant_id: &str) -> GameSettlementResult<()> {
    if is_blank(Some(tenant_id)) {
        return Err(GameSettlementError::invalid("tenant_id is required"));
    }
    Ok(())
}

fn map_sqlx_error(error: sqlx::Error) -> GameSettlementError {
    GameSettlementError::invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
    use sdkwork_database_sqlx::create_pool_from_config;
    use serde_json::json;

    use super::*;

    async fn sqlite_repo() -> SqlxGameSettlementRepository {
        let pool = create_pool_from_config(DatabaseConfig {
            engine: DatabaseEngine::Sqlite,
            url: "sqlite::memory:".into(),
            max_connections: 1,
            ..Default::default()
        })
        .await
        .unwrap();
        pool.execute_raw(
            "CREATE TABLE game_settlement_job (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              session_id TEXT NOT NULL,
              session_result_id TEXT NOT NULL,
              status TEXT NOT NULL DEFAULT 'pending',
              attempt_count INTEGER NOT NULL DEFAULT 0,
              idempotency_key TEXT NOT NULL,
              error_code TEXT,
              error_detail TEXT,
              job_payload TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              started_at TEXT,
              completed_at TEXT,
              next_retry_at TEXT,
              updated_at TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 0,
              UNIQUE (tenant_id, idempotency_key)
            );
            CREATE TABLE game_reward_intent (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              settlement_job_id TEXT NOT NULL,
              user_id TEXT NOT NULL,
              reward_type TEXT NOT NULL,
              external_owner TEXT NOT NULL,
              external_reference_id TEXT,
              intent_payload TEXT NOT NULL DEFAULT '{}',
              status TEXT NOT NULL DEFAULT 'pending',
              idempotency_key TEXT NOT NULL,
              created_at TEXT NOT NULL,
              submitted_at TEXT,
              completed_at TEXT,
              updated_at TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 0,
              UNIQUE (tenant_id, idempotency_key)
            );",
        )
        .await
        .unwrap();
        SqlxGameSettlementRepository::new(pool)
    }

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
    async fn sqlite_create_job_is_idempotent_and_conflict_checked() {
        let repository = sqlite_repo().await;
        let command = job_command("idem-job-sqlite");

        let first = repository.create_job("100001", &command).await.unwrap();
        let replay = repository.create_job("100001", &command).await.unwrap();
        assert_eq!(first.id, replay.id);

        let mut conflict = command;
        conflict.session_result_id = "result-2".into();
        let error = repository
            .create_job("100001", &conflict)
            .await
            .unwrap_err();
        assert_eq!("conflict", error.code());
    }

    #[tokio::test]
    async fn sqlite_failed_job_retry_is_listed_when_due() {
        let repository = sqlite_repo().await;
        let job = repository
            .create_job("100001", &job_command("idem-retry-sqlite"))
            .await
            .unwrap();
        let running = repository
            .start_job(
                "100001",
                &StartSettlementJobCommand {
                    job_id: job.id,
                    expected_version: Some(job.version),
                },
            )
            .await
            .unwrap();
        let retrying = repository
            .record_failure(
                "100001",
                &RecordSettlementFailureCommand {
                    job_id: running.id,
                    error_code: "downstream_timeout".into(),
                    error_detail: Some("wallet dependency timeout".into()),
                    next_retry_at: Some("2026-07-07T00:05:00Z".into()),
                    expected_version: Some(running.version),
                },
            )
            .await
            .unwrap();

        assert_eq!("retrying", retrying.status);
        assert_eq!(1, retrying.attempt_count);

        let page = repository
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
        assert_eq!(retrying.id, page.items[0].id);
    }

    #[tokio::test]
    async fn sqlite_reward_intent_is_idempotent_and_conflict_checked() {
        let repository = sqlite_repo().await;
        let job = repository
            .create_job("100001", &job_command("idem-reward-sqlite"))
            .await
            .unwrap();
        let command = reward_command(&job.id, "idem-reward-sqlite-1");

        let first = repository
            .create_reward_intent("100001", &command)
            .await
            .unwrap();
        let replay = repository
            .create_reward_intent("100001", &command)
            .await
            .unwrap();
        assert_eq!(first.id, replay.id);

        let mut conflict = command;
        conflict.user_id = "user-2".into();
        let error = repository
            .create_reward_intent("100001", &conflict)
            .await
            .unwrap_err();
        assert_eq!("conflict", error.code());
    }
}
