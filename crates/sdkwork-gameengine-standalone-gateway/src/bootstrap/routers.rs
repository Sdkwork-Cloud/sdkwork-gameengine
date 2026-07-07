use axum::Router;
use tower_http::cors::CorsLayer;

use crate::bootstrap::catalog::SharedCatalogService;
use crate::bootstrap::leaderboard::SharedLeaderboardService;
use crate::bootstrap::room::SharedRoomService;
use crate::web_bootstrap::{with_games_app_request_context, with_games_backend_request_context};
use sdkwork_routes_catalog_app_api::build_catalog_app_router;
use sdkwork_routes_catalog_backend_api::build_catalog_backend_router;
use sdkwork_routes_health_app_api::build_health_router;
use sdkwork_routes_leaderboard_app_api::build_leaderboard_app_router;
use sdkwork_routes_room_app_api::build_room_app_router;
use sdkwork_routes_room_backend_api::build_room_backend_router;

pub async fn build_router(
    catalog_service: SharedCatalogService,
    leaderboard_service: SharedLeaderboardService,
    room_service: SharedRoomService,
) -> Router {
    let app_routes = Router::new()
        .merge(with_games_app_request_context(build_health_router()))
        .merge(with_games_app_request_context(build_catalog_app_router(
            catalog_service.clone(),
        )))
        .merge(with_games_app_request_context(
            build_leaderboard_app_router(leaderboard_service),
        ))
        .merge(with_games_app_request_context(build_room_app_router(
            room_service.clone(),
        )));

    let backend_routes =
        with_games_backend_request_context(build_catalog_backend_router(catalog_service)).merge(
            with_games_backend_request_context(build_room_backend_router(room_service)),
        );

    Router::new()
        .merge(app_routes)
        .merge(backend_routes)
        .layer(CorsLayer::permissive())
}
