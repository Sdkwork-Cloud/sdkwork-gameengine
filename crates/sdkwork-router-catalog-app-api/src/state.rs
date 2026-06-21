use std::sync::Arc;

use sdkwork_game_catalog_repository_sqlx::GameCatalogRepositoryAdapter;
use sdkwork_game_catalog_service::GameCatalogService;

pub type CatalogAppState = Arc<GameCatalogService<GameCatalogRepositoryAdapter>>;
