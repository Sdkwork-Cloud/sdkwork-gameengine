use axum::Router;
use sdkwork_iam_web_adapter::{build_web_framework_layer, IamWebRequestContextResolver};
use sdkwork_web_axum::with_web_request_context;
use sdkwork_web_core::HttpRouteManifest;

include!(concat!(env!("OUT_DIR"), "/games_http_routes.rs"));

pub fn games_public_path_prefixes() -> Vec<String> {
    vec![
        "/app/v3/api/games/health".to_owned(),
        "/app/v3/api/games/ready".to_owned(),
    ]
}

fn default_resolver() -> IamWebRequestContextResolver {
    IamWebRequestContextResolver::new(None)
}

fn wrap_router_with_manifest(router: Router, route_manifest: HttpRouteManifest) -> Router {
    with_web_request_context(
        router,
        build_web_framework_layer(
            default_resolver(),
            route_manifest,
            games_public_path_prefixes(),
        ),
    )
}

pub fn with_games_app_request_context(router: Router) -> Router {
    wrap_router_with_manifest(router, HttpRouteManifest::new(GAMES_APP_HTTP_ROUTES))
}

pub fn with_games_backend_request_context(router: Router) -> Router {
    wrap_router_with_manifest(router, HttpRouteManifest::new(GAMES_BACKEND_HTTP_ROUTES))
}
