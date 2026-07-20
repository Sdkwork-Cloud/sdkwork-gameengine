use std::sync::Arc;

use sdkwork_game_room_repository_sqlx::{GameRoomRepositoryKind, SqlxGameRoomRepository};
use sdkwork_game_room_service::GameRoomService;
use sdkwork_games_database_host::GamesDatabaseHost;

pub type SharedRoomService = Arc<GameRoomService<GameRoomRepositoryKind>>;

pub fn build_room_service(host: &GamesDatabaseHost) -> SharedRoomService {
    let repository =
        GameRoomRepositoryKind::Sqlx(Box::new(SqlxGameRoomRepository::new(host.pool().clone())));
    Arc::new(GameRoomService::new(repository))
}
