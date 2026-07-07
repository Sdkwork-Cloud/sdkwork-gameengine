use async_trait::async_trait;
use sdkwork_game_room_service::{
    CloseGameRoomCommand, CreateGameRoomCommand, GameRoomItem, GameRoomPage, GameRoomQuery,
    GameRoomRepository, GameRoomResult, GameRoomSeatItem, JoinGameRoomCommand,
    LeaveGameRoomCommand, ReadyGameRoomCommand, StartGameRoomCommand,
};

#[cfg(any(test, feature = "test-support"))]
use crate::memory::InMemoryGameRoomRepository;
use crate::sqlx::SqlxGameRoomRepository;

#[derive(Clone)]
pub enum GameRoomRepositoryKind {
    #[cfg(any(test, feature = "test-support"))]
    Memory(InMemoryGameRoomRepository),
    Sqlx(Box<SqlxGameRoomRepository>),
}

#[async_trait]
impl GameRoomRepository for GameRoomRepositoryKind {
    async fn list_rooms(
        &self,
        tenant_id: &str,
        query: &GameRoomQuery,
    ) -> GameRoomResult<GameRoomPage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_rooms(tenant_id, query).await,
            Self::Sqlx(repo) => repo.list_rooms(tenant_id, query).await,
        }
    }

    async fn get_room(&self, tenant_id: &str, room_id: &str) -> GameRoomResult<GameRoomItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.get_room(tenant_id, room_id).await,
            Self::Sqlx(repo) => repo.get_room(tenant_id, room_id).await,
        }
    }

    async fn list_room_seats(
        &self,
        tenant_id: &str,
        room_id: &str,
    ) -> GameRoomResult<Vec<GameRoomSeatItem>> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_room_seats(tenant_id, room_id).await,
            Self::Sqlx(repo) => repo.list_room_seats(tenant_id, room_id).await,
        }
    }

    async fn create_room(
        &self,
        tenant_id: &str,
        command: &CreateGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.create_room(tenant_id, command).await,
            Self::Sqlx(repo) => repo.create_room(tenant_id, command).await,
        }
    }

    async fn join_room(
        &self,
        tenant_id: &str,
        command: &JoinGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.join_room(tenant_id, command).await,
            Self::Sqlx(repo) => repo.join_room(tenant_id, command).await,
        }
    }

    async fn leave_room(
        &self,
        tenant_id: &str,
        command: &LeaveGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.leave_room(tenant_id, command).await,
            Self::Sqlx(repo) => repo.leave_room(tenant_id, command).await,
        }
    }

    async fn set_ready(
        &self,
        tenant_id: &str,
        command: &ReadyGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.set_ready(tenant_id, command).await,
            Self::Sqlx(repo) => repo.set_ready(tenant_id, command).await,
        }
    }

    async fn start_room(
        &self,
        tenant_id: &str,
        command: &StartGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.start_room(tenant_id, command).await,
            Self::Sqlx(repo) => repo.start_room(tenant_id, command).await,
        }
    }

    async fn close_room(
        &self,
        tenant_id: &str,
        command: &CloseGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.close_room(tenant_id, command).await,
            Self::Sqlx(repo) => repo.close_room(tenant_id, command).await,
        }
    }
}
