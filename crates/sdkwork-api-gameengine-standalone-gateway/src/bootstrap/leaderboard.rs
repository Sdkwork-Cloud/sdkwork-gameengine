use std::sync::Arc;

use sdkwork_game_leaderboard_repository_sqlx::{
    LeaderboardRepositoryKind, SqlxLeaderboardRepository,
};
use sdkwork_game_leaderboard_service::LeaderboardService;
use sdkwork_gameengine_database_host::GamesDatabaseHost;

pub type SharedLeaderboardService = Arc<LeaderboardService<LeaderboardRepositoryKind>>;

pub fn build_leaderboard_service(host: &GamesDatabaseHost) -> SharedLeaderboardService {
    let repository = LeaderboardRepositoryKind::Sqlx(Box::new(SqlxLeaderboardRepository::new(
        host.pool().clone(),
    )));
    Arc::new(LeaderboardService::new(repository))
}
