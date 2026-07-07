use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_mode_service::{
    CreateGameModeCommand, GameModeError, GameModeItem, GameModePage, GameModeQuery,
    GameModeRepository, GameModeResult, UpdateGameModeCommand,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxGameModeRepository {
    pool: DatabasePool,
}

impl SqlxGameModeRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameModeRepository for SqlxGameModeRepository {
    async fn list_modes(
        &self,
        tenant_id: &str,
        query: &GameModeQuery,
    ) -> GameModeResult<GameModePage> {
        if is_blank(Some(tenant_id)) {
            return Err(GameModeError::invalid("tenant_id is required"));
        }
        let limit = query.limit() as i64;
        let offset = query.offset() as i64;
        let game_id = query.game_id.as_deref();
        let status = query.status.as_deref();
        let q = query
            .q
            .as_deref()
            .filter(|value| !is_blank(Some(value)))
            .map(|value| format!("%{}%", value.trim()));

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                list_postgres(
                    pool,
                    tenant_id,
                    game_id,
                    status,
                    q.as_deref(),
                    limit,
                    offset,
                )
                .await
            }
            DatabasePool::Sqlite(pool, _) => {
                list_sqlite(
                    pool,
                    tenant_id,
                    game_id,
                    status,
                    q.as_deref(),
                    limit,
                    offset,
                )
                .await
            }
        }
    }

    async fn get_mode(&self, tenant_id: &str, mode_id: &str) -> GameModeResult<GameModeItem> {
        if is_blank(Some(tenant_id)) {
            return Err(GameModeError::invalid("tenant_id is required"));
        }
        if is_blank(Some(mode_id)) {
            return Err(GameModeError::invalid("mode_id is required"));
        }

        match &self.pool {
            DatabasePool::Postgres(pool, _) => get_postgres(pool, tenant_id, mode_id).await,
            DatabasePool::Sqlite(pool, _) => get_sqlite(pool, tenant_id, mode_id).await,
        }
    }

    async fn create_mode(
        &self,
        tenant_id: &str,
        command: &CreateGameModeCommand,
    ) -> GameModeResult<GameModeItem> {
        let id = uuid();
        let now = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                create_postgres(pool, tenant_id, &id, &now, command).await
            }
            DatabasePool::Sqlite(pool, _) => {
                create_sqlite(pool, tenant_id, &id, &now, command).await
            }
        }
    }

    async fn update_mode(
        &self,
        tenant_id: &str,
        mode_id: &str,
        command: &UpdateGameModeCommand,
    ) -> GameModeResult<GameModeItem> {
        let current = self.get_mode(tenant_id, mode_id).await?;
        let updated = merge_update(current, command);
        let now = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                update_postgres(pool, tenant_id, &updated, &now).await
            }
            DatabasePool::Sqlite(pool, _) => update_sqlite(pool, tenant_id, &updated, &now).await,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ModeRow {
    id: String,
    game_id: String,
    mode_code: String,
    title: String,
    status: String,
    min_players: i32,
    max_players: i32,
    team_size: Option<i32>,
    ruleset_id: Option<String>,
    matchmaking_enabled: bool,
    room_enabled: bool,
    leaderboard_enabled: bool,
}

impl ModeRow {
    fn into_item(self) -> GameModeItem {
        GameModeItem {
            id: self.id,
            game_id: self.game_id,
            mode_code: self.mode_code,
            title: self.title,
            status: self.status,
            min_players: self.min_players,
            max_players: self.max_players,
            team_size: self.team_size,
            ruleset_id: self.ruleset_id,
            matchmaking_enabled: self.matchmaking_enabled,
            room_enabled: self.room_enabled,
            leaderboard_enabled: self.leaderboard_enabled,
        }
    }
}

const MODE_COLUMNS: &str = "id, game_id, mode_code, title, status, min_players, max_players, \
team_size, ruleset_id, matchmaking_enabled, room_enabled, leaderboard_enabled";

async fn list_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    game_id: Option<&str>,
    status: Option<&str>,
    q: Option<&str>,
    limit: i64,
    offset: i64,
) -> GameModeResult<GameModePage> {
    let rows = sqlx::query_as::<_, ModeRow>(&format!(
        "SELECT {MODE_COLUMNS} FROM game_mode \
         WHERE tenant_id = $1 AND deleted_at IS NULL \
         AND ($2::text IS NULL OR game_id = $2) \
         AND ($3::text IS NULL OR status = $3) \
         AND ($4::text IS NULL OR LOWER(title) LIKE LOWER($4) OR LOWER(mode_code) LIKE LOWER($4)) \
         ORDER BY sort_order ASC, mode_code ASC LIMIT $5 OFFSET $6",
    ))
    .bind(tenant_id)
    .bind(game_id)
    .bind(status)
    .bind(q)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_mode \
         WHERE tenant_id = $1 AND deleted_at IS NULL \
         AND ($2::text IS NULL OR game_id = $2) \
         AND ($3::text IS NULL OR status = $3) \
         AND ($4::text IS NULL OR LOWER(title) LIKE LOWER($4) OR LOWER(mode_code) LIKE LOWER($4))",
    )
    .bind(tenant_id)
    .bind(game_id)
    .bind(status)
    .bind(q)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(page_from_rows(rows, total, limit, offset))
}

async fn list_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    game_id: Option<&str>,
    status: Option<&str>,
    q: Option<&str>,
    limit: i64,
    offset: i64,
) -> GameModeResult<GameModePage> {
    let rows = sqlx::query_as::<_, ModeRow>(&format!(
        "SELECT {MODE_COLUMNS} FROM game_mode \
         WHERE tenant_id = ?1 AND deleted_at IS NULL \
         AND (?2 IS NULL OR game_id = ?2) \
         AND (?3 IS NULL OR status = ?3) \
         AND (?4 IS NULL OR LOWER(title) LIKE LOWER(?4) OR LOWER(mode_code) LIKE LOWER(?4)) \
         ORDER BY sort_order ASC, mode_code ASC LIMIT ?5 OFFSET ?6",
    ))
    .bind(tenant_id)
    .bind(game_id)
    .bind(status)
    .bind(q)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_mode \
         WHERE tenant_id = ?1 AND deleted_at IS NULL \
         AND (?2 IS NULL OR game_id = ?2) \
         AND (?3 IS NULL OR status = ?3) \
         AND (?4 IS NULL OR LOWER(title) LIKE LOWER(?4) OR LOWER(mode_code) LIKE LOWER(?4))",
    )
    .bind(tenant_id)
    .bind(game_id)
    .bind(status)
    .bind(q)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(page_from_rows(rows, total, limit, offset))
}

fn page_from_rows(rows: Vec<ModeRow>, total: i64, limit: i64, offset: i64) -> GameModePage {
    GameModePage {
        items: rows.into_iter().map(ModeRow::into_item).collect(),
        total: total as u64,
        page: ((offset / limit) + 1) as u32,
        page_size: limit as u32,
    }
}

async fn get_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    mode_id: &str,
) -> GameModeResult<GameModeItem> {
    let row = sqlx::query_as::<_, ModeRow>(&format!(
        "SELECT {MODE_COLUMNS} FROM game_mode \
         WHERE tenant_id = $1 AND deleted_at IS NULL AND (id = $2 OR mode_code = $2) LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(mode_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameModeError::not_found("mode not found"))?;

    Ok(row.into_item())
}

async fn get_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    mode_id: &str,
) -> GameModeResult<GameModeItem> {
    let row = sqlx::query_as::<_, ModeRow>(&format!(
        "SELECT {MODE_COLUMNS} FROM game_mode \
         WHERE tenant_id = ?1 AND deleted_at IS NULL AND (id = ?2 OR mode_code = ?2) LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(mode_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameModeError::not_found("mode not found"))?;

    Ok(row.into_item())
}

async fn create_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    id: &str,
    now: &str,
    command: &CreateGameModeCommand,
) -> GameModeResult<GameModeItem> {
    let row = sqlx::query_as::<_, ModeRow>(&format!(
        "INSERT INTO game_mode \
         (id, uuid, tenant_id, organization_id, game_id, mode_code, title, status, min_players, max_players, \
          team_size, ruleset_id, matchmaking_enabled, room_enabled, leaderboard_enabled, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $15) \
         RETURNING {MODE_COLUMNS}",
    ))
    .bind(id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.game_id)
    .bind(&command.mode_code)
    .bind(&command.title)
    .bind(&command.status)
    .bind(command.min_players)
    .bind(command.max_players)
    .bind(command.team_size)
    .bind(&command.ruleset_id)
    .bind(command.matchmaking_enabled)
    .bind(command.room_enabled)
    .bind(command.leaderboard_enabled)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(row.into_item())
}

async fn create_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    id: &str,
    now: &str,
    command: &CreateGameModeCommand,
) -> GameModeResult<GameModeItem> {
    sqlx::query(
        "INSERT INTO game_mode \
         (id, uuid, tenant_id, organization_id, game_id, mode_code, title, status, min_players, max_players, \
          team_size, ruleset_id, matchmaking_enabled, room_enabled, leaderboard_enabled, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
    )
    .bind(id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.game_id)
    .bind(&command.mode_code)
    .bind(&command.title)
    .bind(&command.status)
    .bind(command.min_players)
    .bind(command.max_players)
    .bind(command.team_size)
    .bind(&command.ruleset_id)
    .bind(command.matchmaking_enabled)
    .bind(command.room_enabled)
    .bind(command.leaderboard_enabled)
    .bind(now)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;

    get_sqlite(pool, tenant_id, id).await
}

async fn update_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    item: &GameModeItem,
    now: &str,
) -> GameModeResult<GameModeItem> {
    let row = sqlx::query_as::<_, ModeRow>(&format!(
        "UPDATE game_mode SET title = $3, status = $4, min_players = $5, max_players = $6, \
         team_size = $7, ruleset_id = $8, matchmaking_enabled = $9, room_enabled = $10, \
         leaderboard_enabled = $11, updated_at = $12, version = version + 1 \
         WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL RETURNING {MODE_COLUMNS}",
    ))
    .bind(tenant_id)
    .bind(&item.id)
    .bind(&item.title)
    .bind(&item.status)
    .bind(item.min_players)
    .bind(item.max_players)
    .bind(item.team_size)
    .bind(&item.ruleset_id)
    .bind(item.matchmaking_enabled)
    .bind(item.room_enabled)
    .bind(item.leaderboard_enabled)
    .bind(now)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameModeError::not_found("mode not found"))?;

    Ok(row.into_item())
}

async fn update_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    item: &GameModeItem,
    now: &str,
) -> GameModeResult<GameModeItem> {
    let result = sqlx::query(
        "UPDATE game_mode SET title = ?3, status = ?4, min_players = ?5, max_players = ?6, \
         team_size = ?7, ruleset_id = ?8, matchmaking_enabled = ?9, room_enabled = ?10, \
         leaderboard_enabled = ?11, updated_at = ?12, version = version + 1 \
         WHERE tenant_id = ?1 AND id = ?2 AND deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(&item.id)
    .bind(&item.title)
    .bind(&item.status)
    .bind(item.min_players)
    .bind(item.max_players)
    .bind(item.team_size)
    .bind(&item.ruleset_id)
    .bind(item.matchmaking_enabled)
    .bind(item.room_enabled)
    .bind(item.leaderboard_enabled)
    .bind(now)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;
    if result.rows_affected() == 0 {
        return Err(GameModeError::not_found("mode not found"));
    }
    get_sqlite(pool, tenant_id, &item.id).await
}

fn merge_update(mut item: GameModeItem, command: &UpdateGameModeCommand) -> GameModeItem {
    if let Some(title) = &command.title {
        item.title = title.clone();
    }
    if let Some(status) = &command.status {
        item.status = status.clone();
    }
    if let Some(min_players) = command.min_players {
        item.min_players = min_players;
    }
    if let Some(max_players) = command.max_players {
        item.max_players = max_players;
    }
    if let Some(team_size) = command.team_size {
        item.team_size = team_size;
    }
    if let Some(ruleset_id) = &command.ruleset_id {
        item.ruleset_id = ruleset_id.clone();
    }
    if let Some(matchmaking_enabled) = command.matchmaking_enabled {
        item.matchmaking_enabled = matchmaking_enabled;
    }
    if let Some(room_enabled) = command.room_enabled {
        item.room_enabled = room_enabled;
    }
    if let Some(leaderboard_enabled) = command.leaderboard_enabled {
        item.leaderboard_enabled = leaderboard_enabled;
    }
    item
}

fn map_sqlx_error(error: sqlx::Error) -> GameModeError {
    GameModeError::invalid(error.to_string())
}
