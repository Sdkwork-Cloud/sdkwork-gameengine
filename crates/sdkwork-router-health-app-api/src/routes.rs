use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_web_core::WebRequestContext;
use serde_json::json;

pub fn build_health_router() -> Router {
    Router::new()
        .route("/app/v3/api/system/health", get(health_check))
        .route("/app/v3/api/system/ready", get(ready_check))
}

async fn health_check(_ctx: WebRequestContext) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "sdkwork-games"
        })),
    )
        .into_response()
}

async fn ready_check(_ctx: WebRequestContext) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ready",
            "service": "sdkwork-games"
        })),
    )
        .into_response()
}
