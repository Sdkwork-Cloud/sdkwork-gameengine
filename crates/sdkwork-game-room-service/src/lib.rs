//! SDKWork game room service contracts.

pub mod domain;
pub mod ports;
pub mod service;

pub use domain::models::{
    CloseGameRoomCommand, CreateGameRoomCommand, GameRoomError, GameRoomItem, GameRoomPage,
    GameRoomQuery, GameRoomResult, GameRoomSeatItem, JoinGameRoomCommand, LeaveGameRoomCommand,
    ReadyGameRoomCommand, StartGameRoomCommand,
};
pub use ports::repository::GameRoomRepository;
pub use service::GameRoomService;
