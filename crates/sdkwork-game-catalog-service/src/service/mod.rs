use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;
use sdkwork_utils_rust::validated_offset_list_params;

use crate::domain::models::{
    GameCatalogItem, GameCatalogPage, GameCatalogQuery, GameError, GameResult,
};
use crate::ports::repository::GameCatalogRepository;

pub struct GameCatalogService<R: GameCatalogRepository> {
    repository: R,
}

impl<R: GameCatalogRepository> GameCatalogService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list_games(
        &self,
        tenant_id: &str,
        query: GameCatalogQuery,
    ) -> GameResult<GameCatalogPage> {
        if is_blank(Some(tenant_id)) {
            return Err(GameError::invalid("tenant_id is required"));
        }
        validate_pagination(query.page, query.page_size)?;
        self.repository.list_catalog(tenant_id, &query).await
    }

    pub async fn get_game(&self, tenant_id: &str, game_id: &str) -> GameResult<GameCatalogItem> {
        if is_blank(Some(tenant_id)) {
            return Err(GameError::invalid("tenant_id is required"));
        }
        if is_blank(Some(game_id)) {
            return Err(GameError::invalid("game_id is required"));
        }
        self.repository.get_catalog_item(tenant_id, game_id).await
    }

    pub fn new_game_id() -> String {
        uuid()
    }
}

fn validate_pagination(page: Option<u32>, page_size: Option<u32>) -> GameResult<()> {
    validated_offset_list_params(page.map(i64::from), page_size.map(i64::from))
        .map(|_| ())
        .map_err(|_| {
            GameError::invalid_parameter("page and page_size must follow SDKWork pagination bounds")
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::GameCatalogQuery;

    struct EmptyRepo;

    #[async_trait::async_trait]
    impl GameCatalogRepository for EmptyRepo {
        async fn list_catalog(
            &self,
            _tenant_id: &str,
            query: &GameCatalogQuery,
        ) -> GameResult<GameCatalogPage> {
            Ok(GameCatalogPage {
                items: vec![],
                total: 0,
                page: query.page.unwrap_or(1),
                page_size: query.limit(),
            })
        }

        async fn get_catalog_item(
            &self,
            _tenant_id: &str,
            _game_id: &str,
        ) -> GameResult<GameCatalogItem> {
            Err(GameError::not_found("game not found"))
        }
    }

    #[tokio::test]
    async fn list_games_rejects_empty_tenant() {
        let service = GameCatalogService::new(EmptyRepo);
        let result = service.list_games("", GameCatalogQuery::default()).await;
        assert_eq!(result.unwrap_err().code(), "invalid");
    }

    #[tokio::test]
    async fn list_games_rejects_invalid_pagination_before_repository_access() {
        let service = GameCatalogService::new(EmptyRepo);

        let page_size_error = service
            .list_games(
                "100001",
                GameCatalogQuery {
                    page_size: Some(201),
                    ..Default::default()
                },
            )
            .await
            .expect_err("page_size above the SDKWork maximum must fail");
        assert_eq!(page_size_error.code(), "invalid_parameter");
        assert_eq!(
            page_size_error.message(),
            "page and page_size must follow SDKWork pagination bounds"
        );

        let page_error = service
            .list_games(
                "100001",
                GameCatalogQuery {
                    page: Some(0),
                    ..Default::default()
                },
            )
            .await
            .expect_err("page zero must fail");
        assert_eq!(page_error.code(), "invalid_parameter");
        assert_eq!(
            page_error.message(),
            "page and page_size must follow SDKWork pagination bounds"
        );
    }

    #[test]
    fn new_game_id_is_uuid_v4() {
        let id = GameCatalogService::<EmptyRepo>::new_game_id();
        assert_eq!(id.len(), 36);
    }
}
