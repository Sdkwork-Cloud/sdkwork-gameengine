use axum::http::StatusCode;
use axum::Json;
use sdkwork_game_catalog_service::GameError;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct ProblemDetailsPayload {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

pub fn map_game_error(error: GameError) -> (StatusCode, Json<ProblemDetailsPayload>) {
    let (status, problem_type, title, code) = match error.code() {
        "not_found" => (
            StatusCode::NOT_FOUND,
            "https://sdkwork.dev/problems/not-found",
            "Not Found",
            "not_found",
        ),
        "invalid" => (
            StatusCode::BAD_REQUEST,
            "https://sdkwork.dev/problems/bad-request",
            "Bad Request",
            "invalid",
        ),
        other => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "https://sdkwork.dev/problems/internal",
            "Internal Server Error",
            other,
        ),
    };

    (
        status,
        Json(ProblemDetailsPayload {
            problem_type: problem_type.into(),
            title: title.into(),
            status: status.as_u16(),
            detail: error.message().to_string(),
            code: Some(code.into()),
        }),
    )
}

pub fn ok_envelope<T: Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({
        "code": "ok",
        "message": "success",
        "data": data
    }))
}
