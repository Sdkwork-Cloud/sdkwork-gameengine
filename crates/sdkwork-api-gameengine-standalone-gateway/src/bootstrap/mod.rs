mod routers;

pub use routers::{build_router, build_router_from_business};
pub use sdkwork_api_gameengine_assembly::{
    build_catalog_service, build_gateway_services, build_leaderboard_service, build_room_service,
    GatewayServices,
};
