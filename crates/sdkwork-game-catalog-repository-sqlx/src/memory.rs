use async_trait::async_trait;
use sdkwork_game_catalog_service::{
    GameCatalogItem, GameCatalogPage, GameCatalogQuery, GameCatalogRepository, GameError,
    GameResult,
};

#[derive(Clone, Default)]
pub struct InMemoryGameCatalogRepository {
    items: Vec<GameCatalogItem>,
}

impl InMemoryGameCatalogRepository {
    pub fn with_seed(items: Vec<GameCatalogItem>) -> Self {
        Self { items }
    }
}

#[async_trait]
impl GameCatalogRepository for InMemoryGameCatalogRepository {
    async fn list_catalog(
        &self,
        _tenant_id: &str,
        query: &GameCatalogQuery,
    ) -> GameResult<GameCatalogPage> {
        let filtered: Vec<GameCatalogItem> = if let Some(status) = &query.status {
            self.items
                .iter()
                .filter(|item| item.status == *status)
                .cloned()
                .collect()
        } else {
            self.items.clone()
        };

        let total = filtered.len() as u64;
        let offset = query.offset() as usize;
        let limit = query.limit() as usize;
        let page_items = filtered.into_iter().skip(offset).take(limit).collect();

        Ok(GameCatalogPage {
            items: page_items,
            total,
            page: query.page.unwrap_or(1),
            page_size: query.limit(),
        })
    }

    async fn get_catalog_item(
        &self,
        _tenant_id: &str,
        game_id: &str,
    ) -> GameResult<GameCatalogItem> {
        self.items
            .iter()
            .find(|item| item.id == game_id)
            .cloned()
            .ok_or_else(|| GameError::not_found(format!("game {game_id} not found")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_game_catalog_service::GameCatalogQuery;

    #[tokio::test]
    async fn list_catalog_paginates_items() {
        let repo = InMemoryGameCatalogRepository::with_seed(vec![GameCatalogItem {
            id: "g1".into(),
            game_code: "demo".into(),
            title: "Demo Game".into(),
            summary: None,
            genre: Some("puzzle".into()),
            status: "published".into(),
        }]);

        let page = repo
            .list_catalog("tenant-1", &GameCatalogQuery::default())
            .await
            .expect("page");

        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].title, "Demo Game");
    }
}
