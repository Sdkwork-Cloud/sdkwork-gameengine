//! sdkwork-routes-leaderboard-app-api gateway route manifest (materialized
//! from the authored route manifest; all operations use dual-token auth).

use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/games/leaderboard", "games", "games.leaderboard.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/games/leaderboard/me", "games", "games.leaderboard.me.retrieve"),
];

pub fn gateway_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
