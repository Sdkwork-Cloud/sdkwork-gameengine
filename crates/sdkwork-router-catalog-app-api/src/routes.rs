use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_game_catalog_service::{GameCatalogQuery, GameCatalogRepository, GameCatalogService};
use sdkwork_web_axum::RequirePrincipal;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub type GamesCatalogStore<R> = Arc<GameCatalogService<R>>;

#[derive(Debug, Deserialize, Default)]
pub struct GamesListQuery {
    page: Option<u32>,
    page_size: Option<u32>,
    status: Option<String>,
}

pub fn build_catalog_app_router<R>(store: GamesCatalogStore<R>) -> Router
where
    R: GameCatalogRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/app/v3/api/games", get(list_games::<R>))
        .route("/app/v3/api/games/{gameId}", get(get_game::<R>))
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
    let tenant_id = principal.tenant_id();
    match store.get_game(tenant_id, &game_id).await {
        Ok(item) => success(item),
        Err(error) => not_found(error),
    }
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
    };

    match store.list_games(tenant_id, catalog_query).await {
        Ok(page) => success(page),
        Err(error) => bad_request(error),
    }
}

fn success<T: serde::Serialize>(data: T) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "code": "ok",
            "message": "success",
            "data": data
        })),
    )
        .into_response()
}

fn bad_request(error: sdkwork_game_catalog_service::GameError) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "code": error.code(),
            "message": error.message(),
            "data": null
        })),
    )
        .into_response()
}

fn not_found(error: sdkwork_game_catalog_service::GameError) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "code": error.code(),
            "message": error.message(),
            "data": null
        })),
    )
        .into_response()
}
