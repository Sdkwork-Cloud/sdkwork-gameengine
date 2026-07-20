//! Gateway assembly for sdkwork-gameengine.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM: preserve application-specific IAM and service-host exports.

mod bootstrap;
mod generated;
mod web_bootstrap;

pub use bootstrap::{
    assemble_api_router, assemble_api_router_with_service_parts,
    assemble_api_router_with_services, assemble_api_router,
    ApiAssembly,
};
pub use sdkwork_gameengine_service_host::{
    build_catalog_service, build_gateway_services, build_leaderboard_service, build_room_service,
    GatewayServices, SharedCatalogService, SharedLeaderboardService, SharedRoomService,
};
pub use web_bootstrap::{
    games_public_path_prefixes, with_games_app_request_context, with_games_backend_request_context,
    GAMES_APP_HTTP_ROUTES, GAMES_BACKEND_HTTP_ROUTES,
};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
