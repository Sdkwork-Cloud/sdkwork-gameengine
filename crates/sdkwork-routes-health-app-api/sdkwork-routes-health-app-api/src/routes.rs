use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_routes_games_support::success_resource_response;
use sdkwork_web_core::WebRequestContext;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GamesHealthStatus {
    status: String,
    service: String,
}

pub fn build_health_router() -> Router {
    Router::new()
        .route("/app/v3/api/games/health", get(health_check))
        .route("/app/v3/api/games/ready", get(ready_check))
}

async fn health_check(_ctx: WebRequestContext) -> Response {
    success_resource_response(GamesHealthStatus {
        status: "ok".into(),
        service: "sdkwork-games".into(),
    })
}

async fn ready_check(_ctx: WebRequestContext) -> Response {
    success_resource_response(GamesHealthStatus {
        status: "ready".into(),
        service: "sdkwork-games".into(),
    })
}
