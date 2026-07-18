pub mod bootstrap;

pub use bootstrap::{
    build_catalog_service, build_gateway_services, build_leaderboard_service, build_room_service,
    build_router, build_router_from_business, GatewayServices,
};
pub use sdkwork_gameengine_gateway_assembly::{
    games_public_path_prefixes, with_games_app_request_context, with_games_backend_request_context,
    GAMES_APP_HTTP_ROUTES, GAMES_BACKEND_HTTP_ROUTES,
};
