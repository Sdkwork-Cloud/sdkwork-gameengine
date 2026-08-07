use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_session_service::{
    CreateGameSessionCommand, GameSessionError, GameSessionItem, GameSessionParticipantItem,
    GameSessionRepository, GameSessionResult, GameSessionResultItem, StartGameSessionCommand,
    SubmitSessionResultCommand,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxGameSessionRepository {
    pool: DatabasePool,
}

impl SqlxGameSessionRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameSessionRepository for SqlxGameSessionRepository {
    async fn create_session(
        &self,
        tenant_id: &str,
        command: &CreateGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem> {
        if is_blank(Some(tenant_id)) {
            return Err(GameSessionError::invalid("tenant_id is required"));
        }
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                create_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(_, _) => {
                unreachable!("game repository requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)")
            }
        }
    }

    async fn get_session(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<GameSessionItem> {
        if is_blank(Some(tenant_id)) {
            return Err(GameSessionError::invalid("tenant_id is required"));
        }
        if is_blank(Some(session_id)) {
            return Err(GameSessionError::invalid("session_id is required"));
        }
        match &self.pool {
            DatabasePool::Postgres(pool, _) => get_postgres(pool, tenant_id, session_id).await,
            DatabasePool::Sqlite(_, _) =>
                unreachable!("game repository requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"),
        }
    }

    async fn list_participants(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<Vec<GameSessionParticipantItem>> {
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                list_participants_postgres(pool, tenant_id, session_id).await
            }
            DatabasePool::Sqlite(_, _) => {
                unreachable!("game repository requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)")
            }
        }
    }

    async fn start_session(
        &self,
        tenant_id: &str,
        command: &StartGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem> {
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                start_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(_, _) => {
                unreachable!("game repository requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)")
            }
        }
    }

    async fn submit_result(
        &self,
        tenant_id: &str,
        command: &SubmitSessionResultCommand,
    ) -> GameSessionResult<GameSessionResultItem> {
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                submit_result_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(_, _) => {
                unreachable!("game repository requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)")
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    session_code: String,
    game_id: String,
    mode_id: Option<String>,
    ruleset_id: Option<String>,
    room_id: Option<String>,
    match_result_id: Option<String>,
    server_id: Option<String>,
    status: String,
    started_at: Option<String>,
    ended_at: Option<String>,
    completed_at: Option<String>,
    result_version: i32,
    metadata: String,
    version: i64,
}

impl SessionRow {
    fn into_item(self) -> GameSessionResult<GameSessionItem> {
        let metadata = parse_json(&self.metadata)?;
        Ok(GameSessionItem {
            id: self.id,
            session_code: self.session_code,
            game_id: self.game_id,
            mode_id: self.mode_id,
            ruleset_id: self.ruleset_id,
            room_id: self.room_id,
            match_result_id: self.match_result_id,
            server_id: self.server_id,
            status: self.status,
            started_at: self.started_at,
            ended_at: self.ended_at,
            completed_at: self.completed_at,
            result_version: self.result_version,
            metadata,
            version: self.version,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ParticipantRow {
    id: String,
    session_id: String,
    user_id: String,
    team_no: Option<i32>,
    display_name_snapshot: Option<String>,
    status: String,
    score_delta: i64,
    rank_no: Option<i32>,
    result_payload: String,
    version: i64,
}

impl ParticipantRow {
    fn into_item(self) -> GameSessionResult<GameSessionParticipantItem> {
        let result_payload = parse_json(&self.result_payload)?;
        Ok(GameSessionParticipantItem {
            id: self.id,
            session_id: self.session_id,
            user_id: self.user_id,
            team_no: self.team_no,
            display_name_snapshot: self.display_name_snapshot,
            status: self.status,
            score_delta: self.score_delta,
            rank_no: self.rank_no,
            result_payload,
            version: self.version,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ResultRow {
    id: String,
    session_id: String,
    source_type: String,
    source_id: Option<String>,
    idempotency_key: String,
    payload_hash: String,
    signature_status: String,
    validation_status: String,
    result_payload: String,
    received_at: String,
    validated_at: Option<String>,
    rejection_reason: Option<String>,
    version: i64,
}

impl ResultRow {
    fn into_item(self) -> GameSessionResult<GameSessionResultItem> {
        let result_payload = parse_json(&self.result_payload)?;
        Ok(GameSessionResultItem {
            id: self.id,
            session_id: self.session_id,
            source_type: self.source_type,
            source_id: self.source_id,
            idempotency_key: self.idempotency_key,
            payload_hash: self.payload_hash,
            signature_status: self.signature_status,
            validation_status: self.validation_status,
            result_payload,
            received_at: self.received_at,
            validated_at: self.validated_at,
            rejection_reason: self.rejection_reason,
            version: self.version,
        })
    }
}

const SESSION_COLUMNS_POSTGRES: &str = "id, session_code, game_id, mode_id, ruleset_id, room_id, \
match_result_id, server_id, status, started_at, ended_at, completed_at, result_version, \
metadata::text AS metadata, version";
const SESSION_COLUMNS_SQLITE: &str = "id, session_code, game_id, mode_id, ruleset_id, room_id, \
match_result_id, server_id, status, started_at, ended_at, completed_at, result_version, \
metadata, version";
const PARTICIPANT_COLUMNS_POSTGRES: &str = "id, session_id, user_id, team_no, \
display_name_snapshot, status, score_delta, rank_no, result_payload::text AS result_payload, version";
const PARTICIPANT_COLUMNS_SQLITE: &str = "id, session_id, user_id, team_no, \
display_name_snapshot, status, score_delta, rank_no, result_payload, version";
const RESULT_COLUMNS_POSTGRES: &str = "id, session_id, source_type, source_id, idempotency_key, \
payload_hash, signature_status, validation_status, result_payload::text AS result_payload, \
received_at, validated_at, rejection_reason, version";
const RESULT_COLUMNS_SQLITE: &str = "id, session_id, source_type, source_id, idempotency_key, \
payload_hash, signature_status, validation_status, result_payload, received_at, validated_at, \
rejection_reason, version";

async fn create_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CreateGameSessionCommand,
    timestamp: &str,
) -> GameSessionResult<GameSessionItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    let id = uuid();
    let metadata = command.metadata.to_string();
    let row = sqlx::query_as::<_, SessionRow>(sqlx::AssertSqlSafe(format!(
        "INSERT INTO game_session \
         (id, uuid, tenant_id, organization_id, session_code, game_id, mode_id, ruleset_id, room_id, \
          match_result_id, server_id, status, metadata, created_at, created_by, updated_at, updated_by) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, $9, $10, 'created', $11::jsonb, $12, $13, $12, $13) \
         RETURNING {SESSION_COLUMNS_POSTGRES}",
    )))
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.session_code)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.ruleset_id)
    .bind(&command.room_id)
    .bind(&command.match_result_id)
    .bind(&command.server_id)
    .bind(metadata)
    .bind(timestamp)
    .bind(&command.created_by)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    insert_participants_postgres(&mut tx, tenant_id, &id, command, timestamp).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    row.into_item()
}


async fn insert_participants_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    session_id: &str,
    command: &CreateGameSessionCommand,
    timestamp: &str,
) -> GameSessionResult<()> {
    for participant in &command.participants {
        sqlx::query(
            "INSERT INTO game_session_participant \
             (id, uuid, tenant_id, organization_id, session_id, user_id, team_no, \
              display_name_snapshot, status, result_payload, created_at, updated_at) \
             VALUES ($1, $2, $3, '0', $4, $5, $6, $7, 'joined', '{}'::jsonb, $8, $8)",
        )
        .bind(uuid())
        .bind(uuid())
        .bind(tenant_id)
        .bind(session_id)
        .bind(&participant.user_id)
        .bind(participant.team_no)
        .bind(&participant.display_name_snapshot)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
    }
    Ok(())
}


async fn get_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    session_id: &str,
) -> GameSessionResult<GameSessionItem> {
    let row = sqlx::query_as::<_, SessionRow>(sqlx::AssertSqlSafe(format!(
        "SELECT {SESSION_COLUMNS_POSTGRES} FROM game_session \
         WHERE tenant_id = $1 AND deleted_at IS NULL AND (id = $2 OR session_code = $2) LIMIT 1",
    )))
    .bind(tenant_id)
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameSessionError::not_found("session not found"))?;
    row.into_item()
}


async fn list_participants_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    session_id: &str,
) -> GameSessionResult<Vec<GameSessionParticipantItem>> {
    let rows = sqlx::query_as::<_, ParticipantRow>(sqlx::AssertSqlSafe(format!(
        "SELECT {PARTICIPANT_COLUMNS_POSTGRES} FROM game_session_participant \
         WHERE tenant_id = $1 AND session_id = $2 ORDER BY COALESCE(team_no, 0), user_id",
    )))
    .bind(tenant_id)
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    rows.into_iter().map(ParticipantRow::into_item).collect()
}


async fn start_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &StartGameSessionCommand,
    timestamp: &str,
) -> GameSessionResult<GameSessionItem> {
    let server_id = command.server_id.as_deref();
    let row = if let Some(expected_version) = command.expected_version {
        sqlx::query_as::<_, SessionRow>(sqlx::AssertSqlSafe(format!(
            "UPDATE game_session SET status = 'started', server_id = COALESCE($4, server_id), \
             started_at = $5, updated_at = $5, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $3 AND deleted_at IS NULL \
             RETURNING {SESSION_COLUMNS_POSTGRES}",
        )))
        .bind(tenant_id)
        .bind(&command.session_id)
        .bind(expected_version)
        .bind(server_id)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_as::<_, SessionRow>(sqlx::AssertSqlSafe(format!(
            "UPDATE game_session SET status = 'started', server_id = COALESCE($3, server_id), \
             started_at = $4, updated_at = $4, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING {SESSION_COLUMNS_POSTGRES}",
        )))
        .bind(tenant_id)
        .bind(&command.session_id)
        .bind(server_id)
        .bind(timestamp)
        .fetch_optional(pool)
        .await
    }
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameSessionError::conflict("session version has changed"))?;
    row.into_item()
}


async fn submit_result_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &SubmitSessionResultCommand,
    timestamp: &str,
) -> GameSessionResult<GameSessionResultItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    if let Some(existing) = get_existing_result_postgres(&mut tx, tenant_id, command).await? {
        tx.commit().await.map_err(map_sqlx_error)?;
        return Ok(existing);
    }
    ensure_session_exists_postgres(&mut tx, tenant_id, &command.session_id).await?;
    let id = uuid();
    let payload = command.result_payload.to_string();
    let row = sqlx::query_as::<_, ResultRow>(sqlx::AssertSqlSafe(format!(
        "INSERT INTO game_session_result \
         (id, uuid, tenant_id, organization_id, session_id, source_type, source_id, idempotency_key, \
          payload_hash, signature_status, validation_status, result_payload, received_at, validated_at, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, 'not_required', 'validated', $9::jsonb, $10, $10, $10, $10) \
         ON CONFLICT (tenant_id, session_id, idempotency_key) DO NOTHING RETURNING {RESULT_COLUMNS_POSTGRES}",
    )))
    .bind(&id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.session_id)
    .bind(&command.source_type)
    .bind(&command.source_id)
    .bind(&command.idempotency_key)
    .bind(&command.payload_hash)
    .bind(payload)
    .bind(timestamp)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    let Some(row) = row else {
        let existing = get_existing_result_postgres(&mut tx, tenant_id, command)
            .await?
            .ok_or_else(|| GameSessionError::conflict("session result idempotency conflict"))?;
        tx.commit().await.map_err(map_sqlx_error)?;
        return Ok(existing);
    };
    update_session_after_result_postgres(&mut tx, tenant_id, &command.session_id, timestamp)
        .await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    row.into_item()
}


async fn ensure_session_exists_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    session_id: &str,
) -> GameSessionResult<()> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM game_session WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(tenant_id)
    .bind(session_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    if exists.is_none() {
        return Err(GameSessionError::not_found("session not found"));
    }
    Ok(())
}


async fn get_existing_result_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    command: &SubmitSessionResultCommand,
) -> GameSessionResult<Option<GameSessionResultItem>> {
    let row = sqlx::query_as::<_, ResultRow>(sqlx::AssertSqlSafe(format!(
        "SELECT {RESULT_COLUMNS_POSTGRES} FROM game_session_result \
         WHERE tenant_id = $1 AND session_id = $2 AND idempotency_key = $3 LIMIT 1",
    )))
    .bind(tenant_id)
    .bind(&command.session_id)
    .bind(&command.idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    map_existing_result(row, command)
}


fn map_existing_result(
    row: Option<ResultRow>,
    command: &SubmitSessionResultCommand,
) -> GameSessionResult<Option<GameSessionResultItem>> {
    let Some(row) = row else {
        return Ok(None);
    };
    let item = row.into_item()?;
    ensure_idempotent_replay(&item, command)?;
    Ok(Some(item))
}

async fn update_session_after_result_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    session_id: &str,
    timestamp: &str,
) -> GameSessionResult<()> {
    sqlx::query(
        "UPDATE game_session SET result_version = result_version + 1, status = 'completed', \
         completed_at = COALESCE(completed_at, $3), updated_at = $3, version = version + 1 \
         WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(session_id)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}



fn ensure_idempotent_replay(
    existing: &GameSessionResultItem,
    command: &SubmitSessionResultCommand,
) -> GameSessionResult<()> {
    let same_payload = existing.source_type == command.source_type
        && existing.source_id == command.source_id
        && existing.payload_hash == command.payload_hash
        && existing.result_payload == command.result_payload;
    if !same_payload {
        return Err(GameSessionError::conflict(
            "idempotency_key already belongs to a different session result payload",
        ));
    }
    Ok(())
}

fn parse_json(value: &str) -> GameSessionResult<serde_json::Value> {
    serde_json::from_str(value).map_err(|error| GameSessionError::invalid(error.to_string()))
}

fn map_sqlx_error(error: sqlx::Error) -> GameSessionError {
    GameSessionError::invalid(error.to_string())
}

#[cfg(test)]
mod tests {

fn optional_postgres_database_url() -> Option<String> {
    std::env::var("SDKWORK_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|url| url.starts_with("postgres://") || url.starts_with("postgresql://"))
}
    use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
    use sdkwork_database_sqlx::create_pool_from_config;
    use sdkwork_game_session_service::CreateGameSessionParticipant;
    use serde_json::json;

    use super::*;

    async fn postgres_repo() -> Option<SqlxGameSessionRepository> {
    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!("skipping game repository test: set SDKWORK_DATABASE_URL or DATABASE_URL to a postgres URL");
        return None;
    };
                let pool = create_pool_from_config(DatabaseConfig {
            engine: DatabaseEngine::Postgres,
            url: database_url,
            max_connections: 1,
            ..Default::default()
        })
        .await
        .unwrap();
        pool.execute_raw(
            "CREATE TABLE game_session (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              session_code TEXT NOT NULL,
              game_id TEXT NOT NULL,
              mode_id TEXT,
              ruleset_id TEXT,
              room_id TEXT,
              match_result_id TEXT,
              server_id TEXT,
              status TEXT NOT NULL DEFAULT 'created',
              started_at TEXT,
              ended_at TEXT,
              completed_at TEXT,
              voided_at TEXT,
              result_version INTEGER NOT NULL DEFAULT 0,
              metadata TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              created_by TEXT,
              updated_at TEXT NOT NULL,
              updated_by TEXT,
              version INTEGER NOT NULL DEFAULT 0,
              deleted_at TEXT,
              deleted_by TEXT,
              UNIQUE (tenant_id, session_code)
            );
            CREATE TABLE game_session_participant (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              session_id TEXT NOT NULL,
              user_id TEXT NOT NULL,
              team_no INTEGER,
              display_name_snapshot TEXT,
              status TEXT NOT NULL DEFAULT 'joined',
              score_delta INTEGER NOT NULL DEFAULT 0,
              rank_no INTEGER,
              result_payload TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 0,
              UNIQUE (tenant_id, session_id, user_id)
            );
            CREATE TABLE game_session_result (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              session_id TEXT NOT NULL,
              source_type TEXT NOT NULL,
              source_id TEXT,
              idempotency_key TEXT NOT NULL,
              payload_hash TEXT NOT NULL,
              signature_status TEXT NOT NULL DEFAULT 'not_required',
              validation_status TEXT NOT NULL DEFAULT 'pending',
              result_payload TEXT NOT NULL DEFAULT '{}',
              received_at TEXT NOT NULL,
              validated_at TEXT,
              rejection_reason TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 0,
              UNIQUE (tenant_id, session_id, idempotency_key)
            );",
        )
        .await
        .unwrap();
        Some(SqlxGameSessionRepository::new(pool))
    }

    fn create_command() -> CreateGameSessionCommand {
        CreateGameSessionCommand {
            session_code: "S-1".into(),
            game_id: "game-xiangqi".into(),
            mode_id: Some("mode-ranked".into()),
            ruleset_id: Some("ruleset-standard".into()),
            room_id: Some("room-1".into()),
            match_result_id: None,
            server_id: Some("server-1".into()),
            created_by: Some("user-host".into()),
            metadata: json!({"map": "classic"}),
            participants: vec![
                CreateGameSessionParticipant {
                    user_id: "user-1".into(),
                    team_no: Some(1),
                    display_name_snapshot: Some("Player 1".into()),
                },
                CreateGameSessionParticipant {
                    user_id: "user-2".into(),
                    team_no: Some(2),
                    display_name_snapshot: Some("Player 2".into()),
                },
            ],
        }
    }

    fn result_command(session_id: &str, hash: &str) -> SubmitSessionResultCommand {
        SubmitSessionResultCommand {
            session_id: session_id.into(),
            source_type: "game_server".into(),
            source_id: Some("server-1".into()),
            idempotency_key: "idem-result-1".into(),
            payload_hash: hash.into(),
            result_payload: json!({"winner": "user-1"}),
        }
    }

    #[tokio::test]
    async fn sqlite_create_session_persists_participants() {
let Some(repository) = postgres_repo().await else {
    eprintln!("skipping game repository test: set SDKWORK_DATABASE_URL or DATABASE_URL to a postgres URL");
    return;
};
        let session = repository
            .create_session("100001", &create_command())
            .await
            .unwrap();
        let participants = repository
            .list_participants("100001", &session.id)
            .await
            .unwrap();

        assert_eq!(2, participants.len());
        assert_eq!("user-1", participants[0].user_id);
    }

    #[tokio::test]
    async fn sqlite_start_session_transitions_status() {
let Some(repository) = postgres_repo().await else {
    eprintln!("skipping game repository test: set SDKWORK_DATABASE_URL or DATABASE_URL to a postgres URL");
    return;
};
        let session = repository
            .create_session("100001", &create_command())
            .await
            .unwrap();

        let started = repository
            .start_session(
                "100001",
                &StartGameSessionCommand {
                    session_id: session.id,
                    server_id: Some("server-2".into()),
                    expected_version: Some(session.version),
                },
            )
            .await
            .unwrap();

        assert_eq!("started", started.status);
        assert_eq!(Some("server-2".into()), started.server_id);
        assert!(started.started_at.is_some());
    }

    #[tokio::test]
    async fn sqlite_submit_result_is_idempotent_and_updates_session_result_version() {
let Some(repository) = postgres_repo().await else {
    eprintln!("skipping game repository test: set SDKWORK_DATABASE_URL or DATABASE_URL to a postgres URL");
    return;
};
        let session = repository
            .create_session("100001", &create_command())
            .await
            .unwrap();

        let first = repository
            .submit_result("100001", &result_command(&session.id, "hash-1"))
            .await
            .unwrap();
        let replay = repository
            .submit_result("100001", &result_command(&session.id, "hash-1"))
            .await
            .unwrap();
        assert_eq!(first.id, replay.id);

        let updated = repository.get_session("100001", &session.id).await.unwrap();
        assert_eq!(1, updated.result_version);

        let error = repository
            .submit_result("100001", &result_command(&session.id, "hash-2"))
            .await
            .unwrap_err();
        assert_eq!("conflict", error.code());
    }
}
