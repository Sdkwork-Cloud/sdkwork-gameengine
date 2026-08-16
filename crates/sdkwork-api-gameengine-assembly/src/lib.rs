//! Gateway assembly for sdkwork-gameengine.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM: preserve application-specific IAM and service-host exports.

mod bootstrap;
mod generated;

pub use bootstrap::{
    assemble_api_router, assemble_api_router_runtime, assemble_api_router_with_pool,
    assemble_api_router_with_service_parts, assemble_api_router_with_services,
    assemble_business_routes, ApiAssembly, ApiAssemblyRuntime,
};
pub use sdkwork_gameengine_service_host::{
    build_catalog_service, build_gateway_runtime, build_gateway_runtime_with_pool,
    build_gateway_services, build_leaderboard_service, build_room_service, GatewayRuntime,
    GatewayServices, SharedCatalogService, SharedLeaderboardService, SharedRoomService,
};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
