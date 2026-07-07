use std::sync::Arc;

use sdkwork_game_leaderboard_repository_sqlx::{
    LeaderboardRepositoryKind, SqlxLeaderboardRepository,
};
use sdkwork_game_leaderboard_service::LeaderboardService;
use sdkwork_games_database_host::bootstrap_games_database_from_env;

pub type SharedLeaderboardService = Arc<LeaderboardService<LeaderboardRepositoryKind>>;

pub async fn build_leaderboard_service() -> Result<SharedLeaderboardService, String> {
    let host = bootstrap_games_database_from_env().await?;
    let repository = LeaderboardRepositoryKind::Sqlx(Box::new(SqlxLeaderboardRepository::new(
        host.pool().clone(),
    )));
    Ok(Arc::new(LeaderboardService::new(repository)))
}
