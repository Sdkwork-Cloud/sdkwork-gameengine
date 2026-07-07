use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_matchmaking_service::{
    CancelMatchTicketCommand, CreateMatchTicketCommand, GameMatchmakingError,
    GameMatchmakingRepository, GameMatchmakingResult, MatchTicketItem, MatchTicketPage,
    MatchTicketQuery, MatchmakingQueueQuery,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxGameMatchmakingRepository {
    pool: DatabasePool,
}

impl SqlxGameMatchmakingRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameMatchmakingRepository for SqlxGameMatchmakingRepository {
    async fn create_ticket(
        &self,
        tenant_id: &str,
        command: &CreateMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        if is_blank(Some(tenant_id)) {
            return Err(GameMatchmakingError::invalid("tenant_id is required"));
        }
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                create_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                create_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn get_ticket(
        &self,
        tenant_id: &str,
        ticket_id: &str,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        if is_blank(Some(tenant_id)) {
            return Err(GameMatchmakingError::invalid("tenant_id is required"));
        }
        if is_blank(Some(ticket_id)) {
            return Err(GameMatchmakingError::invalid("ticket_id is required"));
        }
        match &self.pool {
            DatabasePool::Postgres(pool, _) => get_postgres(pool, tenant_id, ticket_id).await,
            DatabasePool::Sqlite(pool, _) => get_sqlite(pool, tenant_id, ticket_id).await,
        }
    }

    async fn cancel_ticket(
        &self,
        tenant_id: &str,
        command: &CancelMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                cancel_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                cancel_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn list_tickets(
        &self,
        tenant_id: &str,
        query: &MatchTicketQuery,
    ) -> GameMatchmakingResult<MatchTicketPage> {
        let params = TicketListParams::from_query(tenant_id, query);
        match &self.pool {
            DatabasePool::Postgres(pool, _) => list_tickets_postgres(pool, params).await,
            DatabasePool::Sqlite(pool, _) => list_tickets_sqlite(pool, params).await,
        }
    }

    async fn list_queue(
        &self,
        tenant_id: &str,
        query: &MatchmakingQueueQuery,
    ) -> GameMatchmakingResult<MatchTicketPage> {
        let limit = query.limit() as i64;
        let offset = query.offset() as i64;
        let mode_id = query.mode_id.as_deref();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                list_queue_postgres(pool, tenant_id, &query.game_id, mode_id, limit, offset).await
            }
            DatabasePool::Sqlite(pool, _) => {
                list_queue_sqlite(pool, tenant_id, &query.game_id, mode_id, limit, offset).await
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct TicketRow {
    id: String,
    ticket_code: String,
    game_id: String,
    mode_id: Option<String>,
    ruleset_id: Option<String>,
    user_id: String,
    party_id: Option<String>,
    status: String,
    priority: i32,
    match_attributes: String,
    idempotency_key: String,
    queued_at: String,
    matched_at: Option<String>,
    cancelled_at: Option<String>,
    expires_at: Option<String>,
    version: i64,
}

impl TicketRow {
    fn into_item(self) -> GameMatchmakingResult<MatchTicketItem> {
        let match_attributes = serde_json::from_str(&self.match_attributes)
            .map_err(|error| GameMatchmakingError::invalid(error.to_string()))?;
        Ok(MatchTicketItem {
            id: self.id,
            ticket_code: self.ticket_code,
            game_id: self.game_id,
            mode_id: self.mode_id,
            ruleset_id: self.ruleset_id,
            user_id: self.user_id,
            party_id: self.party_id,
            status: self.status,
            priority: self.priority,
            match_attributes,
            idempotency_key: self.idempotency_key,
            queued_at: self.queued_at,
            matched_at: self.matched_at,
            cancelled_at: self.cancelled_at,
            expires_at: self.expires_at,
            version: self.version,
        })
    }
}

#[derive(Clone, Copy)]
struct TicketListParams<'a> {
    tenant_id: &'a str,
    game_id: Option<&'a str>,
    mode_id: Option<&'a str>,
    status: Option<&'a str>,
    user_id: Option<&'a str>,
    limit: i64,
    offset: i64,
}

impl<'a> TicketListParams<'a> {
    fn from_query(tenant_id: &'a str, query: &'a MatchTicketQuery) -> Self {
        Self {
            tenant_id,
            game_id: query.game_id.as_deref(),
            mode_id: query.mode_id.as_deref(),
            status: query.status.as_deref(),
            user_id: query.user_id.as_deref(),
            limit: query.limit() as i64,
            offset: query.offset() as i64,
        }
    }
}

const TICKET_COLUMNS_POSTGRES: &str = "id, ticket_code, game_id, mode_id, ruleset_id, user_id, \
party_id, status, priority, match_attributes::text AS match_attributes, idempotency_key, \
queued_at, matched_at, cancelled_at, expires_at, version";
const TICKET_COLUMNS_SQLITE: &str = "id, ticket_code, game_id, mode_id, ruleset_id, user_id, \
party_id, status, priority, match_attributes, idempotency_key, queued_at, matched_at, \
cancelled_at, expires_at, version";

async fn create_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CreateMatchTicketCommand,
    timestamp: &str,
) -> GameMatchmakingResult<MatchTicketItem> {
    if let Some(existing) = get_existing_postgres(pool, tenant_id, command).await? {
        return Ok(existing);
    }
    let id = uuid();
    let ticket_code = format!("MT-{id}");
    let attributes = command.match_attributes.to_string();
    let row = sqlx::query_as::<_, TicketRow>(&format!(
        "INSERT INTO game_match_ticket \
         (id, uuid, tenant_id, organization_id, ticket_code, game_id, mode_id, ruleset_id, \
          user_id, party_id, status, priority, match_attributes, idempotency_key, queued_at, \
          expires_at, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, $9, 'queued', $10, $11::jsonb, \
          $12, $13, $14, $13, $13) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING RETURNING {TICKET_COLUMNS_POSTGRES}",
    ))
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&ticket_code)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.ruleset_id)
    .bind(&command.user_id)
    .bind(&command.party_id)
    .bind(command.priority)
    .bind(attributes)
    .bind(&command.idempotency_key)
    .bind(timestamp)
    .bind(&command.expires_at)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;

    if let Some(row) = row {
        return row.into_item();
    }
    get_existing_postgres(pool, tenant_id, command)
        .await?
        .ok_or_else(|| GameMatchmakingError::conflict("match ticket idempotency conflict"))
}

async fn create_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &CreateMatchTicketCommand,
    timestamp: &str,
) -> GameMatchmakingResult<MatchTicketItem> {
    if let Some(existing) = get_existing_sqlite(pool, tenant_id, command).await? {
        return Ok(existing);
    }
    let id = uuid();
    let ticket_code = format!("MT-{id}");
    let attributes = command.match_attributes.to_string();
    let result = sqlx::query(
        "INSERT INTO game_match_ticket \
         (id, uuid, tenant_id, organization_id, ticket_code, game_id, mode_id, ruleset_id, \
          user_id, party_id, status, priority, match_attributes, idempotency_key, queued_at, \
          expires_at, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, 'queued', ?10, ?11, \
          ?12, ?13, ?14, ?13, ?13) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING",
    )
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&ticket_code)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.ruleset_id)
    .bind(&command.user_id)
    .bind(&command.party_id)
    .bind(command.priority)
    .bind(attributes)
    .bind(&command.idempotency_key)
    .bind(timestamp)
    .bind(&command.expires_at)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;
    if result.rows_affected() == 0 {
        return get_existing_sqlite(pool, tenant_id, command)
            .await?
            .ok_or_else(|| GameMatchmakingError::conflict("match ticket idempotency conflict"));
    }
    get_sqlite(pool, tenant_id, &id).await
}

async fn get_existing_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CreateMatchTicketCommand,
) -> GameMatchmakingResult<Option<MatchTicketItem>> {
    let row = sqlx::query_as::<_, TicketRow>(&format!(
        "SELECT {TICKET_COLUMNS_POSTGRES} FROM game_match_ticket \
         WHERE tenant_id = $1 AND idempotency_key = $2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    map_existing(row, command)
}

async fn get_existing_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &CreateMatchTicketCommand,
) -> GameMatchmakingResult<Option<MatchTicketItem>> {
    let row = sqlx::query_as::<_, TicketRow>(&format!(
        "SELECT {TICKET_COLUMNS_SQLITE} FROM game_match_ticket \
         WHERE tenant_id = ?1 AND idempotency_key = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    map_existing(row, command)
}

fn map_existing(
    row: Option<TicketRow>,
    command: &CreateMatchTicketCommand,
) -> GameMatchmakingResult<Option<MatchTicketItem>> {
    let Some(row) = row else {
        return Ok(None);
    };
    let item = row.into_item()?;
    ensure_idempotent_replay(&item, command)?;
    Ok(Some(item))
}

async fn get_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    ticket_id: &str,
) -> GameMatchmakingResult<MatchTicketItem> {
    let row = sqlx::query_as::<_, TicketRow>(&format!(
        "SELECT {TICKET_COLUMNS_POSTGRES} FROM game_match_ticket \
         WHERE tenant_id = $1 AND (id = $2 OR ticket_code = $2) LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(ticket_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameMatchmakingError::not_found("match ticket not found"))?;
    row.into_item()
}

async fn get_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    ticket_id: &str,
) -> GameMatchmakingResult<MatchTicketItem> {
    let row = sqlx::query_as::<_, TicketRow>(&format!(
        "SELECT {TICKET_COLUMNS_SQLITE} FROM game_match_ticket \
         WHERE tenant_id = ?1 AND (id = ?2 OR ticket_code = ?2) LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(ticket_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameMatchmakingError::not_found("match ticket not found"))?;
    row.into_item()
}

async fn cancel_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CancelMatchTicketCommand,
    timestamp: &str,
) -> GameMatchmakingResult<MatchTicketItem> {
    let row = sqlx::query_as::<_, TicketRow>(&format!(
        "UPDATE game_match_ticket SET status = 'cancelled', cancelled_at = $4, updated_at = $4, \
         version = version + 1 \
         WHERE tenant_id = $1 AND (id = $2 OR ticket_code = $2) AND user_id = $3 AND status = 'queued' \
         RETURNING {TICKET_COLUMNS_POSTGRES}",
    ))
    .bind(tenant_id)
    .bind(&command.ticket_id)
    .bind(&command.user_id)
    .bind(timestamp)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;
    match row {
        Some(row) => row.into_item(),
        None => cancel_failure(pool, tenant_id, &command.ticket_id).await,
    }
}

async fn cancel_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &CancelMatchTicketCommand,
    timestamp: &str,
) -> GameMatchmakingResult<MatchTicketItem> {
    let result = sqlx::query(
        "UPDATE game_match_ticket SET status = 'cancelled', cancelled_at = ?4, updated_at = ?4, \
         version = version + 1 \
         WHERE tenant_id = ?1 AND (id = ?2 OR ticket_code = ?2) AND user_id = ?3 AND status = 'queued'",
    )
    .bind(tenant_id)
    .bind(&command.ticket_id)
    .bind(&command.user_id)
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;
    if result.rows_affected() == 0 {
        let existing = get_sqlite(pool, tenant_id, &command.ticket_id).await?;
        if existing.status != "queued" {
            return Err(GameMatchmakingError::conflict(
                "only queued match tickets can be cancelled",
            ));
        }
        return Err(GameMatchmakingError::not_found("match ticket not found"));
    }
    get_sqlite(pool, tenant_id, &command.ticket_id).await
}

async fn cancel_failure(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    ticket_id: &str,
) -> GameMatchmakingResult<MatchTicketItem> {
    let existing = get_postgres(pool, tenant_id, ticket_id).await?;
    if existing.status != "queued" {
        return Err(GameMatchmakingError::conflict(
            "only queued match tickets can be cancelled",
        ));
    }
    Err(GameMatchmakingError::not_found("match ticket not found"))
}

async fn list_tickets_postgres(
    pool: &sqlx::PgPool,
    params: TicketListParams<'_>,
) -> GameMatchmakingResult<MatchTicketPage> {
    let rows = sqlx::query_as::<_, TicketRow>(&format!(
        "SELECT {TICKET_COLUMNS_POSTGRES} FROM game_match_ticket \
         WHERE tenant_id = $1 \
         AND ($2::text IS NULL OR game_id = $2) \
         AND ($3::text IS NULL OR mode_id = $3) \
         AND ($4::text IS NULL OR status = $4) \
         AND ($5::text IS NULL OR user_id = $5) \
         ORDER BY queued_at DESC LIMIT $6 OFFSET $7",
    ))
    .bind(params.tenant_id)
    .bind(params.game_id)
    .bind(params.mode_id)
    .bind(params.status)
    .bind(params.user_id)
    .bind(params.limit)
    .bind(params.offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_match_ticket \
         WHERE tenant_id = $1 \
         AND ($2::text IS NULL OR game_id = $2) \
         AND ($3::text IS NULL OR mode_id = $3) \
         AND ($4::text IS NULL OR status = $4) \
         AND ($5::text IS NULL OR user_id = $5)",
    )
    .bind(params.tenant_id)
    .bind(params.game_id)
    .bind(params.mode_id)
    .bind(params.status)
    .bind(params.user_id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    page_from_rows(rows, total, params.limit, params.offset)
}

async fn list_tickets_sqlite(
    pool: &sqlx::SqlitePool,
    params: TicketListParams<'_>,
) -> GameMatchmakingResult<MatchTicketPage> {
    let rows = sqlx::query_as::<_, TicketRow>(&format!(
        "SELECT {TICKET_COLUMNS_SQLITE} FROM game_match_ticket \
         WHERE tenant_id = ?1 \
         AND (?2 IS NULL OR game_id = ?2) \
         AND (?3 IS NULL OR mode_id = ?3) \
         AND (?4 IS NULL OR status = ?4) \
         AND (?5 IS NULL OR user_id = ?5) \
         ORDER BY queued_at DESC LIMIT ?6 OFFSET ?7",
    ))
    .bind(params.tenant_id)
    .bind(params.game_id)
    .bind(params.mode_id)
    .bind(params.status)
    .bind(params.user_id)
    .bind(params.limit)
    .bind(params.offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_match_ticket \
         WHERE tenant_id = ?1 \
         AND (?2 IS NULL OR game_id = ?2) \
         AND (?3 IS NULL OR mode_id = ?3) \
         AND (?4 IS NULL OR status = ?4) \
         AND (?5 IS NULL OR user_id = ?5)",
    )
    .bind(params.tenant_id)
    .bind(params.game_id)
    .bind(params.mode_id)
    .bind(params.status)
    .bind(params.user_id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    page_from_rows(rows, total, params.limit, params.offset)
}

async fn list_queue_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    game_id: &str,
    mode_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> GameMatchmakingResult<MatchTicketPage> {
    let rows = sqlx::query_as::<_, TicketRow>(&format!(
        "SELECT {TICKET_COLUMNS_POSTGRES} FROM game_match_ticket \
         WHERE tenant_id = $1 AND game_id = $2 AND status = 'queued' \
         AND ($3::text IS NULL OR mode_id = $3) \
         ORDER BY priority DESC, queued_at ASC LIMIT $4 OFFSET $5",
    ))
    .bind(tenant_id)
    .bind(game_id)
    .bind(mode_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_match_ticket \
         WHERE tenant_id = $1 AND game_id = $2 AND status = 'queued' \
         AND ($3::text IS NULL OR mode_id = $3)",
    )
    .bind(tenant_id)
    .bind(game_id)
    .bind(mode_id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    page_from_rows(rows, total, limit, offset)
}

async fn list_queue_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    game_id: &str,
    mode_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> GameMatchmakingResult<MatchTicketPage> {
    let rows = sqlx::query_as::<_, TicketRow>(&format!(
        "SELECT {TICKET_COLUMNS_SQLITE} FROM game_match_ticket \
         WHERE tenant_id = ?1 AND game_id = ?2 AND status = 'queued' \
         AND (?3 IS NULL OR mode_id = ?3) \
         ORDER BY priority DESC, queued_at ASC LIMIT ?4 OFFSET ?5",
    ))
    .bind(tenant_id)
    .bind(game_id)
    .bind(mode_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_match_ticket \
         WHERE tenant_id = ?1 AND game_id = ?2 AND status = 'queued' \
         AND (?3 IS NULL OR mode_id = ?3)",
    )
    .bind(tenant_id)
    .bind(game_id)
    .bind(mode_id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    page_from_rows(rows, total, limit, offset)
}

fn page_from_rows(
    rows: Vec<TicketRow>,
    total: i64,
    limit: i64,
    offset: i64,
) -> GameMatchmakingResult<MatchTicketPage> {
    let items = rows
        .into_iter()
        .map(TicketRow::into_item)
        .collect::<GameMatchmakingResult<Vec<_>>>()?;
    Ok(MatchTicketPage {
        items,
        total: total as u64,
        page: ((offset / limit) + 1) as u32,
        page_size: limit as u32,
    })
}

fn ensure_idempotent_replay(
    existing: &MatchTicketItem,
    command: &CreateMatchTicketCommand,
) -> GameMatchmakingResult<()> {
    let same_payload = existing.game_id == command.game_id
        && existing.mode_id == command.mode_id
        && existing.ruleset_id == command.ruleset_id
        && existing.user_id == command.user_id
        && existing.party_id == command.party_id
        && existing.priority == command.priority
        && existing.match_attributes == command.match_attributes
        && existing.expires_at == command.expires_at;
    if !same_payload {
        return Err(GameMatchmakingError::conflict(
            "idempotency_key already belongs to a different match ticket payload",
        ));
    }
    Ok(())
}

fn map_sqlx_error(error: sqlx::Error) -> GameMatchmakingError {
    GameMatchmakingError::invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
    use sdkwork_database_sqlx::create_pool_from_config;
    use serde_json::json;

    use super::*;

    async fn sqlite_repo() -> SqlxGameMatchmakingRepository {
        let pool = create_pool_from_config(DatabaseConfig {
            engine: DatabaseEngine::Sqlite,
            url: "sqlite::memory:".into(),
            max_connections: 1,
            ..Default::default()
        })
        .await
        .unwrap();
        pool.execute_raw(
            "CREATE TABLE game_match_ticket (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              ticket_code TEXT NOT NULL,
              game_id TEXT NOT NULL,
              mode_id TEXT,
              ruleset_id TEXT,
              user_id TEXT NOT NULL,
              party_id TEXT,
              status TEXT NOT NULL DEFAULT 'queued',
              priority INTEGER NOT NULL DEFAULT 0,
              match_attributes TEXT NOT NULL DEFAULT '{}',
              idempotency_key TEXT NOT NULL,
              queued_at TEXT NOT NULL,
              matched_at TEXT,
              cancelled_at TEXT,
              expires_at TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 0,
              UNIQUE (tenant_id, idempotency_key),
              UNIQUE (tenant_id, ticket_code)
            );",
        )
        .await
        .unwrap();
        SqlxGameMatchmakingRepository::new(pool)
    }

    fn command(idempotency_key: &str, user_id: &str, priority: i32) -> CreateMatchTicketCommand {
        CreateMatchTicketCommand {
            game_id: "game-xiangqi".into(),
            mode_id: Some("mode-ranked".into()),
            ruleset_id: Some("ruleset-standard".into()),
            user_id: user_id.into(),
            party_id: None,
            priority,
            match_attributes: json!({"rank": priority}),
            idempotency_key: idempotency_key.into(),
            expires_at: None,
        }
    }

    #[tokio::test]
    async fn sqlite_create_ticket_is_idempotent_and_conflicting_payload_fails() {
        let repository = sqlite_repo().await;
        let command = command("idem-sqlite-1", "user-1", 10);

        let first = repository.create_ticket("100001", &command).await.unwrap();
        let replay = repository.create_ticket("100001", &command).await.unwrap();
        assert_eq!(first.id, replay.id);

        let mut conflict = command.clone();
        conflict.user_id = "user-2".into();
        let error = repository
            .create_ticket("100001", &conflict)
            .await
            .unwrap_err();
        assert_eq!("conflict", error.code());
    }

    #[tokio::test]
    async fn sqlite_cancel_ticket_updates_status_and_version() {
        let repository = sqlite_repo().await;
        let ticket = repository
            .create_ticket("100001", &command("idem-cancel", "user-1", 10))
            .await
            .unwrap();

        let cancelled = repository
            .cancel_ticket(
                "100001",
                &CancelMatchTicketCommand {
                    ticket_id: ticket.id,
                    user_id: "user-1".into(),
                    reason: "player_cancelled".into(),
                },
            )
            .await
            .unwrap();

        assert_eq!("cancelled", cancelled.status);
        assert!(cancelled.cancelled_at.is_some());
        assert!(cancelled.version > ticket.version);
    }

    #[tokio::test]
    async fn sqlite_queue_list_is_paginated_and_sorted_by_priority() {
        let repository = sqlite_repo().await;
        repository
            .create_ticket("100001", &command("idem-low", "user-low", 10))
            .await
            .unwrap();
        repository
            .create_ticket("100001", &command("idem-high", "user-high", 30))
            .await
            .unwrap();
        repository
            .create_ticket("100001", &command("idem-mid", "user-mid", 20))
            .await
            .unwrap();

        let page = repository
            .list_queue(
                "100001",
                &MatchmakingQueueQuery {
                    game_id: "game-xiangqi".into(),
                    mode_id: Some("mode-ranked".into()),
                    page: Some(1),
                    page_size: Some(2),
                },
            )
            .await
            .unwrap();

        assert_eq!(3, page.total);
        assert_eq!(2, page.items.len());
        assert_eq!(30, page.items[0].priority);
        assert_eq!(20, page.items[1].priority);
    }
}
