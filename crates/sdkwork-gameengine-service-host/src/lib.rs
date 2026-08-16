use std::sync::Arc;

use sdkwork_database_sqlx::DatabasePool;
use sdkwork_game_catalog_repository_sqlx::{GameCatalogRepositoryKind, SqlxGameCatalogRepository};
use sdkwork_game_catalog_service::GameCatalogService;
use sdkwork_game_leaderboard_repository_sqlx::{
    LeaderboardRepositoryKind, SqlxLeaderboardRepository,
};
use sdkwork_game_leaderboard_service::LeaderboardService;
use sdkwork_game_room_repository_sqlx::{GameRoomRepositoryKind, SqlxGameRoomRepository};
use sdkwork_game_room_service::GameRoomService;
use sdkwork_gameengine_database_host::GamesDatabaseHost;

pub type SharedCatalogService = Arc<GameCatalogService<GameCatalogRepositoryKind>>;
pub type SharedLeaderboardService = Arc<LeaderboardService<LeaderboardRepositoryKind>>;
pub type SharedRoomService = Arc<GameRoomService<GameRoomRepositoryKind>>;

#[derive(Clone)]
pub struct GatewayServices {
    pub catalog: SharedCatalogService,
    pub leaderboard: SharedLeaderboardService,
    pub room: SharedRoomService,
}

pub struct GatewayRuntime {
    pub services: GatewayServices,
    pub database_pool: DatabasePool,
}

pub async fn build_gateway_services() -> Result<GatewayServices, String> {
    Ok(build_gateway_runtime().await?.services)
}

pub async fn build_gateway_runtime() -> Result<GatewayRuntime, String> {
    let host = sdkwork_gameengine_database_host::bootstrap_games_database_from_env().await?;
    Ok(gateway_runtime_from_host(host))
}

pub async fn build_gateway_runtime_with_pool(pool: DatabasePool) -> Result<GatewayRuntime, String> {
    let host = sdkwork_gameengine_database_host::bootstrap_games_database(pool).await?;
    Ok(gateway_runtime_from_host(host))
}

fn gateway_runtime_from_host(host: GamesDatabaseHost) -> GatewayRuntime {
    let database_pool = host.pool().clone();
    let services = GatewayServices {
        catalog: build_catalog_service(&host),
        leaderboard: build_leaderboard_service(&host),
        room: build_room_service(&host),
    };
    GatewayRuntime {
        services,
        database_pool,
    }
}

pub fn build_catalog_service(host: &GamesDatabaseHost) -> SharedCatalogService {
    let repository = GameCatalogRepositoryKind::Sqlx(Box::new(SqlxGameCatalogRepository::new(
        host.pool().clone(),
    )));
    Arc::new(GameCatalogService::new(repository))
}

pub fn build_leaderboard_service(host: &GamesDatabaseHost) -> SharedLeaderboardService {
    let repository = LeaderboardRepositoryKind::Sqlx(Box::new(SqlxLeaderboardRepository::new(
        host.pool().clone(),
    )));
    Arc::new(LeaderboardService::new(repository))
}

pub fn build_room_service(host: &GamesDatabaseHost) -> SharedRoomService {
    let repository =
        GameRoomRepositoryKind::Sqlx(Box::new(SqlxGameRoomRepository::new(host.pool().clone())));
    Arc::new(GameRoomService::new(repository))
}
