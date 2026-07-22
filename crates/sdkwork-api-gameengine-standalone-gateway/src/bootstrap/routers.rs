use axum::Router;

use sdkwork_api_gameengine_assembly::{
    assemble_api_router_with_service_parts, SharedCatalogService, SharedLeaderboardService,
    SharedRoomService,
};
use sdkwork_web_bootstrap::{service_router, ServiceRouterConfig};

pub async fn build_router(
    catalog_service: SharedCatalogService,
    leaderboard_service: SharedLeaderboardService,
    room_service: SharedRoomService,
) -> Router {
    let business =
        assemble_api_router_with_service_parts(catalog_service, leaderboard_service, room_service)
            .router;
    build_router_from_business(business)
}

pub fn build_router_from_business(business: Router) -> Router {
    service_router(business, ServiceRouterConfig::default().with_always_ready()).layer(
        sdkwork_web_bootstrap::application_cors_layer_from_env(
            &["SDKWORK_GAMEENGINE_ENVIRONMENT"],
            &[
                "SDKWORK_GAMEENGINE_CORS_ALLOWED_ORIGINS",
                "SDKWORK_CORS_ALLOWED_ORIGINS",
            ],
        ),
    )
}
