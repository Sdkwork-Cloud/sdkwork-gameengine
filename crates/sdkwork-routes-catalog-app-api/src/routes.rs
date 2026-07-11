use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_game_catalog_service::{GameCatalogQuery, GameCatalogRepository, GameCatalogService};
use sdkwork_routes_games_support::{
    catalog_page_to_list_data, finish_page_response, finish_resource_response, StrictListQuery,
};
use sdkwork_web_axum::RequirePrincipal;
use std::sync::Arc;

pub type GamesCatalogStore<R> = Arc<GameCatalogService<R>>;

#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GamesListQuery {
    page: Option<u32>,
    page_size: Option<u32>,
    status: Option<String>,
    genre: Option<String>,
    q: Option<String>,
    sort: Option<String>,
}

pub fn build_catalog_app_router<R>(store: GamesCatalogStore<R>) -> Router
where
    R: GameCatalogRepository + Send + Sync + 'static,
{
    Router::new()
        .route(crate::paths::GAMES_LIST_PATH, get(list_games::<R>))
        .route(crate::paths::GAME_DETAIL_PATH, get(get_game::<R>))
        .with_state(store)
}

async fn list_games<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesCatalogStore<R>>,
    StrictListQuery(query): StrictListQuery<GamesListQuery>,
) -> Response
where
    R: GameCatalogRepository + Send + Sync,
{
    respond_list(&store, principal.tenant_id(), query).await
}

async fn get_game<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesCatalogStore<R>>,
    Path(game_id): Path<String>,
) -> Response
where
    R: GameCatalogRepository + Send + Sync,
{
    finish_resource_response(store.get_game(principal.tenant_id(), &game_id).await)
}

pub async fn respond_list<R>(
    store: &GameCatalogService<R>,
    tenant_id: &str,
    query: GamesListQuery,
) -> Response
where
    R: GameCatalogRepository + Send + Sync,
{
    let catalog_query = GameCatalogQuery {
        page: query.page,
        page_size: query.page_size,
        status: query.status,
        genre: query.genre,
        q: query.q,
        sort: query.sort,
    };

    finish_page_response(
        store
            .list_games(tenant_id, catalog_query)
            .await
            .map(catalog_page_to_list_data),
    )
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use sdkwork_game_catalog_service::{
        GameCatalogItem, GameCatalogPage, GameCatalogQuery, GameCatalogRepository, GameError,
        GameResult,
    };

    use super::*;

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
    async fn catalog_list_invalid_page_size_returns_invalid_parameter_problem() {
        let store = GameCatalogService::new(EmptyRepo);
        let response = respond_list(
            &store,
            "100001",
            GamesListQuery {
                page_size: Some(201),
                ..Default::default()
            },
        )
        .await;

        assert_eq!(StatusCode::BAD_REQUEST, response.status());
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("problem body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("problem json");
        assert_eq!(40003, payload["code"].as_i64().unwrap());
        assert_eq!(
            "page and page_size must follow SDKWork pagination bounds",
            payload["detail"]
        );
    }

    #[test]
    fn catalog_list_query_rejects_forbidden_pagination_aliases() {
        for alias in [
            "pageSize=20",
            "limit=20",
            "page_no=1",
            "pageNo=1",
            "per_page=20",
            "size=20",
        ] {
            let uri = format!("/app/v3/api/games?{alias}").parse().expect("uri");
            assert!(
                axum::extract::Query::<GamesListQuery>::try_from_uri(&uri).is_err(),
                "forbidden pagination alias must be rejected: {alias}"
            );
        }
    }
}
