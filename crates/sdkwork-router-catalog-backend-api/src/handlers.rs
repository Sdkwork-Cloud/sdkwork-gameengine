use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use sdkwork_game_catalog_service::GameCatalogQuery;
use sdkwork_web_core::WebRequestContext;
use serde::Deserialize;

use crate::error::{map_game_error, ok_envelope};
use crate::state::CatalogBackendState;

#[derive(Debug, Deserialize, Default)]
pub struct GamesListQuery {
    page: Option<u32>,
    page_size: Option<u32>,
    status: Option<String>,
}

pub async fn list_games(
    State(store): State<CatalogBackendState>,
    ctx: WebRequestContext,
    Query(query): Query<GamesListQuery>,
) -> Response {
    let tenant_id = match ctx.require_tenant_id() {
        Ok(tenant_id) => tenant_id,
        Err(error) => return sdkwork_web_core::problem::problem_response(&error, ctx.request_id()),
    };

    let catalog_query = GameCatalogQuery {
        page: query.page,
        page_size: query.page_size,
        status: query.status,
    };

    match store.list_games(tenant_id, catalog_query).await {
        Ok(page) => (axum::http::StatusCode::OK, ok_envelope(page)).into_response(),
        Err(error) => {
            let (status, problem) = map_game_error(error);
            (status, problem).into_response()
        }
    }
}
