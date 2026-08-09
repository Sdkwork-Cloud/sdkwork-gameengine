//! sdkwork-routes-room-app-api gateway route manifest (materialized from the authored route
//! manifest; all operations use dual-token auth).

use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/games/rooms", "games", "games.rooms.list"),
    HttpRoute::dual_token(HttpMethod::Post, "/app/v3/api/games/rooms", "games", "games.rooms.create"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/games/rooms/{roomId}", "games", "games.rooms.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/games/rooms/{roomId}/seats", "games", "games.rooms.seats.list"),
    HttpRoute::dual_token(HttpMethod::Post, "/app/v3/api/games/rooms/{roomId}/join", "games", "games.rooms.join"),
    HttpRoute::dual_token(HttpMethod::Post, "/app/v3/api/games/rooms/{roomId}/leave", "games", "games.rooms.leave"),
    HttpRoute::dual_token(HttpMethod::Post, "/app/v3/api/games/rooms/{roomId}/ready", "games", "games.rooms.ready"),
    HttpRoute::dual_token(HttpMethod::Post, "/app/v3/api/games/rooms/{roomId}/start", "games", "games.rooms.start"),
    HttpRoute::dual_token(HttpMethod::Post, "/app/v3/api/games/rooms/{roomId}/close", "games", "games.rooms.close"),
];

pub fn gateway_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
