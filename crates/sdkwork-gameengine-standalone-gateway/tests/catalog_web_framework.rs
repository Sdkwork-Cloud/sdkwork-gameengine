use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use sdkwork_game_catalog_repository_sqlx::{
    GameCatalogRepositoryKind, InMemoryGameCatalogRepository,
};
use sdkwork_game_catalog_service::GameCatalogService;
use sdkwork_gameengine_standalone_gateway::{build_router, with_games_app_request_context};
use sdkwork_routes_catalog_app_api::build_catalog_app_router;
use sdkwork_web_core::{access_token_jwt, auth_token_jwt, TRACEPARENT_HEADER};
use std::sync::Arc;
use tower::ServiceExt;

const DEMO_TENANT: &str = "100001";
const DEMO_USER: &str = "user-1";
const DEMO_SESSION: &str = "session-1";
const DEMO_APP: &str = "games";

type SharedCatalogService = Arc<GameCatalogService<GameCatalogRepositoryKind>>;

fn memory_catalog_service() -> SharedCatalogService {
    Arc::new(GameCatalogService::new(GameCatalogRepositoryKind::Memory(
        InMemoryGameCatalogRepository::with_seed(vec![]),
    )))
}

#[tokio::test]
async fn catalog_router_rejects_unauthenticated_requests() {
    let router = with_games_app_request_context(build_catalog_app_router(memory_catalog_service()));

    let response = router
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/games")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn catalog_router_accepts_dev_inline_dual_tokens() {
    std::env::set_var("SDKWORK_ENV", "dev");
    let auth_token = format!(
        "Bearer {}",
        auth_token_jwt(DEMO_TENANT, DEMO_USER, DEMO_SESSION, DEMO_APP)
    );
    let access_token = access_token_jwt(DEMO_TENANT, DEMO_USER, DEMO_SESSION, DEMO_APP);
    let router = with_games_app_request_context(build_catalog_app_router(memory_catalog_service()));

    let response = router
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/games")
                .header("Authorization", auth_token)
                .header("Access-Token", access_token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn catalog_router_rejects_forbidden_pagination_alias_with_problem_json() {
    std::env::set_var("SDKWORK_ENV", "dev");
    let auth_token = format!(
        "Bearer {}",
        auth_token_jwt(DEMO_TENANT, DEMO_USER, DEMO_SESSION, DEMO_APP)
    );
    let access_token = access_token_jwt(DEMO_TENANT, DEMO_USER, DEMO_SESSION, DEMO_APP);
    let router = with_games_app_request_context(build_catalog_app_router(memory_catalog_service()));

    let response = router
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/games?pageSize=20")
                .header("Authorization", auth_token)
                .header("Access-Token", access_token)
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
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    assert!(
        content_type.starts_with("application/problem+json"),
        "unexpected content-type: {content_type}"
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("problem body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("problem json");
    assert_eq!(40003, payload["code"].as_i64().unwrap());
    assert_eq!(
        "page and page_size must follow SDKWork pagination bounds",
        payload["detail"].as_str().unwrap()
    );
    assert_eq!(
        "4bf92f3577b34da6a3ce929d0e0e4736",
        payload["traceId"].as_str().unwrap()
    );
}

#[tokio::test]
async fn build_router_merges_health_and_catalog_routes() {
    use sdkwork_game_leaderboard_repository_sqlx::{
        InMemoryLeaderboardRepository, LeaderboardRepositoryKind,
    };
    use sdkwork_game_leaderboard_service::LeaderboardService;
    use sdkwork_game_room_repository_sqlx::{GameRoomRepositoryKind, InMemoryGameRoomRepository};
    use sdkwork_game_room_service::GameRoomService;

    let leaderboard = Arc::new(LeaderboardService::new(LeaderboardRepositoryKind::Memory(
        InMemoryLeaderboardRepository::with_seed(vec![]),
    )));
    let room_service = Arc::new(GameRoomService::new(GameRoomRepositoryKind::Memory(
        InMemoryGameRoomRepository::with_seed(vec![]),
    )));
    let app = build_router(memory_catalog_service(), leaderboard, room_service).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/games/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn standalone_gateway_main_uses_single_database_bootstrap() {
    let main_source = include_str!("../src/main.rs");

    assert!(
        main_source.contains("assemble_application_router"),
        "standalone gateway main must bootstrap all business routes through gateway assembly"
    );
    assert!(
        !main_source.contains("build_catalog_service()"),
        "catalog service must not bootstrap its own database pool in the production entrypoint"
    );
    assert!(
        !main_source.contains("build_leaderboard_service()"),
        "leaderboard service must not bootstrap its own database pool in the production entrypoint"
    );
    assert!(
        !main_source.contains("build_room_service()"),
        "room service must not bootstrap its own database pool in the production entrypoint"
    );
}
