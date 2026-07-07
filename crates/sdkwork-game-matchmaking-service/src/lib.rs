//! SDKWork games matchmaking domain service.

pub mod domain;
pub mod ports;
pub mod service;

pub use domain::models::{
    CancelMatchTicketCommand, CreateMatchTicketCommand, GameMatchmakingError,
    GameMatchmakingResult, MatchTicketItem, MatchTicketPage, MatchTicketQuery,
    MatchmakingQueueQuery,
};
pub use ports::repository::GameMatchmakingRepository;
pub use service::GameMatchmakingService;
