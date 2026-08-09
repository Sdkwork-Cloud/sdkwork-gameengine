//! sdkwork-routes-room-backend-api gateway route manifest (materialized from the authored route
//! manifest; all operations use dual-token auth).

use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(HttpMethod::Get, "/backend/v3/api/games/rooms", "games", "backend.games.rooms.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/backend/v3/api/games/rooms/{roomId}", "games", "backend.games.rooms.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/backend/v3/api/games/rooms/{roomId}/seats", "games", "backend.games.rooms.seats.list"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/games/rooms/{roomId}/force_close", "games", "backend.games.rooms.forceClose"),
];

pub fn gateway_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
