use axum::Router;

use sdkwork_gameengine_gateway_assembly::{
    assemble_application_business_router_with_service_parts, with_games_app_request_context,
    SharedCatalogService, SharedLeaderboardService, SharedRoomService,
};
use sdkwork_routes_health_app_api::build_health_router;

pub async fn build_router(
    catalog_service: SharedCatalogService,
    leaderboard_service: SharedLeaderboardService,
    room_service: SharedRoomService,
) -> Router {
    let business = assemble_application_business_router_with_service_parts(
        catalog_service,
        leaderboard_service,
        room_service,
    )
    .router;
    build_router_from_business(business)
}

pub fn build_router_from_business(business: Router) -> Router {
    Router::new()
        .merge(with_games_app_request_context(build_health_router()))
        .merge(business)
        .layer(sdkwork_web_bootstrap::application_cors_layer_from_env(
            &["SDKWORK_GAMEENGINE_ENVIRONMENT"],
            &[
                "SDKWORK_GAMEENGINE_CORS_ALLOWED_ORIGINS",
                "SDKWORK_CORS_ALLOWED_ORIGINS",
            ],
        ))
}
