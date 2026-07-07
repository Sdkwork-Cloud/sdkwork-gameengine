use async_trait::async_trait;

use crate::domain::models::{
    CloseGameRoomCommand, CreateGameRoomCommand, GameRoomItem, GameRoomPage, GameRoomQuery,
    GameRoomResult, GameRoomSeatItem, JoinGameRoomCommand, LeaveGameRoomCommand,
    ReadyGameRoomCommand, StartGameRoomCommand,
};

#[async_trait]
pub trait GameRoomRepository: Send + Sync {
    async fn list_rooms(
        &self,
        tenant_id: &str,
        query: &GameRoomQuery,
    ) -> GameRoomResult<GameRoomPage>;

    async fn get_room(&self, tenant_id: &str, room_id: &str) -> GameRoomResult<GameRoomItem>;

    async fn list_room_seats(
        &self,
        tenant_id: &str,
        room_id: &str,
    ) -> GameRoomResult<Vec<GameRoomSeatItem>>;

    async fn create_room(
        &self,
        tenant_id: &str,
        command: &CreateGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem>;

    async fn join_room(
        &self,
        tenant_id: &str,
        command: &JoinGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem>;

    async fn leave_room(
        &self,
        tenant_id: &str,
        command: &LeaveGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem>;

    async fn set_ready(
        &self,
        tenant_id: &str,
        command: &ReadyGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem>;

    async fn start_room(
        &self,
        tenant_id: &str,
        command: &StartGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem>;

    async fn close_room(
        &self,
        tenant_id: &str,
        command: &CloseGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem>;
}
