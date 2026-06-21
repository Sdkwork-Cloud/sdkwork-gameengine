use async_trait::async_trait;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_catalog_service::{
    GameCatalogItem, GameCatalogPage, GameCatalogQuery, GameCatalogRepository, GameError,
    GameResult,
};
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone)]
pub struct SqlxGameCatalogRepository {
    pool: DatabasePool,
}

impl SqlxGameCatalogRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameCatalogRepository for SqlxGameCatalogRepository {
    async fn list_catalog(
        &self,
        tenant_id: &str,
        query: &GameCatalogQuery,
    ) -> GameResult<GameCatalogPage> {
        if is_blank(Some(tenant_id)) {
            return Err(GameError::invalid("tenant_id is required"));
        }

        let limit = query.limit() as i64;
        let offset = query.offset() as i64;
        let status = query.status.as_deref();

        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                list_postgres(pool, tenant_id, status, limit, offset).await
            }
            DatabasePool::Sqlite(pool, _) => {
                list_sqlite(pool, tenant_id, status, limit, offset).await
            }
        }
    }

    async fn get_catalog_item(
        &self,
        tenant_id: &str,
        game_id: &str,
    ) -> GameResult<GameCatalogItem> {
        if is_blank(Some(tenant_id)) {
            return Err(GameError::invalid("tenant_id is required"));
        }
        if is_blank(Some(game_id)) {
            return Err(GameError::invalid("game_id is required"));
        }

        match &self.pool {
            DatabasePool::Postgres(pool, _) => get_postgres(pool, tenant_id, game_id).await,
            DatabasePool::Sqlite(pool, _) => get_sqlite(pool, tenant_id, game_id).await,
        }
    }
}

async fn list_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> GameResult<GameCatalogPage> {
    let rows = if let Some(status) = status {
        sqlx::query_as::<_, CatalogRow>(
            "SELECT id, game_code, title, summary, genre, status FROM game_catalog \
             WHERE tenant_id = $1 AND deleted_at IS NULL AND status = $2 \
             ORDER BY sort_order ASC, title ASC LIMIT $3 OFFSET $4",
        )
        .bind(tenant_id)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, CatalogRow>(
            "SELECT id, game_code, title, summary, genre, status FROM game_catalog \
             WHERE tenant_id = $1 AND deleted_at IS NULL \
             ORDER BY sort_order ASC, title ASC LIMIT $2 OFFSET $3",
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
    .map_err(map_sqlx_error)?;

    let total: i64 = if let Some(status) = status {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM game_catalog WHERE tenant_id = $1 AND deleted_at IS NULL AND status = $2",
        )
        .bind(tenant_id)
        .bind(status)
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM game_catalog WHERE tenant_id = $1 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
    }
    .map_err(map_sqlx_error)?;

    Ok(GameCatalogPage {
        items: rows.into_iter().map(CatalogRow::into_item).collect(),
        total: total as u64,
        page: ((offset / limit) + 1) as u32,
        page_size: limit as u32,
    })
}

async fn get_postgres(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    game_id: &str,
) -> GameResult<GameCatalogItem> {
    let row = sqlx::query_as::<_, CatalogRow>(
        "SELECT id, game_code, title, summary, genre, status FROM game_catalog \
         WHERE tenant_id = $1 AND deleted_at IS NULL AND (id = $2 OR game_code = $2) LIMIT 1",
    )
    .bind(tenant_id)
    .bind(game_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameError::not_found(format!("game {game_id} not found")))?;

    Ok(row.into_item())
}

async fn list_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> GameResult<GameCatalogPage> {
    let rows = if let Some(status) = status {
        sqlx::query_as::<_, CatalogRow>(
            "SELECT id, game_code, title, summary, genre, status FROM game_catalog \
             WHERE tenant_id = ? AND deleted_at IS NULL AND status = ? \
             ORDER BY sort_order ASC, title ASC LIMIT ? OFFSET ?",
        )
        .bind(tenant_id)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, CatalogRow>(
            "SELECT id, game_code, title, summary, genre, status FROM game_catalog \
             WHERE tenant_id = ? AND deleted_at IS NULL \
             ORDER BY sort_order ASC, title ASC LIMIT ? OFFSET ?",
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
    .map_err(map_sqlx_error)?;

    let total: i64 = if let Some(status) = status {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM game_catalog WHERE tenant_id = ? AND deleted_at IS NULL AND status = ?",
        )
        .bind(tenant_id)
        .bind(status)
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM game_catalog WHERE tenant_id = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
    }
    .map_err(map_sqlx_error)?;

    Ok(GameCatalogPage {
        items: rows.into_iter().map(CatalogRow::into_item).collect(),
        total: total as u64,
        page: ((offset / limit) + 1) as u32,
        page_size: limit as u32,
    })
}

async fn get_sqlite(
    pool: &sqlx::SqlitePool,
    tenant_id: &str,
    game_id: &str,
) -> GameResult<GameCatalogItem> {
    let row = sqlx::query_as::<_, CatalogRow>(
        "SELECT id, game_code, title, summary, genre, status FROM game_catalog \
         WHERE tenant_id = ? AND deleted_at IS NULL AND (id = ? OR game_code = ?) LIMIT 1",
    )
    .bind(tenant_id)
    .bind(game_id)
    .bind(game_id)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| GameError::not_found(format!("game {game_id} not found")))?;

    Ok(row.into_item())
}

#[derive(sqlx::FromRow)]
struct CatalogRow {
    id: String,
    game_code: String,
    title: String,
    summary: Option<String>,
    genre: Option<String>,
    status: String,
}

impl CatalogRow {
    fn into_item(self) -> GameCatalogItem {
        GameCatalogItem {
            id: self.id,
            game_code: self.game_code,
            title: self.title,
            summary: self.summary,
            genre: self.genre,
            status: self.status,
        }
    }
}

fn map_sqlx_error(error: sqlx::Error) -> GameError {
    GameError::invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_row_maps_to_domain_item() {
        let row = CatalogRow {
            id: "g1".into(),
            game_code: "xiangqi".into(),
            title: "Chinese Chess".into(),
            summary: None,
            genre: Some("chess".into()),
            status: "published".into(),
        };
        let item = row.into_item();
        assert_eq!(item.game_code, "xiangqi");
    }
}
