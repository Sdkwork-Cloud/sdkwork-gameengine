//! sdkwork-routes-gameengine-catalog-backend-api gateway route manifest (materialized from the authored route
//! manifest; all operations use dual-token auth).

use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(HttpMethod::Get, "/backend/v3/api/games", "games", "backend.games.catalog.list"),
];

pub fn gateway_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
