use std::sync::Arc;

use sdkwork_game_catalog_repository_sqlx::{GameCatalogRepositoryKind, SqlxGameCatalogRepository};
use sdkwork_game_catalog_service::GameCatalogService;
use sdkwork_games_database_host::bootstrap_games_database_from_env;

pub type SharedCatalogService = Arc<GameCatalogService<GameCatalogRepositoryKind>>;

pub async fn build_catalog_service() -> Result<SharedCatalogService, String> {
    let host = bootstrap_games_database_from_env().await?;
    let repository = GameCatalogRepositoryKind::Sqlx(Box::new(SqlxGameCatalogRepository::new(
        host.pool().clone(),
    )));
    Ok(Arc::new(GameCatalogService::new(repository)))
}
