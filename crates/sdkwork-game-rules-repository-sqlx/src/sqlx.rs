use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_rules_service::{
    CreateGameRulesetCommand, GameRulesetError, GameRulesetItem, GameRulesetRepository,
    GameRulesetResult,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxGameRulesetRepository {
    pool: DatabasePool,
}

impl SqlxGameRulesetRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameRulesetRepository for SqlxGameRulesetRepository {
    async fn get_active_ruleset(
        &self,
        tenant_id: &str,
        game_id: &str,
        mode_id: Option<&str>,
    ) -> GameRulesetResult<GameRulesetItem> {
        if is_blank(Some(tenant_id)) {
            return Err(GameRulesetError::invalid("tenant_id is required"));
        }
        if is_blank(Some(game_id)) {
            return Err(GameRulesetError::invalid("game_id is required"));
        }

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                get_active_postgres(pool, tenant_id, game_id, mode_id).await
            }
            DatabasePool::Sqlite(pool, _) => {
                get_active_sqlite(pool, tenant_id, game_id, mode_id).await
            }
        }
    }

    async fn create_ruleset(
        &self,
        tenant_id: &str,
        command: &CreateGameRulesetCommand,
    ) -> GameRulesetResult<GameRulesetItem> {
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
}

#[derive(sqlx::FromRow)]
struct RulesetRow {
    id: String,
    game_id: String,
    mode_id: Option<String>,
    ruleset_code: String,
    version_no: i32,
    status: String,
    config_schema: String,
    config_values: String,
    activated_at: Option<String>,
}

impl RulesetRow {
    fn into_item(self) -> GameRulesetResult<GameRulesetItem> {
        Ok(GameRulesetItem {
            id: self.id,
            game_id: self.game_id,
            mode_id: self.mode_id,
            ruleset_code: self.ruleset_code,
            version_no: self.version_no,
            status: self.status,
            config_schema: parse_json(&self.config_schema)?,
            config_values: parse_json(&self.config_values)?,
            activated_at: self.activated_at,
        })
    }
}

const RULESET_COLUMNS: &str = "id, game_id, mode_id, ruleset_code, version_no, status, \
config_schema::text AS config_schema, config_values::text AS config_values, activated_at";

const SQLITE_RULESET_COLUMNS: &str = "id, game_id, mode_id, ruleset_code, version_no, status, \
config_schema, config_values, activated_at";

async fn get_active_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    game_id: &str,
    mode_id: Option<&str>,
) -> GameRulesetResult<GameRulesetItem> {
    let row = sqlx::query_as::<_, RulesetRow>(&format!(
        "SELECT {RULESET_COLUMNS} FROM game_ruleset \
         WHERE tenant_id = $1 AND game_id = $2 AND deleted_at IS NULL AND status = 'active' \
         AND (($3::text IS NULL AND mode_id IS NULL) OR mode_id = $3) \
         ORDER BY version_no DESC LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(game_id)
    .bind(mode_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameRulesetError::not_found("active ruleset not found"))?;

    row.into_item()
}

async fn get_active_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    game_id: &str,
    mode_id: Option<&str>,
) -> GameRulesetResult<GameRulesetItem> {
    let row = sqlx::query_as::<_, RulesetRow>(&format!(
        "SELECT {SQLITE_RULESET_COLUMNS} FROM game_ruleset \
         WHERE tenant_id = ?1 AND game_id = ?2 AND deleted_at IS NULL AND status = 'active' \
         AND ((?3 IS NULL AND mode_id IS NULL) OR mode_id = ?3) \
         ORDER BY version_no DESC LIMIT 1",
    ))
    .bind(tenant_id)
    .bind(game_id)
    .bind(mode_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameRulesetError::not_found("active ruleset not found"))?;

    row.into_item()
}

async fn create_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    id: &str,
    now: &str,
    command: &CreateGameRulesetCommand,
) -> GameRulesetResult<GameRulesetItem> {
    let row = sqlx::query_as::<_, RulesetRow>(&format!(
        "INSERT INTO game_ruleset \
         (id, uuid, tenant_id, organization_id, game_id, mode_id, ruleset_code, version_no, status, \
          config_schema, config_values, activated_at, created_at, updated_at) \
         VALUES ($1, $2, $3, '0', $4, $5, $6, $7, $8, CAST($9 AS jsonb), CAST($10 AS jsonb), \
          CASE WHEN $8 = 'active' THEN $11 ELSE NULL END, $11, $11) \
         RETURNING {RULESET_COLUMNS}",
    ))
    .bind(id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.ruleset_code)
    .bind(command.version_no)
    .bind(&command.status)
    .bind(command.config_schema.to_string())
    .bind(command.config_values.to_string())
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;

    row.into_item()
}

async fn create_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    id: &str,
    now: &str,
    command: &CreateGameRulesetCommand,
) -> GameRulesetResult<GameRulesetItem> {
    sqlx::query(
        "INSERT INTO game_ruleset \
         (id, uuid, tenant_id, organization_id, game_id, mode_id, ruleset_code, version_no, status, \
          config_schema, config_values, activated_at, created_at, updated_at) \
         VALUES (?1, ?2, ?3, '0', ?4, ?5, ?6, ?7, ?8, ?9, ?10, CASE WHEN ?8 = 'active' THEN ?11 ELSE NULL END, ?11, ?11)",
    )
    .bind(id)
    .bind(uuid())
    .bind(tenant_id)
    .bind(&command.game_id)
    .bind(&command.mode_id)
    .bind(&command.ruleset_code)
    .bind(command.version_no)
    .bind(&command.status)
    .bind(command.config_schema.to_string())
    .bind(command.config_values.to_string())
    .bind(now)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;

    let mode_id = command.mode_id.as_deref();
    if command.status == "active" {
        return get_active_sqlite(pool, tenant_id, &command.game_id, mode_id).await;
    }

    let row = sqlx::query_as::<_, RulesetRow>(&format!(
        "SELECT {SQLITE_RULESET_COLUMNS} FROM game_ruleset WHERE tenant_id = ?1 AND id = ?2",
    ))
    .bind(tenant_id)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?;

    row.into_item()
}

fn parse_json(value: &str) -> GameRulesetResult<serde_json::Value> {
    serde_json::from_str(value).map_err(|error| GameRulesetError::invalid(error.to_string()))
}

fn map_sqlx_error(error: sqlx::Error) -> GameRulesetError {
    GameRulesetError::invalid(error.to_string())
}
