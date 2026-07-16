//! Gateway bootstrap for sdkwork-gameengine.

use axum::Router;
use sdkwork_gameengine_standalone_gateway::{
    build_gateway_services, with_games_app_request_context, with_games_backend_request_context,
};
use sdkwork_routes_catalog_app_api::build_catalog_app_router;
use sdkwork_routes_catalog_backend_api::build_catalog_backend_router;
use sdkwork_routes_leaderboard_app_api::build_leaderboard_app_router;
use sdkwork_routes_room_app_api::build_room_app_router;
use sdkwork_routes_room_backend_api::build_room_backend_router;

pub struct ApplicationAssembly {
    pub router: Router,
}

pub async fn assemble_application_business_router() -> Result<ApplicationAssembly, String> {
    let services = build_gateway_services().await?;
    let app = Router::new()
        .merge(with_games_app_request_context(build_catalog_app_router(
            services.catalog.clone(),
        )))
        .merge(with_games_app_request_context(build_leaderboard_app_router(
            services.leaderboard,
        )))
        .merge(with_games_app_request_context(build_room_app_router(
            services.room.clone(),
        )));
    let backend = with_games_backend_request_context(
        build_catalog_backend_router(services.catalog)
            .merge(build_room_backend_router(services.room)),
    );
    Ok(ApplicationAssembly {
        router: Router::new().merge(app).merge(backend),
    })
}

pub async fn assemble_application_router() -> Result<ApplicationAssembly, String> {
    assemble_application_business_router().await
}
