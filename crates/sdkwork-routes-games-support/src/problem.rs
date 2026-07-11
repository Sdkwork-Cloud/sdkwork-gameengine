use axum::{
    http::header,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_game_catalog_service::GameError;
use sdkwork_game_leaderboard_service::LeaderboardError;
use sdkwork_game_room_service::GameRoomError;
use sdkwork_utils_rust::{SdkWorkProblemDetail, SdkWorkProblemRouting, SdkWorkResultCode};
use sdkwork_web_core::{
    new_request_id, problem_response, ProblemCorrelation, WebFrameworkError, WebFrameworkErrorKind,
};

pub type GamesRouteResult<T> = Result<T, GamesApiProblem>;

#[derive(Debug, Clone)]
pub struct GamesApiError {
    status: StatusCode,
    detail: String,
    result_code: Option<SdkWorkResultCode>,
}

impl GamesApiError {
    pub fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
            result_code: None,
        }
    }

    pub fn with_result_code(
        status: StatusCode,
        detail: impl Into<String>,
        result_code: SdkWorkResultCode,
    ) -> Self {
        Self {
            status,
            detail: detail.into(),
            result_code: Some(result_code),
        }
    }

    pub fn invalid_parameter(detail: impl Into<String>) -> Self {
        Self::with_result_code(
            StatusCode::BAD_REQUEST,
            detail,
            SdkWorkResultCode::InvalidParameter,
        )
    }

    fn framework_error(&self) -> WebFrameworkError {
        let kind = match self.status {
            StatusCode::BAD_REQUEST => WebFrameworkErrorKind::BadRequest,
            StatusCode::UNAUTHORIZED => WebFrameworkErrorKind::MissingCredentials,
            StatusCode::FORBIDDEN => WebFrameworkErrorKind::Forbidden,
            StatusCode::NOT_FOUND => WebFrameworkErrorKind::NotFound,
            StatusCode::CONFLICT => WebFrameworkErrorKind::Conflict,
            StatusCode::PAYLOAD_TOO_LARGE => WebFrameworkErrorKind::PayloadTooLarge,
            StatusCode::TOO_MANY_REQUESTS => WebFrameworkErrorKind::RateLimitExceeded,
            StatusCode::SERVICE_UNAVAILABLE => WebFrameworkErrorKind::DependencyUnavailable,
            StatusCode::REQUEST_TIMEOUT => WebFrameworkErrorKind::RequestTimeout,
            StatusCode::METHOD_NOT_ALLOWED => WebFrameworkErrorKind::MethodNotAllowed,
            StatusCode::NOT_IMPLEMENTED => WebFrameworkErrorKind::NotImplemented,
            _ if self.status.is_server_error() => WebFrameworkErrorKind::InternalServerError,
            _ => WebFrameworkErrorKind::BadRequest,
        };
        WebFrameworkError {
            kind,
            message: self.detail.clone(),
            retry_after_seconds: None,
        }
    }
}

impl From<GameError> for GamesApiError {
    fn from(error: GameError) -> Self {
        let status = match error.code() {
            "not_found" => StatusCode::NOT_FOUND,
            "invalid" => StatusCode::BAD_REQUEST,
            "invalid_parameter" => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if error.code() == "invalid_parameter" {
            Self::with_result_code(status, error.message(), SdkWorkResultCode::InvalidParameter)
        } else {
            Self::new(status, error.message())
        }
    }
}

#[derive(Debug, Clone)]
pub struct GamesApiProblem {
    error: GamesApiError,
}

impl GamesApiProblem {
    pub fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            error: GamesApiError::new(status, detail),
        }
    }
}

impl From<GamesApiError> for GamesApiProblem {
    fn from(error: GamesApiError) -> Self {
        Self { error }
    }
}

impl From<GameError> for GamesApiProblem {
    fn from(error: GameError) -> Self {
        GamesApiError::from(error).into()
    }
}

impl From<LeaderboardError> for GamesApiError {
    fn from(error: LeaderboardError) -> Self {
        let status = match error.code() {
            "not_found" => StatusCode::NOT_FOUND,
            "invalid" => StatusCode::BAD_REQUEST,
            "invalid_parameter" => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if error.code() == "invalid_parameter" {
            Self::with_result_code(status, error.message(), SdkWorkResultCode::InvalidParameter)
        } else {
            Self::new(status, error.message())
        }
    }
}

impl From<LeaderboardError> for GamesApiProblem {
    fn from(error: LeaderboardError) -> Self {
        GamesApiError::from(error).into()
    }
}

impl From<GameRoomError> for GamesApiError {
    fn from(error: GameRoomError) -> Self {
        let status = match error.code() {
            "not_found" => StatusCode::NOT_FOUND,
            "invalid" => StatusCode::BAD_REQUEST,
            "invalid_parameter" => StatusCode::BAD_REQUEST,
            "conflict" => StatusCode::CONFLICT,
            "forbidden" => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if error.code() == "invalid_parameter" {
            Self::with_result_code(status, error.message(), SdkWorkResultCode::InvalidParameter)
        } else {
            Self::new(status, error.message())
        }
    }
}

impl From<GameRoomError> for GamesApiProblem {
    fn from(error: GameRoomError) -> Self {
        GamesApiError::from(error).into()
    }
}

impl IntoResponse for GamesApiProblem {
    fn into_response(self) -> Response {
        let correlation = crate::correlation::GamesProblemCorrelation::current();
        let fallback_request_id = correlation.is_none().then(new_request_id);
        let request_id = correlation
            .as_ref()
            .map(|value| value.request_id.as_str())
            .or(fallback_request_id.as_deref());
        let trace_id = correlation
            .as_ref()
            .and_then(|value| value.trace_id.as_deref());
        if let Some(result_code) = self.error.result_code {
            let correlation = ProblemCorrelation::new(request_id, trace_id);
            let resolved_trace_id = correlation
                .resolved_trace_id()
                .unwrap_or_else(|| "unknown".to_owned());
            let mut problem = SdkWorkProblemDetail::platform_enriched(
                result_code,
                self.error.detail,
                resolved_trace_id.clone(),
                SdkWorkProblemRouting::default(),
            );
            problem.status = self.error.status.as_u16();
            let mut response = (
                self.error.status,
                [(header::CONTENT_TYPE, "application/problem+json")],
                Json(problem),
            )
                .into_response();
            if let Ok(value) = axum::http::HeaderValue::from_str(&resolved_trace_id) {
                response.headers_mut().insert(
                    axum::http::HeaderName::from_static("x-sdkwork-trace-id"),
                    value,
                );
            }
            return response;
        }
        problem_response(
            &self.error.framework_error(),
            ProblemCorrelation::new(request_id, trace_id),
        )
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware::from_fn;
    use axum::routing::get;
    use axum::Router;
    use sdkwork_web_core::{REQUEST_ID_HEADER, TRACEPARENT_HEADER};
    use tower::util::ServiceExt;

    use crate::correlation::problem_correlation_middleware;
    use crate::problem::{GamesApiError, GamesApiProblem};

    async fn failing_handler() -> Result<&'static str, GamesApiProblem> {
        Err(GamesApiProblem::new(
            StatusCode::BAD_REQUEST,
            "game_id is required",
        ))
    }

    #[tokio::test]
    async fn problem_response_includes_trace_id_and_numeric_code() {
        let app = Router::new()
            .route("/test", get(failing_handler))
            .layer(from_fn(problem_correlation_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header(REQUEST_ID_HEADER, "req-games-1")
                    .header(
                        TRACEPARENT_HEADER,
                        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(payload.get("requestId").is_none());
        assert_eq!(payload["traceId"], "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(40_001, payload["code"].as_i64().unwrap());
    }

    #[test]
    fn room_domain_errors_map_to_http_problem_statuses() {
        let conflict = GamesApiError::from(sdkwork_game_room_service::GameRoomError::conflict(
            "room version has changed",
        ));
        assert_eq!(StatusCode::CONFLICT, conflict.status);

        let forbidden = GamesApiError::from(sdkwork_game_room_service::GameRoomError::forbidden(
            "only room host can perform this action",
        ));
        assert_eq!(StatusCode::FORBIDDEN, forbidden.status);
    }
}
