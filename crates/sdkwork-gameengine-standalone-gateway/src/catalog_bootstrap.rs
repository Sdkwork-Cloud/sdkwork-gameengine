use std::sync::Arc;

use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_catalog_repository_sqlx::GameCatalogRepositoryAdapter;
use sdkwork_game_catalog_service::GameCatalogService;
use sdkwork_routes_catalog_app_api::CatalogAppState;
use sdkwork_routes_catalog_backend_api::CatalogBackendState;

pub fn default_catalog_app_state() -> CatalogAppState {
    Arc::new(GameCatalogService::new(GameCatalogRepositoryAdapter::in_memory()))
}

pub fn default_catalog_backend_state() -> CatalogBackendState {
    default_catalog_app_state()
}

pub fn catalog_app_state_from_pool(pool: DatabasePool) -> CatalogAppState {
    Arc::new(GameCatalogService::new(GameCatalogRepositoryAdapter::from_pool(
        pool.clone(),
    )))
}

pub fn catalog_backend_state_from_pool(pool: DatabasePool) -> CatalogBackendState {
    catalog_app_state_from_pool(pool)
}
