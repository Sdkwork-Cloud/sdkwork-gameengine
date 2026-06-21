use async_trait::async_trait;

use crate::domain::models::{GameCatalogItem, GameCatalogPage, GameCatalogQuery, GameResult};

#[async_trait]
pub trait GameCatalogRepository: Send + Sync {
    async fn list_catalog(
        &self,
        tenant_id: &str,
        query: &GameCatalogQuery,
    ) -> GameResult<GameCatalogPage>;

    async fn get_catalog_item(&self, tenant_id: &str, game_id: &str)
        -> GameResult<GameCatalogItem>;
}
