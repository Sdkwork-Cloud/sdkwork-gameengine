//! Gateway bootstrap for sdkwork-gameengine.
//!
//! The assembly exports the indivisible `ApiAssemblyContribution` contract
//! (API_ASSEMBLY_SPEC.md section 4); the platform cloud gateway composes the
//! contribution with its process-shared PostgreSQL pool.

use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_gameengine_service_host::{
    build_gateway_runtime, build_gateway_runtime_with_pool, GatewayServices, SharedCatalogService,
    SharedLeaderboardService, SharedRoomService,
};
use sdkwork_routes_gameengine_catalog_app_api::build_catalog_app_router;
use sdkwork_routes_gameengine_catalog_backend_api::build_catalog_backend_router;
use sdkwork_routes_leaderboard_app_api::build_leaderboard_app_router;
use sdkwork_routes_room_app_api::build_room_app_router;
use sdkwork_routes_room_backend_api::build_room_backend_router;
use sdkwork_web_bootstrap::{ApiAssemblyContribution, DatabasePoolReadinessCheck, ReadinessCheck};
use sdkwork_web_core::HttpRouteManifest;
use std::sync::Arc;

/// Indivisible host-neutral API assembly contribution (web-bootstrap contract).
pub type ApiAssembly = ApiAssemblyContribution;

pub struct ApiAssemblyRuntime {
    pub contribution: ApiAssembly,
    pub database_pool: DatabasePool,
}

fn combined_route_manifest() -> HttpRouteManifest {
    let manifests = [
        sdkwork_routes_gameengine_catalog_app_api::gateway_route_manifest(),
        sdkwork_routes_gameengine_catalog_backend_api::gateway_route_manifest(),
        sdkwork_routes_leaderboard_app_api::gateway_route_manifest(),
        sdkwork_routes_room_app_api::gateway_route_manifest(),
        sdkwork_routes_room_backend_api::gateway_route_manifest(),
    ];
    HttpRouteManifest::from_owned_routes(
        manifests
            .into_iter()
            .flat_map(|manifest| manifest.routes().to_vec())
            .collect(),
    )
}

fn games_router(services: GatewayServices) -> Router {
    let app = Router::new()
        .merge(build_catalog_app_router(services.catalog.clone()))
        .merge(build_leaderboard_app_router(services.leaderboard))
        .merge(build_room_app_router(services.room.clone()));
    let backend = build_catalog_backend_router(services.catalog)
        .merge(build_room_backend_router(services.room));
    Router::new().merge(app).merge(backend)
}

fn contribution_from(
    router: Router,
    readiness_check: Arc<dyn ReadinessCheck>,
) -> Result<ApiAssembly, String> {
    ApiAssemblyContribution::from_manifest(
        "sdkwork-gameengine",
        "SDKWork Game Engine API",
        router,
        combined_route_manifest(),
        Vec::new(),
        readiness_check,
    )
}

pub async fn assemble_api_router() -> Result<ApiAssembly, String> {
    Ok(assemble_api_router_runtime().await?.contribution)
}

pub async fn assemble_api_router_runtime() -> Result<ApiAssemblyRuntime, String> {
    let runtime = build_gateway_runtime().await?;
    let contribution = contribution_from(
        games_router(runtime.services),
        Arc::new(DatabasePoolReadinessCheck::new(
            runtime.database_pool.clone(),
        )),
    )?;
    Ok(ApiAssemblyRuntime {
        contribution,
        database_pool: runtime.database_pool,
    })
}

pub fn assemble_api_router_with_services(services: GatewayServices) -> ApiAssembly {
    contribution_from(
        games_router(services),
        Arc::new(sdkwork_web_bootstrap::AlwaysReady),
    )
    .expect("gameengine contribution contract is valid")
}

pub fn assemble_api_router_with_service_parts(
    catalog: SharedCatalogService,
    leaderboard: SharedLeaderboardService,
    room: SharedRoomService,
) -> ApiAssembly {
    assemble_api_router_with_services(GatewayServices {
        catalog,
        leaderboard,
        room,
    })
}

pub async fn assemble_business_routes() -> Result<ApiAssembly, String> {
    assemble_api_router().await
}

/// Assemble the Game Engine contribution against a caller-provided database
/// pool so the platform cloud gateway can share its process-wide PostgreSQL
/// pool.
pub async fn assemble_api_router_with_pool(pool: DatabasePool) -> Result<ApiAssembly, String> {
    let runtime = build_gateway_runtime_with_pool(pool.clone()).await?;
    contribution_from(
        games_router(runtime.services),
        Arc::new(DatabasePoolReadinessCheck::new(pool)),
    )
}
