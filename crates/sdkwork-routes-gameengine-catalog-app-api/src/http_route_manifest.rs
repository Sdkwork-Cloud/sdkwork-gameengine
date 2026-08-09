//! sdkwork-routes-gameengine-catalog-app-api gateway route manifest (materialized from the authored route
//! manifest; all operations use dual-token auth).

use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/games", "games", "games.catalog.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/games/{gameId}", "games", "games.catalog.retrieve"),
];

pub fn gateway_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
