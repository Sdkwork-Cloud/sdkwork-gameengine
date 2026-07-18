//! Gateway bootstrap for sdkwork-gameengine.

use axum::Router;
use sdkwork_gameengine_service_host::{
    build_gateway_services, GatewayServices, SharedCatalogService, SharedLeaderboardService,
    SharedRoomService,
};
use sdkwork_routes_catalog_app_api::build_catalog_app_router;
use sdkwork_routes_catalog_backend_api::build_catalog_backend_router;
use sdkwork_routes_leaderboard_app_api::build_leaderboard_app_router;
use sdkwork_routes_room_app_api::build_room_app_router;
use sdkwork_routes_room_backend_api::build_room_backend_router;

use crate::web_bootstrap::{with_games_app_request_context, with_games_backend_request_context};

pub struct ApplicationAssembly {
    pub router: Router,
}

pub async fn assemble_application_business_router() -> Result<ApplicationAssembly, String> {
    let services = build_gateway_services().await?;
    Ok(assemble_application_business_router_with_services(services))
}

pub fn assemble_application_business_router_with_services(
    services: GatewayServices,
) -> ApplicationAssembly {
    let app = Router::new()
        .merge(with_games_app_request_context(build_catalog_app_router(
            services.catalog.clone(),
        )))
        .merge(with_games_app_request_context(
            build_leaderboard_app_router(services.leaderboard),
        ))
        .merge(with_games_app_request_context(build_room_app_router(
            services.room.clone(),
        )));
    let backend = with_games_backend_request_context(
        build_catalog_backend_router(services.catalog)
            .merge(build_room_backend_router(services.room)),
    );
    ApplicationAssembly {
        router: Router::new().merge(app).merge(backend),
    }
}

pub fn assemble_application_business_router_with_service_parts(
    catalog: SharedCatalogService,
    leaderboard: SharedLeaderboardService,
    room: SharedRoomService,
) -> ApplicationAssembly {
    assemble_application_business_router_with_services(GatewayServices {
        catalog,
        leaderboard,
        room,
    })
}

pub async fn assemble_application_router() -> Result<ApplicationAssembly, String> {
    assemble_application_business_router().await
}
