use axum::Router;
use tower_http::cors::CorsLayer;

use crate::bootstrap::catalog::SharedCatalogService;
use crate::web_bootstrap::{with_games_app_request_context, with_games_backend_request_context};
use sdkwork_router_catalog_app_api::build_catalog_app_router;
use sdkwork_router_catalog_backend_api::build_catalog_backend_router;
use sdkwork_router_health_app_api::build_health_router;

pub async fn build_router(catalog_service: SharedCatalogService) -> Router {
    let app_routes = Router::new()
        .merge(with_games_app_request_context(build_health_router()))
        .merge(with_games_app_request_context(build_catalog_app_router(
            catalog_service.clone(),
        )));

    let backend_routes =
        with_games_backend_request_context(build_catalog_backend_router(catalog_service));

    Router::new()
        .merge(app_routes)
        .merge(backend_routes)
        .layer(CorsLayer::permissive())
}
