use axum::extract::State;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_game_catalog_service::{GameCatalogRepository, GameCatalogService};
use sdkwork_routes_gameengine_catalog_app_api::GamesListQuery;
use sdkwork_routes_games_support::StrictListQuery;
use sdkwork_web_axum::RequirePrincipal;
use std::sync::Arc;

pub type GamesCatalogStore<R> = Arc<GameCatalogService<R>>;

pub fn build_catalog_backend_router<R>(store: GamesCatalogStore<R>) -> Router
where
    R: GameCatalogRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/backend/v3/api/games", get(list_games::<R>))
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
    sdkwork_routes_gameengine_catalog_app_api::respond_list(
        store.as_ref(),
        principal.tenant_id(),
        query,
    )
    .await
}
