mod http_route_manifest;
mod paths;
mod routes;

pub use http_route_manifest::gateway_route_manifest;
pub use routes::{build_catalog_app_router, respond_list, GamesListQuery};
