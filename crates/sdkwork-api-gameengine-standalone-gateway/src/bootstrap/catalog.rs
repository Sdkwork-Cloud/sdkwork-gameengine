use std::sync::Arc;

use sdkwork_game_catalog_repository_sqlx::{GameCatalogRepositoryKind, SqlxGameCatalogRepository};
use sdkwork_game_catalog_service::GameCatalogService;
use sdkwork_gameengine_database_host::GamesDatabaseHost;

pub type SharedCatalogService = Arc<GameCatalogService<GameCatalogRepositoryKind>>;

pub fn build_catalog_service(host: &GamesDatabaseHost) -> SharedCatalogService {
    let repository = GameCatalogRepositoryKind::Sqlx(Box::new(SqlxGameCatalogRepository::new(
        host.pool().clone(),
    )));
    Arc::new(GameCatalogService::new(repository))
}
