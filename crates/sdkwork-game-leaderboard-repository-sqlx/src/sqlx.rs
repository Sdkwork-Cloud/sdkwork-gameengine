use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_leaderboard_service::{
    LeaderboardConfigItem, LeaderboardConfigPage, LeaderboardConfigQuery,
    LeaderboardEntriesRebuildCommand, LeaderboardEntry, LeaderboardEntryUpdateCommand,
    LeaderboardError, LeaderboardPage, LeaderboardQuery, LeaderboardRepository, LeaderboardResult,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxLeaderboardRepository {
    pool: DatabasePool,
}

impl SqlxLeaderboardRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LeaderboardRepository for SqlxLeaderboardRepository {
    async fn list_configs(
        &self,
        tenant_id: &str,
        query: &LeaderboardConfigQuery,
    ) -> LeaderboardResult<LeaderboardConfigPage> {
        if is_blank(Some(tenant_id)) {
            return Err(LeaderboardError::invalid("tenant_id is required"));
        }

        let limit = query.limit() as i64;
        let offset = query.offset() as i64;
        let game_id = query.game_id.as_deref();
        let status = query
            .status
            .as_deref()
            .filter(|value| !is_blank(Some(value)));

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                list_configs_postgres(pool, tenant_id, game_id, status, limit, offset).await
            }
            DatabasePool::Sqlite(pool, _) => {
                list_configs_sqlite(pool, tenant_id, game_id, status, limit, offset).await
            }
        }
    }

    async fn get_config(
        &self,
        tenant_id: &str,
        leaderboard_id: &str,
    ) -> LeaderboardResult<LeaderboardConfigItem> {
        if is_blank(Some(tenant_id)) {
            return Err(LeaderboardError::invalid("tenant_id is required"));
        }
        if is_blank(Some(leaderboard_id)) {
            return Err(LeaderboardError::invalid("leaderboard_id is required"));
        }

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                get_config_postgres(pool, tenant_id, leaderboard_id).await
            }
            DatabasePool::Sqlite(pool, _) => {
                get_config_sqlite(pool, tenant_id, leaderboard_id).await
            }
        }
    }

    async fn list_rankings(
        &self,
        tenant_id: &str,
        query: &LeaderboardQuery,
    ) -> LeaderboardResult<LeaderboardPage> {
        if is_blank(Some(tenant_id)) {
            return Err(LeaderboardError::invalid("tenant_id is required"));
        }

        let limit = query.limit() as i64;
        let offset = query.offset() as i64;
        let leaderboard_id = query.leaderboard_id.as_deref();
        let game_id = query.game_id.as_deref();

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                list_rankings_postgres(
                    pool,
                    tenant_id,
                    leaderboard_id,
                    game_id,
                    limit,
                    offset,
                    query,
                )
                .await
            }
            DatabasePool::Sqlite(pool, _) => {
                list_rankings_sqlite(
                    pool,
                    tenant_id,
                    leaderboard_id,
                    game_id,
                    limit,
                    offset,
                    query,
                )
                .await
            }
        }
    }

    async fn get_user_ranking(
        &self,
        tenant_id: &str,
        user_id: &str,
        game_id: Option<&str>,
    ) -> LeaderboardResult<LeaderboardEntry> {
        if is_blank(Some(tenant_id)) {
            return Err(LeaderboardError::invalid("tenant_id is required"));
        }
        if is_blank(Some(user_id)) {
            return Err(LeaderboardError::invalid("user_id is required"));
        }

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                get_user_postgres(pool, tenant_id, user_id, game_id).await
            }
            DatabasePool::Sqlite(pool, _) => {
                get_user_sqlite(pool, tenant_id, user_id, game_id).await
            }
        }
    }

    async fn upsert_entry(
        &self,
        tenant_id: &str,
        command: &LeaderboardEntryUpdateCommand,
    ) -> LeaderboardResult<LeaderboardEntry> {
        if is_blank(Some(tenant_id)) {
            return Err(LeaderboardError::invalid("tenant_id is required"));
        }
        let timestamp = now().to_rfc3339();

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                upsert_entry_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                upsert_entry_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn rebuild_entries(
        &self,
        tenant_id: &str,
        command: &LeaderboardEntriesRebuildCommand,
    ) -> LeaderboardResult<LeaderboardPage> {
        if is_blank(Some(tenant_id)) {
            return Err(LeaderboardError::invalid("tenant_id is required"));
        }
        let timestamp = now().to_rfc3339();

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                rebuild_entries_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                rebuild_entries_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct ConfigRow {
    id: String,
    game_id: String,
    mode_id: Option<String>,
    season_id: Option<String>,
    leaderboard_code: String,
    title: String,
    status: String,
    ranking_metric: String,
    ranking_order: String,
    tie_breaker: String,
    version: i64,
}

impl ConfigRow {
    fn into_item(self) -> LeaderboardConfigItem {
        LeaderboardConfigItem {
            id: self.id,
            game_id: self.game_id,
            mode_id: self.mode_id,
            season_id: self.season_id,
            leaderboard_code: self.leaderboard_code,
            title: self.title,
            status: self.status,
            ranking_metric: self.ranking_metric,
            ranking_order: self.ranking_order,
            tie_breaker: self.tie_breaker,
            version: self.version,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LeaderboardRow {
    id: String,
    leaderboard_id: String,
    game_id: String,
    user_id: String,
    display_name_snapshot: Option<String>,
    score_value: i64,
    rank_no: Option<i32>,
    recorded_at: String,
}

impl LeaderboardRow {
    fn into_entry(self, rank_no: i32) -> LeaderboardEntry {
        LeaderboardEntry {
            id: self.id,
            game_id: self.game_id,
            user_id: self.user_id,
            display_name: self.display_name_snapshot,
            score: self.score_value,
            rank_no: Some(self.rank_no.unwrap_or(rank_no)),
            recorded_at: self.recorded_at,
        }
    }
}

const CONFIG_COLUMNS: &str = "id, game_id, mode_id, season_id, leaderboard_code, title, status, \
ranking_metric, ranking_order, tie_breaker, version";
const ENTRY_COLUMNS: &str = "id, leaderboard_id, game_id, user_id, display_name_snapshot, \
score_value, rank_no, recorded_at";

async fn list_configs_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    game_id: Option<&str>,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> LeaderboardResult<LeaderboardConfigPage> {
    let rows = sqlx::query_as::<_, ConfigRow>(&format!(
        "SELECT {CONFIG_COLUMNS} FROM game_leaderboard_config \
         WHERE tenant_id = $1 AND deleted_at IS NULL \
         AND ($2::text IS NULL OR game_id = $2) \
         AND ($3::text IS NULL OR status = $3) \
         ORDER BY sort_order ASC, leaderboard_code ASC LIMIT $4 OFFSET $5",
    ))
    .bind(tenant_id)
    .bind(game_id)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_leaderboard_config \
         WHERE tenant_id = $1 AND deleted_at IS NULL \
         AND ($2::text IS NULL OR game_id = $2) \
         AND ($3::text IS NULL OR status = $3)",
    )
    .bind(tenant_id)
    .bind(game_id)
    .bind(status)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(config_page_from_rows(rows, total, limit, offset))
}

async fn list_configs_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    game_id: Option<&str>,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> LeaderboardResult<LeaderboardConfigPage> {
    let rows = sqlx::query_as::<_, ConfigRow>(&format!(
        "SELECT {CONFIG_COLUMNS} FROM game_leaderboard_config \
         WHERE tenant_id = ?1 AND deleted_at IS NULL \
         AND (?2 IS NULL OR game_id = ?2) \
         AND (?3 IS NULL OR status = ?3) \
         ORDER BY sort_order ASC, leaderboard_code ASC LIMIT ?4 OFFSET ?5",
    ))
    .bind(tenant_id)
    .bind(game_id)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_leaderboard_config \
         WHERE tenant_id = ?1 AND deleted_at IS NULL \
         AND (?2 IS NULL OR game_id = ?2) \
         AND (?3 IS NULL OR status = ?3)",
    )
    .bind(tenant_id)
    .bind(game_id)
    .bind(status)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(config_page_from_rows(rows, total, limit, offset))
}

fn config_page_from_rows(
    rows: Vec<ConfigRow>,
    total: i64,
    limit: i64,
    offset: i64,
) -> LeaderboardConfigPage {
    LeaderboardConfigPage {
        items: rows.into_iter().map(ConfigRow::into_item).collect(),
        total: total as u64,
        page: ((offset / limit) + 1) as u32,
        page_size: limit as u32,
    }
}

async fn get_config_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    leaderboard_id: &str,
) -> LeaderboardResult<LeaderboardConfigItem> {
    let row = sqlx::query_as::<_, ConfigRow>(&format!(
        "SELECT {CONFIG_COLUMNS} FROM game_leaderboard_config \
         WHERE tenant_id = $1 AND deleted_at IS NULL AND (id = $2 OR leaderboard_code = $2) LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(leaderboard_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| LeaderboardError::not_found("leaderboard config not found"))?;
    Ok(row.into_item())
}

async fn get_config_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    leaderboard_id: &str,
) -> LeaderboardResult<LeaderboardConfigItem> {
    let row = sqlx::query_as::<_, ConfigRow>(&format!(
        "SELECT {CONFIG_COLUMNS} FROM game_leaderboard_config \
         WHERE tenant_id = ?1 AND deleted_at IS NULL AND (id = ?2 OR leaderboard_code = ?2) LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(leaderboard_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| LeaderboardError::not_found("leaderboard config not found"))?;
    Ok(row.into_item())
}

async fn list_rankings_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    leaderboard_id: Option<&str>,
    game_id: Option<&str>,
    limit: i64,
    offset: i64,
    query: &LeaderboardQuery,
) -> LeaderboardResult<LeaderboardPage> {
    let rows = sqlx::query_as::<_, LeaderboardRow>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM game_leaderboard_entry \
         WHERE tenant_id = $1 \
         AND ($2::text IS NULL OR leaderboard_id = $2) \
         AND ($3::text IS NULL OR game_id = $3) \
         ORDER BY score_value DESC, recorded_at ASC, id ASC LIMIT $4 OFFSET $5",
    ))
    .bind(tenant_id)
    .bind(leaderboard_id)
    .bind(game_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_leaderboard_entry \
         WHERE tenant_id = $1 \
         AND ($2::text IS NULL OR leaderboard_id = $2) \
         AND ($3::text IS NULL OR game_id = $3)",
    )
    .bind(tenant_id)
    .bind(leaderboard_id)
    .bind(game_id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(entry_page_from_rows(rows, total, limit, offset, query))
}

async fn list_rankings_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    leaderboard_id: Option<&str>,
    game_id: Option<&str>,
    limit: i64,
    offset: i64,
    query: &LeaderboardQuery,
) -> LeaderboardResult<LeaderboardPage> {
    let rows = sqlx::query_as::<_, LeaderboardRow>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM game_leaderboard_entry \
         WHERE tenant_id = ?1 \
         AND (?2 IS NULL OR leaderboard_id = ?2) \
         AND (?3 IS NULL OR game_id = ?3) \
         ORDER BY score_value DESC, recorded_at ASC, id ASC LIMIT ?4 OFFSET ?5",
    ))
    .bind(tenant_id)
    .bind(leaderboard_id)
    .bind(game_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_leaderboard_entry \
         WHERE tenant_id = ?1 \
         AND (?2 IS NULL OR leaderboard_id = ?2) \
         AND (?3 IS NULL OR game_id = ?3)",
    )
    .bind(tenant_id)
    .bind(leaderboard_id)
    .bind(game_id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(entry_page_from_rows(rows, total, limit, offset, query))
}

fn entry_page_from_rows(
    rows: Vec<LeaderboardRow>,
    total: i64,
    limit: i64,
    offset: i64,
    query: &LeaderboardQuery,
) -> LeaderboardPage {
    let items = rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| row.into_entry((offset as i32) + index as i32 + 1))
        .collect();
    LeaderboardPage {
        items,
        total: total as u64,
        page: query.page.unwrap_or(1),
        page_size: limit as u32,
    }
}

async fn get_user_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    user_id: &str,
    game_id: Option<&str>,
) -> LeaderboardResult<LeaderboardEntry> {
    let row = sqlx::query_as::<_, LeaderboardRow>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM game_leaderboard_entry \
         WHERE tenant_id = $1 AND user_id = $2 \
         AND ($3::text IS NULL OR game_id = $3) \
         ORDER BY score_value DESC, recorded_at ASC, id ASC LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(user_id)
    .bind(game_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| LeaderboardError::not_found("leaderboard entry not found"))?;

    let rank_no = resolve_rank_postgres(pool, tenant_id, &row).await?;
    Ok(row.into_entry(rank_no))
}

async fn get_user_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    user_id: &str,
    game_id: Option<&str>,
) -> LeaderboardResult<LeaderboardEntry> {
    let row = if let Some(game_id) = game_id {
        sqlx::query_as::<_, LeaderboardRow>(&format!(
            "SELECT {ENTRY_COLUMNS} FROM game_leaderboard_entry \
             WHERE tenant_id = ?1 AND user_id = ?2 AND game_id = ?3 \
             ORDER BY score_value DESC, recorded_at ASC, id ASC LIMIT 1",
        ))
        .bind(tenant_id)
        .bind(user_id)
        .bind(game_id)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_as::<_, LeaderboardRow>(&format!(
            "SELECT {ENTRY_COLUMNS} FROM game_leaderboard_entry \
             WHERE tenant_id = ?1 AND user_id = ?2 \
             ORDER BY score_value DESC, recorded_at ASC, id ASC LIMIT 1",
        ))
        .bind(tenant_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }
    .map_err(map_sqlx_error)?
    .ok_or_else(|| LeaderboardError::not_found("leaderboard entry not found"))?;

    let rank_no = resolve_rank_sqlite(pool, tenant_id, &row).await?;
    Ok(row.into_entry(rank_no))
}

async fn resolve_rank_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    row: &LeaderboardRow,
) -> LeaderboardResult<i32> {
    if let Some(rank_no) = row.rank_no {
        return Ok(rank_no);
    }
    let computed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) + 1 FROM game_leaderboard_entry \
         WHERE tenant_id = $1 AND leaderboard_id = $2 \
         AND (score_value > $3 OR (score_value = $3 AND recorded_at < $4) \
          OR (score_value = $3 AND recorded_at = $4 AND id < $5))",
    )
    .bind(tenant_id)
    .bind(&row.leaderboard_id)
    .bind(row.score_value)
    .bind(&row.recorded_at)
    .bind(&row.id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    Ok(computed as i32)
}

async fn resolve_rank_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    row: &LeaderboardRow,
) -> LeaderboardResult<i32> {
    if let Some(rank_no) = row.rank_no {
        return Ok(rank_no);
    }
    let computed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) + 1 FROM game_leaderboard_entry \
         WHERE tenant_id = ?1 AND leaderboard_id = ?2 \
         AND (score_value > ?3 OR (score_value = ?3 AND recorded_at < ?4) \
          OR (score_value = ?3 AND recorded_at = ?4 AND id < ?5))",
    )
    .bind(tenant_id)
    .bind(&row.leaderboard_id)
    .bind(row.score_value)
    .bind(&row.recorded_at)
    .bind(&row.id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;
    Ok(computed as i32)
}

async fn upsert_entry_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &LeaderboardEntryUpdateCommand,
    timestamp: &str,
) -> LeaderboardResult<LeaderboardEntry> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    insert_or_update_entry_postgres(&mut tx, tenant_id, command, timestamp).await?;
    refresh_ranks_postgres(&mut tx, tenant_id, &command.leaderboard_id, timestamp).await?;
    let row = get_entry_by_user_postgres(&mut tx, tenant_id, command).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    let rank_no = row.rank_no.unwrap_or(1);
    Ok(row.into_entry(rank_no))
}

async fn upsert_entry_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &LeaderboardEntryUpdateCommand,
    timestamp: &str,
) -> LeaderboardResult<LeaderboardEntry> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    insert_or_update_entry_sqlite(&mut tx, tenant_id, command, timestamp).await?;
    refresh_ranks_sqlite(&mut tx, tenant_id, &command.leaderboard_id, timestamp).await?;
    let row = get_entry_by_user_sqlite(&mut tx, tenant_id, command).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    let rank_no = row.rank_no.unwrap_or(1);
    Ok(row.into_entry(rank_no))
}

async fn rebuild_entries_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &LeaderboardEntriesRebuildCommand,
    timestamp: &str,
) -> LeaderboardResult<LeaderboardPage> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    sqlx::query("DELETE FROM game_leaderboard_entry WHERE tenant_id = $1 AND leaderboard_id = $2")
        .bind(tenant_id)
        .bind(&command.leaderboard_id)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;
    for entry in &command.entries {
        insert_entry_postgres(&mut tx, tenant_id, entry, timestamp).await?;
    }
    refresh_ranks_postgres(&mut tx, tenant_id, &command.leaderboard_id, timestamp).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    list_rankings_postgres(
        pool,
        tenant_id,
        Some(&command.leaderboard_id),
        None,
        200,
        0,
        &LeaderboardQuery {
            leaderboard_id: Some(command.leaderboard_id.clone()),
            page_size: Some(200),
            ..Default::default()
        },
    )
    .await
}

async fn rebuild_entries_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &LeaderboardEntriesRebuildCommand,
    timestamp: &str,
) -> LeaderboardResult<LeaderboardPage> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    sqlx::query("DELETE FROM game_leaderboard_entry WHERE tenant_id = ?1 AND leaderboard_id = ?2")
        .bind(tenant_id)
        .bind(&command.leaderboard_id)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;
    for entry in &command.entries {
        insert_entry_sqlite(&mut tx, tenant_id, entry, timestamp).await?;
    }
    refresh_ranks_sqlite(&mut tx, tenant_id, &command.leaderboard_id, timestamp).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    list_rankings_sqlite(
        pool,
        tenant_id,
        Some(&command.leaderboard_id),
        None,
        200,
        0,
        &LeaderboardQuery {
            leaderboard_id: Some(command.leaderboard_id.clone()),
            page_size: Some(200),
            ..Default::default()
        },
    )
    .await
}

async fn insert_or_update_entry_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    command: &LeaderboardEntryUpdateCommand,
    timestamp: &str,
) -> LeaderboardResult<()> {
    let recorded_at = command.recorded_at.as_deref().unwrap_or(timestamp);
    sqlx::query(
        "INSERT INTO game_leaderboard_entry \
         (id, uuid, tenant_id, organization_id, leaderboard_id, game_id, mode_id, season_id, \
          user_id, display_name_snapshot, score_value, tie_breaker_value, last_ledger_id, \
          recorded_at, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14) \
         ON CONFLICT (tenant_id, leaderboard_id, user_id) DO UPDATE SET \
          game_id = EXCLUDED.game_id, mode_id = EXCLUDED.mode_id, season_id = EXCLUDED.season_id, \
          display_name_snapshot = EXCLUDED.display_name_snapshot, score_value = EXCLUDED.score_value, \
          tie_breaker_value = EXCLUDED.tie_breaker_value, last_ledger_id = EXCLUDED.last_ledger_id, \
          recorded_at = EXCLUDED.recorded_at, updated_at = EXCLUDED.updated_at, \
          version = game_leaderboard_entry.version + 1",
    )
    .bind(uuid())
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.leaderboard_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.season_id)
    .bind(&command.user_id)
    .bind(&command.display_name_snapshot)
    .bind(command.score_value)
    .bind(&command.tie_breaker_value)
    .bind(&command.last_ledger_id)
    .bind(recorded_at)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn insert_or_update_entry_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    command: &LeaderboardEntryUpdateCommand,
    timestamp: &str,
) -> LeaderboardResult<()> {
    let recorded_at = command.recorded_at.as_deref().unwrap_or(timestamp);
    sqlx::query(
        "INSERT INTO game_leaderboard_entry \
         (id, uuid, tenant_id, organization_id, leaderboard_id, game_id, mode_id, season_id, \
          user_id, display_name_snapshot, score_value, tie_breaker_value, last_ledger_id, \
          recorded_at, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14) \
         ON CONFLICT (tenant_id, leaderboard_id, user_id) DO UPDATE SET \
          game_id = excluded.game_id, mode_id = excluded.mode_id, season_id = excluded.season_id, \
          display_name_snapshot = excluded.display_name_snapshot, score_value = excluded.score_value, \
          tie_breaker_value = excluded.tie_breaker_value, last_ledger_id = excluded.last_ledger_id, \
          recorded_at = excluded.recorded_at, updated_at = excluded.updated_at, \
          version = game_leaderboard_entry.version + 1",
    )
    .bind(uuid())
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.leaderboard_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.season_id)
    .bind(&command.user_id)
    .bind(&command.display_name_snapshot)
    .bind(command.score_value)
    .bind(&command.tie_breaker_value)
    .bind(&command.last_ledger_id)
    .bind(recorded_at)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn insert_entry_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    command: &LeaderboardEntryUpdateCommand,
    timestamp: &str,
) -> LeaderboardResult<()> {
    let recorded_at = command.recorded_at.as_deref().unwrap_or(timestamp);
    sqlx::query(
        "INSERT INTO game_leaderboard_entry \
         (id, uuid, tenant_id, organization_id, leaderboard_id, game_id, mode_id, season_id, \
          user_id, display_name_snapshot, score_value, tie_breaker_value, last_ledger_id, \
          recorded_at, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14)",
    )
    .bind(uuid())
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.leaderboard_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.season_id)
    .bind(&command.user_id)
    .bind(&command.display_name_snapshot)
    .bind(command.score_value)
    .bind(&command.tie_breaker_value)
    .bind(&command.last_ledger_id)
    .bind(recorded_at)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn insert_entry_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    command: &LeaderboardEntryUpdateCommand,
    timestamp: &str,
) -> LeaderboardResult<()> {
    let recorded_at = command.recorded_at.as_deref().unwrap_or(timestamp);
    sqlx::query(
        "INSERT INTO game_leaderboard_entry \
         (id, uuid, tenant_id, organization_id, leaderboard_id, game_id, mode_id, season_id, \
          user_id, display_name_snapshot, score_value, tie_breaker_value, last_ledger_id, \
          recorded_at, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)",
    )
    .bind(uuid())
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.leaderboard_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.season_id)
    .bind(&command.user_id)
    .bind(&command.display_name_snapshot)
    .bind(command.score_value)
    .bind(&command.tie_breaker_value)
    .bind(&command.last_ledger_id)
    .bind(recorded_at)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn refresh_ranks_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    leaderboard_id: &str,
    timestamp: &str,
) -> LeaderboardResult<()> {
    sqlx::query(
        "WITH ranked AS (
           SELECT id, ROW_NUMBER() OVER (ORDER BY score_value DESC, recorded_at ASC, id ASC) AS new_rank
           FROM game_leaderboard_entry WHERE tenant_id = $1 AND leaderboard_id = $2
         )
         UPDATE game_leaderboard_entry AS entry
         SET rank_no = ranked.new_rank, updated_at = $3, version = entry.version + 1
         FROM ranked WHERE entry.id = ranked.id",
    )
    .bind(tenant_id)
    .bind(leaderboard_id)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn refresh_ranks_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    leaderboard_id: &str,
    timestamp: &str,
) -> LeaderboardResult<()> {
    sqlx::query(
        "WITH ranked AS (
           SELECT id, ROW_NUMBER() OVER (ORDER BY score_value DESC, recorded_at ASC, id ASC) AS new_rank
           FROM game_leaderboard_entry WHERE tenant_id = ?1 AND leaderboard_id = ?2
         )
         UPDATE game_leaderboard_entry
         SET rank_no = (SELECT new_rank FROM ranked WHERE ranked.id = game_leaderboard_entry.id),
             updated_at = ?3,
             version = version + 1
         WHERE tenant_id = ?1 AND leaderboard_id = ?2",
    )
    .bind(tenant_id)
    .bind(leaderboard_id)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn get_entry_by_user_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    command: &LeaderboardEntryUpdateCommand,
) -> LeaderboardResult<LeaderboardRow> {
    sqlx::query_as::<_, LeaderboardRow>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM game_leaderboard_entry \
         WHERE tenant_id = $1 AND leaderboard_id = $2 AND user_id = $3 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.leaderboard_id)
    .bind(&command.user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| LeaderboardError::not_found("leaderboard entry not found"))
}

async fn get_entry_by_user_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    command: &LeaderboardEntryUpdateCommand,
) -> LeaderboardResult<LeaderboardRow> {
    sqlx::query_as::<_, LeaderboardRow>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM game_leaderboard_entry \
         WHERE tenant_id = ?1 AND leaderboard_id = ?2 AND user_id = ?3 LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(&command.leaderboard_id)
    .bind(&command.user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| LeaderboardError::not_found("leaderboard entry not found"))
}

fn map_sqlx_error(error: sqlx::Error) -> LeaderboardError {
    LeaderboardError::invalid(error.to_string())
}
