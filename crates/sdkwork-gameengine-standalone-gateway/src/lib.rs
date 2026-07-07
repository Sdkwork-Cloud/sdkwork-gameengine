pub mod bootstrap;
mod web_bootstrap;

pub use bootstrap::{
    build_catalog_service, build_leaderboard_service, build_room_service, build_router,
};
pub use route_manifest::{GAMES_APP_HTTP_ROUTES, GAMES_BACKEND_HTTP_ROUTES};
pub use web_bootstrap::{
    games_public_path_prefixes, with_games_app_request_context, with_games_backend_request_context,
};

pub mod route_manifest {
    include!(concat!(env!("OUT_DIR"), "/games_http_routes.rs"));
}
