use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_game_catalog_repository_sqlx::{
    GameCatalogRepositoryKind, InMemoryGameCatalogRepository,
};
use sdkwork_game_catalog_service::GameCatalogService;
use sdkwork_gameengine_standalone_gateway::{
    build_catalog_service, build_router, with_games_app_request_context,
};
use sdkwork_routes_catalog_app_api::build_catalog_app_router;
use std::sync::Arc;
use tower::ServiceExt;

const DEV_AUTH_TOKEN: &str =
    "Bearer tenant_id=demo-tenant;user_id=user-1;session_id=session-1;app_id=games;auth_level=password";
const DEV_ACCESS_TOKEN: &str =
    "tenant_id=demo-tenant;user_id=user-1;session_id=session-1;app_id=games;environment=dev;deployment_mode=saas";

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
    let router = with_games_app_request_context(build_catalog_app_router(memory_catalog_service()));

    let response = router
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/games")
                .header("Authorization", DEV_AUTH_TOKEN)
                .header("Access-Token", DEV_ACCESS_TOKEN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn build_router_merges_health_and_catalog_routes() {
    let app = build_router(memory_catalog_service()).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/system/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn memory_repository_mode_builds_catalog_service() {
    std::env::set_var("GAMES_REPOSITORY_MODE", "memory");
    let service = build_catalog_service().await.expect("catalog service");
    assert!(Arc::strong_count(&service) >= 1);
}
