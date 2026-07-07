use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_points_service::{
    AppendPointLedgerCommand, GamePointBalance, GamePointError, GamePointLedgerEntry,
    GamePointRepository, GamePointResult,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxGamePointRepository {
    pool: DatabasePool,
}

impl SqlxGamePointRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GamePointRepository for SqlxGamePointRepository {
    async fn append_ledger(
        &self,
        tenant_id: &str,
        command: &AppendPointLedgerCommand,
    ) -> GamePointResult<GamePointLedgerEntry> {
        if is_blank(Some(tenant_id)) {
            return Err(GamePointError::invalid("tenant_id is required"));
        }

        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                append_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                append_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn get_balance(
        &self,
        tenant_id: &str,
        ledger_account_id: &str,
    ) -> GamePointResult<GamePointBalance> {
        if is_blank(Some(tenant_id)) {
            return Err(GamePointError::invalid("tenant_id is required"));
        }
        if is_blank(Some(ledger_account_id)) {
            return Err(GamePointError::invalid("ledger_account_id is required"));
        }

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                get_balance_postgres(pool, tenant_id, ledger_account_id).await
            }
            DatabasePool::Sqlite(pool, _) => {
                get_balance_sqlite(pool, tenant_id, ledger_account_id).await
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct LedgerRow {
    id: String,
    ledger_account_id: String,
    game_id: String,
    mode_id: Option<String>,
    season_id: Option<String>,
    user_id: String,
    direction: String,
    points_delta: i64,
    points_after: Option<i64>,
    source_event_id: String,
    reason_code: String,
    idempotency_key: String,
    created_at: String,
    version: i64,
}

impl LedgerRow {
    fn into_item(self) -> GamePointLedgerEntry {
        GamePointLedgerEntry {
            id: self.id,
            ledger_account_id: self.ledger_account_id,
            game_id: self.game_id,
            mode_id: self.mode_id,
            season_id: self.season_id,
            user_id: self.user_id,
            direction: self.direction,
            points_delta: self.points_delta,
            points_after: self.points_after.unwrap_or_default(),
            source_event_id: self.source_event_id,
            reason_code: self.reason_code,
            idempotency_key: self.idempotency_key,
            created_at: self.created_at,
            version: self.version,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BalanceRow {
    id: String,
    ledger_account_id: String,
    game_id: String,
    mode_id: Option<String>,
    season_id: Option<String>,
    user_id: String,
    points: i64,
    last_ledger_id: Option<String>,
    updated_at: String,
    version: i64,
}

impl BalanceRow {
    fn into_item(self) -> GamePointBalance {
        GamePointBalance {
            id: self.id,
            ledger_account_id: self.ledger_account_id,
            game_id: self.game_id,
            mode_id: self.mode_id,
            season_id: self.season_id,
            user_id: self.user_id,
            points: self.points,
            last_ledger_id: self.last_ledger_id,
            updated_at: self.updated_at,
            version: self.version,
        }
    }
}

const LEDGER_COLUMNS: &str = "id, ledger_account_id, game_id, mode_id, season_id, user_id, \
direction, points_delta, points_after, source_event_id, reason_code, idempotency_key, created_at, version";
const BALANCE_COLUMNS: &str = "id, ledger_account_id, game_id, mode_id, season_id, user_id, \
points, last_ledger_id, updated_at, version";

async fn append_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &AppendPointLedgerCommand,
    timestamp: &str,
) -> GamePointResult<GamePointLedgerEntry> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    if let Some(existing) = get_existing_ledger_postgres(&mut tx, tenant_id, command).await? {
        tx.commit().await.map_err(map_sqlx_error)?;
        return Ok(existing);
    }

    let ledger_id = uuid();
    let inserted = sqlx::query_as::<_, LedgerRow>(&format!(
        "INSERT INTO game_point_ledger \
         (id, uuid, tenant_id, organization_id, ledger_account_id, game_id, mode_id, season_id, \
          user_id, direction, points_delta, source_event_id, reason_code, idempotency_key, \
          created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING RETURNING {LEDGER_COLUMNS}",
    ))
    .bind(&ledger_id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.ledger_account_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.season_id)
    .bind(&command.user_id)
    .bind(&command.direction)
    .bind(command.points_delta)
    .bind(&command.source_event_id)
    .bind(&command.reason_code)
    .bind(&command.idempotency_key)
    .bind(timestamp)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;

    if inserted.is_none() {
        let existing = get_existing_ledger_postgres(&mut tx, tenant_id, command)
            .await?
            .ok_or_else(|| GamePointError::conflict("point ledger idempotency conflict"))?;
        tx.commit().await.map_err(map_sqlx_error)?;
        return Ok(existing);
    }

    let points_after =
        upsert_balance_postgres(&mut tx, tenant_id, command, &ledger_id, timestamp).await?;
    let row = sqlx::query_as::<_, LedgerRow>(&format!(
        "UPDATE game_point_ledger SET points_after = $3, updated_at = $4, version = version + 1 \
         WHERE tenant_id = $1 AND id = $2 RETURNING {LEDGER_COLUMNS}",
    ))
    .bind(tenant_id)
    .bind(&ledger_id)
    .bind(points_after)
    .bind(timestamp)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;

    tx.commit().await.map_err(map_sqlx_error)?;
    Ok(row.into_item())
}

async fn append_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &AppendPointLedgerCommand,
    timestamp: &str,
) -> GamePointResult<GamePointLedgerEntry> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    if let Some(existing) = get_existing_ledger_sqlite(&mut tx, tenant_id, command).await? {
        tx.commit().await.map_err(map_sqlx_error)?;
        return Ok(existing);
    }

    let ledger_id = uuid();
    let result = sqlx::query(
        "INSERT INTO game_point_ledger \
         (id, uuid, tenant_id, organization_id, ledger_account_id, game_id, mode_id, season_id, \
          user_id, direction, points_delta, source_event_id, reason_code, idempotency_key, \
          created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING",
    )
    .bind(&ledger_id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.ledger_account_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.season_id)
    .bind(&command.user_id)
    .bind(&command.direction)
    .bind(command.points_delta)
    .bind(&command.source_event_id)
    .bind(&command.reason_code)
    .bind(&command.idempotency_key)
    .bind(timestamp)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;

    if result.rows_affected() == 0 {
        let existing = get_existing_ledger_sqlite(&mut tx, tenant_id, command)
            .await?
            .ok_or_else(|| GamePointError::conflict("point ledger idempotency conflict"))?;
        tx.commit().await.map_err(map_sqlx_error)?;
        return Ok(existing);
    }

    let points_after =
        upsert_balance_sqlite(&mut tx, tenant_id, command, &ledger_id, timestamp).await?;
    sqlx::query(
        "UPDATE game_point_ledger SET points_after = ?3, updated_at = ?4, version = version + 1 \
         WHERE tenant_id = ?1 AND id = ?2",
    )
    .bind(tenant_id)
    .bind(&ledger_id)
    .bind(points_after)
    .bind(timestamp)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;

    let row = get_ledger_by_id_sqlite(&mut tx, tenant_id, &ledger_id).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    Ok(row)
}

async fn get_existing_ledger_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    command: &AppendPointLedgerCommand,
) -> GamePointResult<Option<GamePointLedgerEntry>> {
    let row = sqlx::query_as::<_, LedgerRow>(&format!(
        "SELECT {LEDGER_COLUMNS} FROM game_point_ledger \
         WHERE tenant_id = $1 AND idempotency_key = $2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    map_existing_row(row, command)
}

async fn get_existing_ledger_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    command: &AppendPointLedgerCommand,
) -> GamePointResult<Option<GamePointLedgerEntry>> {
    let row = sqlx::query_as::<_, LedgerRow>(&format!(
        "SELECT {LEDGER_COLUMNS} FROM game_point_ledger \
         WHERE tenant_id = ?1 AND idempotency_key = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    map_existing_row(row, command)
}

fn map_existing_row(
    row: Option<LedgerRow>,
    command: &AppendPointLedgerCommand,
) -> GamePointResult<Option<GamePointLedgerEntry>> {
    let Some(row) = row else {
        return Ok(None);
    };
    let item = row.into_item();
    ensure_idempotent_replay(&item, command)?;
    Ok(Some(item))
}

async fn upsert_balance_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    command: &AppendPointLedgerCommand,
    ledger_id: &str,
    timestamp: &str,
) -> GamePointResult<i64> {
    let points_after: i64 = sqlx::query_scalar(
        "INSERT INTO game_point_balance \
         (id, uuid, tenant_id, organization_id, ledger_account_id, game_id, mode_id, season_id, \
          user_id, points, last_ledger_id, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, $9, $10, $11, $11) \
         ON CONFLICT (tenant_id, ledger_account_id) DO UPDATE SET \
          points = game_point_balance.points + EXCLUDED.points, \
          last_ledger_id = EXCLUDED.last_ledger_id, \
          updated_at = EXCLUDED.updated_at, \
          version = game_point_balance.version + 1 \
         RETURNING points",
    )
    .bind(uuid())
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.ledger_account_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.season_id)
    .bind(&command.user_id)
    .bind(signed_delta(command))
    .bind(ledger_id)
    .bind(timestamp)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(points_after)
}

async fn upsert_balance_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    command: &AppendPointLedgerCommand,
    ledger_id: &str,
    timestamp: &str,
) -> GamePointResult<i64> {
    sqlx::query(
        "INSERT INTO game_point_balance \
         (id, uuid, tenant_id, organization_id, ledger_account_id, game_id, mode_id, season_id, \
          user_id, points, last_ledger_id, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11) \
         ON CONFLICT (tenant_id, ledger_account_id) DO UPDATE SET \
          points = game_point_balance.points + excluded.points, \
          last_ledger_id = excluded.last_ledger_id, \
          updated_at = excluded.updated_at, \
          version = game_point_balance.version + 1",
    )
    .bind(uuid())
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.ledger_account_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.season_id)
    .bind(&command.user_id)
    .bind(signed_delta(command))
    .bind(ledger_id)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;

    let points_after: i64 = sqlx::query_scalar(
        "SELECT points FROM game_point_balance WHERE tenant_id = ?1 AND ledger_account_id = ?2",
    )
    .bind(tenant_id)
    .bind(&command.ledger_account_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(points_after)
}

async fn get_ledger_by_id_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    ledger_id: &str,
) -> GamePointResult<GamePointLedgerEntry> {
    let row = sqlx::query_as::<_, LedgerRow>(&format!(
        "SELECT {LEDGER_COLUMNS} FROM game_point_ledger WHERE tenant_id = ?1 AND id = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(ledger_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GamePointError::not_found("point ledger not found"))?;
    Ok(row.into_item())
}

async fn get_balance_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    ledger_account_id: &str,
) -> GamePointResult<GamePointBalance> {
    let row = sqlx::query_as::<_, BalanceRow>(&format!(
        "SELECT {BALANCE_COLUMNS} FROM game_point_balance \
         WHERE tenant_id = $1 AND ledger_account_id = $2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(ledger_account_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GamePointError::not_found("point balance not found"))?;
    Ok(row.into_item())
}

async fn get_balance_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    ledger_account_id: &str,
) -> GamePointResult<GamePointBalance> {
    let row = sqlx::query_as::<_, BalanceRow>(&format!(
        "SELECT {BALANCE_COLUMNS} FROM game_point_balance \
         WHERE tenant_id = ?1 AND ledger_account_id = ?2 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(ledger_account_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GamePointError::not_found("point balance not found"))?;
    Ok(row.into_item())
}

fn signed_delta(command: &AppendPointLedgerCommand) -> i64 {
    if command.direction == "debit" {
        -command.points_delta
    } else {
        command.points_delta
    }
}

fn ensure_idempotent_replay(
    existing: &GamePointLedgerEntry,
    command: &AppendPointLedgerCommand,
) -> GamePointResult<()> {
    let same_payload = existing.ledger_account_id == command.ledger_account_id
        && existing.game_id == command.game_id
        && existing.mode_id == command.mode_id
        && existing.season_id == command.season_id
        && existing.user_id == command.user_id
        && existing.direction == command.direction
        && existing.points_delta == command.points_delta
        && existing.source_event_id == command.source_event_id
        && existing.reason_code == command.reason_code;
    if !same_payload {
        return Err(GamePointError::conflict(
            "idempotency_key already belongs to a different point ledger payload",
        ));
    }
    Ok(())
}

fn map_sqlx_error(error: sqlx::Error) -> GamePointError {
    GamePointError::invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
    use sdkwork_database_sqlx::create_pool_from_config;

    use super::*;

    async fn sqlite_repo() -> SqlxGamePointRepository {
        let pool = create_pool_from_config(DatabaseConfig {
            engine: DatabaseEngine::Sqlite,
            url: "sqlite::memory:".into(),
            max_connections: 1,
            ..Default::default()
        })
        .await
        .unwrap();
        pool.execute_raw(
            "CREATE TABLE game_point_ledger (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              ledger_account_id TEXT NOT NULL,
              game_id TEXT NOT NULL,
              mode_id TEXT,
              season_id TEXT,
              user_id TEXT NOT NULL,
              direction TEXT NOT NULL,
              points_delta INTEGER NOT NULL DEFAULT 0,
              points_after INTEGER,
              source_event_id TEXT NOT NULL,
              reason_code TEXT NOT NULL,
              idempotency_key TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 0,
              UNIQUE (tenant_id, idempotency_key)
            );
            CREATE TABLE game_point_balance (
              id TEXT PRIMARY KEY,
              uuid TEXT NOT NULL UNIQUE,
              tenant_id TEXT NOT NULL,
              organization_id TEXT NOT NULL DEFAULT '0',
              ledger_account_id TEXT NOT NULL,
              game_id TEXT NOT NULL,
              mode_id TEXT,
              season_id TEXT,
              user_id TEXT NOT NULL,
              points INTEGER NOT NULL DEFAULT 0,
              last_ledger_id TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 0,
              UNIQUE (tenant_id, ledger_account_id)
            );",
        )
        .await
        .unwrap();
        SqlxGamePointRepository::new(pool)
    }

    fn command(idempotency_key: &str, points_delta: i64) -> AppendPointLedgerCommand {
        AppendPointLedgerCommand {
            ledger_account_id: "acct-game-1-user-1".into(),
            game_id: "game-xiangqi".into(),
            mode_id: Some("mode-ranked".into()),
            season_id: Some("season-2026".into()),
            user_id: "user-1".into(),
            direction: "credit".into(),
            points_delta,
            source_event_id: format!("score-event-{idempotency_key}"),
            reason_code: "match_win".into(),
            idempotency_key: idempotency_key.into(),
        }
    }

    #[tokio::test]
    async fn sqlite_append_ledger_is_idempotent_and_projects_balance() {
        let repo = sqlite_repo().await;
        let first_command = command("idem-sqlite-1", 30);
        let second_command = command("idem-sqlite-2", 15);

        let first = repo.append_ledger("100001", &first_command).await.unwrap();
        let replay = repo.append_ledger("100001", &first_command).await.unwrap();
        let second = repo.append_ledger("100001", &second_command).await.unwrap();
        let balance = repo
            .get_balance("100001", &first_command.ledger_account_id)
            .await
            .unwrap();

        assert_eq!(first.id, replay.id);
        assert_eq!(30, replay.points_after);
        assert_eq!(45, second.points_after);
        assert_eq!(45, balance.points);
        assert_eq!(Some(second.id), balance.last_ledger_id);
    }

    #[tokio::test]
    async fn sqlite_append_ledger_rejects_conflicting_idempotency_payload() {
        let repo = sqlite_repo().await;
        let first_command = command("idem-sqlite-conflict", 30);
        repo.append_ledger("100001", &first_command).await.unwrap();

        let mut conflict = first_command.clone();
        conflict.reason_code = "manual_adjustment".into();

        let error = repo.append_ledger("100001", &conflict).await.unwrap_err();
        assert_eq!("conflict", error.code());
    }
}
