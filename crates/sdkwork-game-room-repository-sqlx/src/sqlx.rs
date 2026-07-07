use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_room_service::{
    CloseGameRoomCommand, CreateGameRoomCommand, GameRoomError, GameRoomItem, GameRoomPage,
    GameRoomQuery, GameRoomRepository, GameRoomResult, GameRoomSeatItem, JoinGameRoomCommand,
    LeaveGameRoomCommand, ReadyGameRoomCommand, StartGameRoomCommand,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxGameRoomRepository {
    pool: DatabasePool,
}

impl SqlxGameRoomRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameRoomRepository for SqlxGameRoomRepository {
    async fn list_rooms(
        &self,
        tenant_id: &str,
        query: &GameRoomQuery,
    ) -> GameRoomResult<GameRoomPage> {
        if is_blank(Some(tenant_id)) {
            return Err(GameRoomError::invalid("tenant_id is required"));
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
                list_postgres(pool, tenant_id, game_id, status, limit, offset).await
            }
            DatabasePool::Sqlite(pool, _) => {
                list_sqlite(pool, tenant_id, game_id, status, limit, offset).await
            }
        }
    }

    async fn get_room(&self, tenant_id: &str, room_id: &str) -> GameRoomResult<GameRoomItem> {
        if is_blank(Some(tenant_id)) {
            return Err(GameRoomError::invalid("tenant_id is required"));
        }
        if is_blank(Some(room_id)) {
            return Err(GameRoomError::invalid("room_id is required"));
        }
        match &self.pool {
            DatabasePool::Postgres(pool, _) => get_postgres(pool, tenant_id, room_id).await,
            DatabasePool::Sqlite(pool, _) => get_sqlite(pool, tenant_id, room_id).await,
        }
    }

    async fn list_room_seats(
        &self,
        tenant_id: &str,
        room_id: &str,
    ) -> GameRoomResult<Vec<GameRoomSeatItem>> {
        if is_blank(Some(tenant_id)) {
            return Err(GameRoomError::invalid("tenant_id is required"));
        }
        if is_blank(Some(room_id)) {
            return Err(GameRoomError::invalid("room_id is required"));
        }
        match &self.pool {
            DatabasePool::Postgres(pool, _) => list_seats_postgres(pool, tenant_id, room_id).await,
            DatabasePool::Sqlite(pool, _) => list_seats_sqlite(pool, tenant_id, room_id).await,
        }
    }

    async fn create_room(
        &self,
        tenant_id: &str,
        command: &CreateGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let id = uuid();
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                create_postgres(pool, tenant_id, &id, &timestamp, command).await
            }
            DatabasePool::Sqlite(pool, _) => {
                create_sqlite(pool, tenant_id, &id, &timestamp, command).await
            }
        }
    }

    async fn join_room(
        &self,
        tenant_id: &str,
        command: &JoinGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let room = self.get_room(tenant_id, &command.room_id).await?;
        ensure_expected_version(&room, command.expected_version)?;
        let seats = self.list_room_seats(tenant_id, &command.room_id).await?;
        let seat_no = next_available_seat_no(&room, &seats)
            .ok_or_else(|| GameRoomError::invalid("room is full"))?;
        let reusable = seats
            .iter()
            .any(|seat| seat.seat_no == seat_no && !is_active_seat_status(&seat.status));
        let active_count = active_seat_count(&seats) + 1;
        let timestamp = now().to_rfc3339();

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                join_postgres(
                    pool,
                    tenant_id,
                    command,
                    seat_no,
                    reusable,
                    active_count,
                    &timestamp,
                )
                .await
            }
            DatabasePool::Sqlite(pool, _) => {
                join_sqlite(
                    pool,
                    tenant_id,
                    command,
                    seat_no,
                    reusable,
                    active_count,
                    &timestamp,
                )
                .await
            }
        }
    }

    async fn leave_room(
        &self,
        tenant_id: &str,
        command: &LeaveGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let room = self.get_room(tenant_id, &command.room_id).await?;
        ensure_expected_version(&room, command.expected_version)?;
        let seats = self.list_room_seats(tenant_id, &command.room_id).await?;
        let active_count = active_seat_count(&seats).saturating_sub(1);
        let timestamp = now().to_rfc3339();

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                leave_postgres(pool, tenant_id, command, active_count, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                leave_sqlite(pool, tenant_id, command, active_count, &timestamp).await
            }
        }
    }

    async fn set_ready(
        &self,
        tenant_id: &str,
        command: &ReadyGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let room = self.get_room(tenant_id, &command.room_id).await?;
        ensure_expected_version(&room, command.expected_version)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                ready_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                ready_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn start_room(
        &self,
        tenant_id: &str,
        command: &StartGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let room = self.get_room(tenant_id, &command.room_id).await?;
        ensure_expected_version(&room, command.expected_version)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                start_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                start_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }

    async fn close_room(
        &self,
        tenant_id: &str,
        command: &CloseGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let room = self.get_room(tenant_id, &command.room_id).await?;
        ensure_expected_version(&room, command.expected_version)?;
        let timestamp = now().to_rfc3339();
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                close_postgres(pool, tenant_id, command, &timestamp).await
            }
            DatabasePool::Sqlite(pool, _) => {
                close_sqlite(pool, tenant_id, command, &timestamp).await
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct RoomRow {
    id: String,
    game_id: String,
    mode_id: Option<String>,
    ruleset_id: Option<String>,
    room_code: String,
    host_user_id: Option<String>,
    visibility: String,
    join_policy: String,
    max_players: i32,
    current_players: i32,
    status: String,
    version: i64,
}

impl RoomRow {
    fn into_item(self) -> GameRoomItem {
        GameRoomItem {
            id: self.id,
            game_id: self.game_id,
            mode_id: self.mode_id,
            ruleset_id: self.ruleset_id,
            room_code: self.room_code,
            host_user_id: self.host_user_id.unwrap_or_default(),
            visibility: self.visibility,
            join_policy: self.join_policy,
            max_players: self.max_players,
            current_players: self.current_players,
            status: self.status,
            version: self.version,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SeatRow {
    id: String,
    room_id: String,
    seat_no: i32,
    team_no: Option<i32>,
    user_id: Option<String>,
    display_name_snapshot: Option<String>,
    status: String,
    version: i64,
}

impl SeatRow {
    fn into_item(self) -> GameRoomSeatItem {
        GameRoomSeatItem {
            id: self.id,
            room_id: self.room_id,
            seat_no: self.seat_no,
            team_no: self.team_no,
            user_id: self.user_id,
            display_name_snapshot: self.display_name_snapshot,
            status: self.status,
            version: self.version,
        }
    }
}

const ROOM_COLUMNS: &str = "id, game_id, mode_id, ruleset_id, room_code, host_user_id, \
visibility, join_policy, max_players, current_players, status, version";
const SEAT_COLUMNS: &str =
    "id, room_id, seat_no, team_no, user_id, display_name_snapshot, status, version";

async fn list_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    game_id: Option<&str>,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> GameRoomResult<GameRoomPage> {
    let rows = sqlx::query_as::<_, RoomRow>(&format!(
        "SELECT {ROOM_COLUMNS} FROM game_room \
         WHERE tenant_id = $1 AND deleted_at IS NULL \
         AND ($2::text IS NULL OR game_id = $2) \
         AND ($3::text IS NULL OR status = $3) \
         ORDER BY updated_at DESC LIMIT $4 OFFSET $5",
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
        "SELECT COUNT(*) FROM game_room \
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

    Ok(page_from_rows(rows, total, limit, offset))
}

async fn list_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    game_id: Option<&str>,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> GameRoomResult<GameRoomPage> {
    let rows = sqlx::query_as::<_, RoomRow>(&format!(
        "SELECT {ROOM_COLUMNS} FROM game_room \
         WHERE tenant_id = ?1 AND deleted_at IS NULL \
         AND (?2 IS NULL OR game_id = ?2) \
         AND (?3 IS NULL OR status = ?3) \
         ORDER BY updated_at DESC LIMIT ?4 OFFSET ?5",
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
        "SELECT COUNT(*) FROM game_room \
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

    Ok(page_from_rows(rows, total, limit, offset))
}

fn page_from_rows(rows: Vec<RoomRow>, total: i64, limit: i64, offset: i64) -> GameRoomPage {
    GameRoomPage {
        items: rows.into_iter().map(RoomRow::into_item).collect(),
        total: total as u64,
        page: ((offset / limit) + 1) as u32,
        page_size: limit as u32,
    }
}

async fn get_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    room_id: &str,
) -> GameRoomResult<GameRoomItem> {
    let row = sqlx::query_as::<_, RoomRow>(&format!(
        "SELECT {ROOM_COLUMNS} FROM game_room \
         WHERE tenant_id = $1 AND deleted_at IS NULL AND (id = $2 OR room_code = $2) LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(room_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameRoomError::not_found("room not found"))?;

    Ok(row.into_item())
}

async fn get_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    room_id: &str,
) -> GameRoomResult<GameRoomItem> {
    let row = sqlx::query_as::<_, RoomRow>(&format!(
        "SELECT {ROOM_COLUMNS} FROM game_room \
         WHERE tenant_id = ?1 AND deleted_at IS NULL AND (id = ?2 OR room_code = ?2) LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(room_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameRoomError::not_found("room not found"))?;

    Ok(row.into_item())
}

async fn list_seats_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    room_id: &str,
) -> GameRoomResult<Vec<GameRoomSeatItem>> {
    let rows = sqlx::query_as::<_, SeatRow>(&format!(
        "SELECT {SEAT_COLUMNS} FROM game_room_seat \
         WHERE tenant_id = $1 AND room_id = $2 ORDER BY seat_no ASC",
    ))
    .bind(tenant_id)
    .bind(room_id)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    Ok(rows.into_iter().map(SeatRow::into_item).collect())
}

async fn list_seats_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    room_id: &str,
) -> GameRoomResult<Vec<GameRoomSeatItem>> {
    let rows = sqlx::query_as::<_, SeatRow>(&format!(
        "SELECT {SEAT_COLUMNS} FROM game_room_seat \
         WHERE tenant_id = ?1 AND room_id = ?2 ORDER BY seat_no ASC",
    ))
    .bind(tenant_id)
    .bind(room_id)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;
    Ok(rows.into_iter().map(SeatRow::into_item).collect())
}

async fn create_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    id: &str,
    timestamp: &str,
    command: &CreateGameRoomCommand,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    let row = sqlx::query_as::<_, RoomRow>(&format!(
        "INSERT INTO game_room \
         (id, uuid, tenant_id, organization_id, game_id, mode_id, ruleset_id, room_code, \
          host_user_id, visibility, join_policy, max_players, current_players, status, \
          opened_at, created_at, created_by, updated_at, updated_by) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, $9, $10, $11, 1, 'open', \
          $12, $12, $8, $12, $8) RETURNING {ROOM_COLUMNS}",
    ))
    .bind(id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.ruleset_id)
    .bind(&command.room_code)
    .bind(&command.host_user_id)
    .bind(&command.visibility)
    .bind(&command.join_policy)
    .bind(command.max_players)
    .bind(timestamp)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;

    insert_host_seat_postgres(&mut tx, tenant_id, id, &command.host_user_id, timestamp).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    Ok(row.into_item())
}

async fn create_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    id: &str,
    timestamp: &str,
    command: &CreateGameRoomCommand,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    sqlx::query(
        "INSERT INTO game_room \
         (id, uuid, tenant_id, organization_id, game_id, mode_id, ruleset_id, room_code, \
          host_user_id, visibility, join_policy, max_players, current_players, status, \
          opened_at, created_at, created_by, updated_at, updated_by) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1, 'open', \
          ?12, ?12, ?8, ?12, ?8)",
    )
    .bind(id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.ruleset_id)
    .bind(&command.room_code)
    .bind(&command.host_user_id)
    .bind(&command.visibility)
    .bind(&command.join_policy)
    .bind(command.max_players)
    .bind(timestamp)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;

    insert_host_seat_sqlite(&mut tx, tenant_id, id, &command.host_user_id, timestamp).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_sqlite(pool, tenant_id, id).await
}

async fn insert_host_seat_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    room_id: &str,
    host_user_id: &str,
    timestamp: &str,
) -> GameRoomResult<()> {
    sqlx::query(
        "INSERT INTO game_room_seat \
         (id, uuid, tenant_id, organization_id, room_id, seat_no, user_id, status, joined_at, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, 1, $5, 'joined', $6, $6, $6)",
    )
    .bind(uuid())
    .bind(uuid())
    .bind(tenant_id)
    .bind(room_id)
    .bind(host_user_id)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn insert_host_seat_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    room_id: &str,
    host_user_id: &str,
    timestamp: &str,
) -> GameRoomResult<()> {
    sqlx::query(
        "INSERT INTO game_room_seat \
         (id, uuid, tenant_id, organization_id, room_id, seat_no, user_id, status, joined_at, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, 1, ?5, 'joined', ?6, ?6, ?6)",
    )
    .bind(uuid())
    .bind(uuid())
    .bind(tenant_id)
    .bind(room_id)
    .bind(host_user_id)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn join_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &JoinGameRoomCommand,
    seat_no: i32,
    reusable: bool,
    current_players: i32,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    update_room_players_postgres(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        current_players,
        timestamp,
    )
    .await?;
    upsert_join_seat_postgres(&mut tx, tenant_id, command, seat_no, reusable, timestamp).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_postgres(pool, tenant_id, &command.room_id).await
}

async fn join_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &JoinGameRoomCommand,
    seat_no: i32,
    reusable: bool,
    current_players: i32,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    update_room_players_sqlite(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        current_players,
        timestamp,
    )
    .await?;
    upsert_join_seat_sqlite(&mut tx, tenant_id, command, seat_no, reusable, timestamp).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_sqlite(pool, tenant_id, &command.room_id).await
}

async fn upsert_join_seat_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    command: &JoinGameRoomCommand,
    seat_no: i32,
    reusable: bool,
    timestamp: &str,
) -> GameRoomResult<()> {
    if reusable {
        sqlx::query(
            "UPDATE game_room_seat SET user_id = $4, display_name_snapshot = $5, status = 'joined', \
             joined_at = $6, ready_at = NULL, left_at = NULL, updated_at = $6, version = version + 1 \
             WHERE tenant_id = $1 AND room_id = $2 AND seat_no = $3",
        )
        .bind(tenant_id)
        .bind(&command.room_id)
        .bind(seat_no)
        .bind(&command.user_id)
        .bind(&command.display_name_snapshot)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
    } else {
        sqlx::query(
            "INSERT INTO game_room_seat \
             (id, uuid, tenant_id, organization_id, room_id, seat_no, user_id, display_name_snapshot, \
              status, joined_at, created_at, updated_at) \
             VALUES ($1, $2, $3, '0', $4, $5, $6, $7, 'joined', $8, $8, $8)",
        )
        .bind(uuid())
        .bind(uuid())
        .bind(tenant_id)
        .bind(&command.room_id)
        .bind(seat_no)
        .bind(&command.user_id)
        .bind(&command.display_name_snapshot)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
    }
    Ok(())
}

async fn upsert_join_seat_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    command: &JoinGameRoomCommand,
    seat_no: i32,
    reusable: bool,
    timestamp: &str,
) -> GameRoomResult<()> {
    if reusable {
        sqlx::query(
            "UPDATE game_room_seat SET user_id = ?4, display_name_snapshot = ?5, status = 'joined', \
             joined_at = ?6, ready_at = NULL, left_at = NULL, updated_at = ?6, version = version + 1 \
             WHERE tenant_id = ?1 AND room_id = ?2 AND seat_no = ?3",
        )
        .bind(tenant_id)
        .bind(&command.room_id)
        .bind(seat_no)
        .bind(&command.user_id)
        .bind(&command.display_name_snapshot)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
    } else {
        sqlx::query(
            "INSERT INTO game_room_seat \
             (id, uuid, tenant_id, organization_id, room_id, seat_no, user_id, display_name_snapshot, \
              status, joined_at, created_at, updated_at) \
             VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, 'joined', ?8, ?8, ?8)",
        )
        .bind(uuid())
        .bind(uuid())
        .bind(tenant_id)
        .bind(&command.room_id)
        .bind(seat_no)
        .bind(&command.user_id)
        .bind(&command.display_name_snapshot)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
    }
    Ok(())
}

async fn leave_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &LeaveGameRoomCommand,
    current_players: i32,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    update_room_players_postgres(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        current_players,
        timestamp,
    )
    .await?;
    let result = sqlx::query(
        "UPDATE game_room_seat SET status = 'left', left_at = $4, updated_at = $4, version = version + 1 \
         WHERE tenant_id = $1 AND room_id = $2 AND user_id = $3 \
         AND status IN ('reserved', 'joined', 'ready', 'playing')",
    )
    .bind(tenant_id)
    .bind(&command.room_id)
    .bind(&command.user_id)
    .bind(timestamp)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    ensure_rows_affected(result.rows_affected(), "active room seat not found")?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_postgres(pool, tenant_id, &command.room_id).await
}

async fn leave_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &LeaveGameRoomCommand,
    current_players: i32,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    update_room_players_sqlite(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        current_players,
        timestamp,
    )
    .await?;
    let result = sqlx::query(
        "UPDATE game_room_seat SET status = 'left', left_at = ?4, updated_at = ?4, version = version + 1 \
         WHERE tenant_id = ?1 AND room_id = ?2 AND user_id = ?3 \
         AND status IN ('reserved', 'joined', 'ready', 'playing')",
    )
    .bind(tenant_id)
    .bind(&command.room_id)
    .bind(&command.user_id)
    .bind(timestamp)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    ensure_rows_affected(result.rows_affected(), "active room seat not found")?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_sqlite(pool, tenant_id, &command.room_id).await
}

async fn ready_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &ReadyGameRoomCommand,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    touch_room_postgres(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        timestamp,
    )
    .await?;
    let status = if command.ready { "ready" } else { "joined" };
    let ready_at = if command.ready { Some(timestamp) } else { None };
    let result = sqlx::query(
        "UPDATE game_room_seat SET status = $4, ready_at = $5, updated_at = $6, version = version + 1 \
         WHERE tenant_id = $1 AND room_id = $2 AND user_id = $3 \
         AND status IN ('reserved', 'joined', 'ready')",
    )
    .bind(tenant_id)
    .bind(&command.room_id)
    .bind(&command.user_id)
    .bind(status)
    .bind(ready_at)
    .bind(timestamp)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    ensure_rows_affected(result.rows_affected(), "active room seat not found")?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_postgres(pool, tenant_id, &command.room_id).await
}

async fn ready_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &ReadyGameRoomCommand,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    touch_room_sqlite(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        timestamp,
    )
    .await?;
    let status = if command.ready { "ready" } else { "joined" };
    let ready_at = if command.ready { Some(timestamp) } else { None };
    let result = sqlx::query(
        "UPDATE game_room_seat SET status = ?4, ready_at = ?5, updated_at = ?6, version = version + 1 \
         WHERE tenant_id = ?1 AND room_id = ?2 AND user_id = ?3 \
         AND status IN ('reserved', 'joined', 'ready')",
    )
    .bind(tenant_id)
    .bind(&command.room_id)
    .bind(&command.user_id)
    .bind(status)
    .bind(ready_at)
    .bind(timestamp)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    ensure_rows_affected(result.rows_affected(), "active room seat not found")?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_sqlite(pool, tenant_id, &command.room_id).await
}

async fn start_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &StartGameRoomCommand,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    update_room_status_postgres(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        "in_progress",
        Some("started_at"),
        timestamp,
    )
    .await?;
    sqlx::query(
        "UPDATE game_room_seat SET status = 'playing', updated_at = $3, version = version + 1 \
         WHERE tenant_id = $1 AND room_id = $2 AND status IN ('reserved', 'joined', 'ready')",
    )
    .bind(tenant_id)
    .bind(&command.room_id)
    .bind(timestamp)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_postgres(pool, tenant_id, &command.room_id).await
}

async fn start_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &StartGameRoomCommand,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    update_room_status_sqlite(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        "in_progress",
        Some("started_at"),
        timestamp,
    )
    .await?;
    sqlx::query(
        "UPDATE game_room_seat SET status = 'playing', updated_at = ?3, version = version + 1 \
         WHERE tenant_id = ?1 AND room_id = ?2 AND status IN ('reserved', 'joined', 'ready')",
    )
    .bind(tenant_id)
    .bind(&command.room_id)
    .bind(timestamp)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_sqlite(pool, tenant_id, &command.room_id).await
}

async fn close_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    command: &CloseGameRoomCommand,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    update_room_status_postgres(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        "closed",
        Some("closed_at"),
        timestamp,
    )
    .await?;
    sqlx::query(
        "UPDATE game_room SET current_players = 0, updated_by = $3 WHERE tenant_id = $1 AND id = $2",
    )
    .bind(tenant_id)
    .bind(&command.room_id)
    .bind(&command.operator_user_id)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    close_active_seats_postgres(&mut tx, tenant_id, &command.room_id, timestamp).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_postgres(pool, tenant_id, &command.room_id).await
}

async fn close_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    command: &CloseGameRoomCommand,
    timestamp: &str,
) -> GameRoomResult<GameRoomItem> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;
    update_room_status_sqlite(
        &mut tx,
        tenant_id,
        &command.room_id,
        command.expected_version,
        "closed",
        Some("closed_at"),
        timestamp,
    )
    .await?;
    sqlx::query(
        "UPDATE game_room SET current_players = 0, updated_by = ?3 WHERE tenant_id = ?1 AND id = ?2",
    )
    .bind(tenant_id)
    .bind(&command.room_id)
    .bind(&command.operator_user_id)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx_error)?;
    close_active_seats_sqlite(&mut tx, tenant_id, &command.room_id, timestamp).await?;
    tx.commit().await.map_err(map_sqlx_error)?;
    get_sqlite(pool, tenant_id, &command.room_id).await
}

async fn update_room_players_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    room_id: &str,
    expected_version: Option<i64>,
    current_players: i32,
    timestamp: &str,
) -> GameRoomResult<()> {
    let result = if let Some(expected_version) = expected_version {
        sqlx::query(
            "UPDATE game_room SET current_players = $4, updated_at = $5, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(room_id)
        .bind(expected_version)
        .bind(current_players)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    } else {
        sqlx::query(
            "UPDATE game_room SET current_players = $3, updated_at = $4, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(room_id)
        .bind(current_players)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_version_update(result.rows_affected())
}

async fn update_room_players_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    room_id: &str,
    expected_version: Option<i64>,
    current_players: i32,
    timestamp: &str,
) -> GameRoomResult<()> {
    let result = if let Some(expected_version) = expected_version {
        sqlx::query(
            "UPDATE game_room SET current_players = ?4, updated_at = ?5, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND version = ?3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(room_id)
        .bind(expected_version)
        .bind(current_players)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    } else {
        sqlx::query(
            "UPDATE game_room SET current_players = ?3, updated_at = ?4, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(room_id)
        .bind(current_players)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_version_update(result.rows_affected())
}

async fn touch_room_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    room_id: &str,
    expected_version: Option<i64>,
    timestamp: &str,
) -> GameRoomResult<()> {
    let result = if let Some(expected_version) = expected_version {
        sqlx::query(
            "UPDATE game_room SET updated_at = $4, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(room_id)
        .bind(expected_version)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    } else {
        sqlx::query(
            "UPDATE game_room SET updated_at = $3, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(room_id)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_version_update(result.rows_affected())
}

async fn touch_room_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    room_id: &str,
    expected_version: Option<i64>,
    timestamp: &str,
) -> GameRoomResult<()> {
    let result = if let Some(expected_version) = expected_version {
        sqlx::query(
            "UPDATE game_room SET updated_at = ?4, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND version = ?3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(room_id)
        .bind(expected_version)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    } else {
        sqlx::query(
            "UPDATE game_room SET updated_at = ?3, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(room_id)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_version_update(result.rows_affected())
}

async fn update_room_status_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    room_id: &str,
    expected_version: Option<i64>,
    status: &str,
    timestamp_column: Option<&str>,
    timestamp: &str,
) -> GameRoomResult<()> {
    let result = if let Some(expected_version) = expected_version {
        let timestamp_assignment = timestamp_column
            .map(|column| format!(", {column} = $5"))
            .unwrap_or_default();
        sqlx::query(&format!(
            "UPDATE game_room SET status = $4{timestamp_assignment}, updated_at = $5, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $3 AND deleted_at IS NULL",
        ))
        .bind(tenant_id)
        .bind(room_id)
        .bind(expected_version)
        .bind(status)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    } else {
        let timestamp_assignment = timestamp_column
            .map(|column| format!(", {column} = $4"))
            .unwrap_or_default();
        sqlx::query(&format!(
            "UPDATE game_room SET status = $3{timestamp_assignment}, updated_at = $4, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        ))
        .bind(tenant_id)
        .bind(room_id)
        .bind(status)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_version_update(result.rows_affected())
}

async fn update_room_status_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    room_id: &str,
    expected_version: Option<i64>,
    status: &str,
    timestamp_column: Option<&str>,
    timestamp: &str,
) -> GameRoomResult<()> {
    let result = if let Some(expected_version) = expected_version {
        let timestamp_assignment = timestamp_column
            .map(|column| format!(", {column} = ?5"))
            .unwrap_or_default();
        sqlx::query(&format!(
            "UPDATE game_room SET status = ?4{timestamp_assignment}, updated_at = ?5, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND version = ?3 AND deleted_at IS NULL",
        ))
        .bind(tenant_id)
        .bind(room_id)
        .bind(expected_version)
        .bind(status)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    } else {
        let timestamp_assignment = timestamp_column
            .map(|column| format!(", {column} = ?4"))
            .unwrap_or_default();
        sqlx::query(&format!(
            "UPDATE game_room SET status = ?3{timestamp_assignment}, updated_at = ?4, version = version + 1 \
             WHERE tenant_id = ?1 AND id = ?2 AND deleted_at IS NULL",
        ))
        .bind(tenant_id)
        .bind(room_id)
        .bind(status)
        .bind(timestamp)
        .execute(&mut **tx)
        .await
    }
    .map_err(map_sqlx_error)?;
    ensure_version_update(result.rows_affected())
}

async fn close_active_seats_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    room_id: &str,
    timestamp: &str,
) -> GameRoomResult<()> {
    sqlx::query(
        "UPDATE game_room_seat SET status = 'left', left_at = $3, updated_at = $3, version = version + 1 \
         WHERE tenant_id = $1 AND room_id = $2 AND status IN ('reserved', 'joined', 'ready', 'playing')",
    )
    .bind(tenant_id)
    .bind(room_id)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn close_active_seats_sqlite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tenant_id: &str,
    room_id: &str,
    timestamp: &str,
) -> GameRoomResult<()> {
    sqlx::query(
        "UPDATE game_room_seat SET status = 'left', left_at = ?3, updated_at = ?3, version = version + 1 \
         WHERE tenant_id = ?1 AND room_id = ?2 AND status IN ('reserved', 'joined', 'ready', 'playing')",
    )
    .bind(tenant_id)
    .bind(room_id)
    .bind(timestamp)
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

fn ensure_expected_version(room: &GameRoomItem, expected: Option<i64>) -> GameRoomResult<()> {
    if let Some(expected) = expected {
        if room.version != expected {
            return Err(GameRoomError::conflict("room version has changed"));
        }
    }
    Ok(())
}

fn ensure_version_update(rows_affected: u64) -> GameRoomResult<()> {
    if rows_affected == 0 {
        return Err(GameRoomError::conflict("room version has changed"));
    }
    Ok(())
}

fn ensure_rows_affected(rows_affected: u64, message: &str) -> GameRoomResult<()> {
    if rows_affected == 0 {
        return Err(GameRoomError::not_found(message));
    }
    Ok(())
}

fn next_available_seat_no(room: &GameRoomItem, seats: &[GameRoomSeatItem]) -> Option<i32> {
    (1..=room.max_players).find(|seat_no| {
        seats
            .iter()
            .all(|seat| seat.seat_no != *seat_no || !is_active_seat_status(&seat.status))
    })
}

fn active_seat_count(seats: &[GameRoomSeatItem]) -> i32 {
    seats
        .iter()
        .filter(|seat| is_active_seat_status(&seat.status))
        .count() as i32
}

fn is_active_seat_status(status: &str) -> bool {
    matches!(status, "reserved" | "joined" | "ready" | "playing")
}

fn map_sqlx_error(error: sqlx::Error) -> GameRoomError {
    GameRoomError::invalid(error.to_string())
}
