use async_trait::async_trait;
use sdkwork_game_catalog_service::{
    GameCatalogItem, GameCatalogPage, GameCatalogQuery, GameCatalogRepository, GameResult,
};

use crate::memory::InMemoryGameCatalogRepository;
use crate::sqlx::SqlxGameCatalogRepository;

#[derive(Clone)]
pub enum GameCatalogRepositoryKind {
    Memory(InMemoryGameCatalogRepository),
    Sqlx(Box<SqlxGameCatalogRepository>),
}

#[async_trait]
impl GameCatalogRepository for GameCatalogRepositoryKind {
    async fn list_catalog(
        &self,
        tenant_id: &str,
        query: &GameCatalogQuery,
    ) -> GameResult<GameCatalogPage> {
        match self {
            Self::Memory(repo) => repo.list_catalog(tenant_id, query).await,
            Self::Sqlx(repo) => repo.list_catalog(tenant_id, query).await,
        }
    }

    async fn get_catalog_item(
        &self,
        tenant_id: &str,
        game_id: &str,
    ) -> GameResult<GameCatalogItem> {
        match self {
            Self::Memory(repo) => repo.get_catalog_item(tenant_id, game_id).await,
            Self::Sqlx(repo) => repo.get_catalog_item(tenant_id, game_id).await,
        }
    }
}
