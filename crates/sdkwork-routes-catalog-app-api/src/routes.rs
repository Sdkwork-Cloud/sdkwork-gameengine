use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_game_catalog_service::{GameCatalogQuery, GameCatalogRepository, GameCatalogService};
use sdkwork_routes_games_support::{
    catalog_page_to_list_data, finish_page_response, finish_resource_response,
};
use sdkwork_web_axum::RequirePrincipal;
use std::sync::Arc;

pub type GamesCatalogStore<R> = Arc<GameCatalogService<R>>;

#[derive(Debug, serde::Deserialize, Default)]
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
    Query(query): Query<GamesListQuery>,
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
