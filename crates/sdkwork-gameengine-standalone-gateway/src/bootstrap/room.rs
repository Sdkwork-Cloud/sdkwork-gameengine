use std::sync::Arc;

use sdkwork_game_room_repository_sqlx::{GameRoomRepositoryKind, SqlxGameRoomRepository};
use sdkwork_game_room_service::GameRoomService;
use sdkwork_games_database_host::bootstrap_games_database_from_env;

pub type SharedRoomService = Arc<GameRoomService<GameRoomRepositoryKind>>;

pub async fn build_room_service() -> Result<SharedRoomService, String> {
    let host = bootstrap_games_database_from_env().await?;
    let repository =
        GameRoomRepositoryKind::Sqlx(Box::new(SqlxGameRoomRepository::new(host.pool().clone())));
    Ok(Arc::new(GameRoomService::new(repository)))
}
