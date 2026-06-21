//! SDKWork game catalog service contracts.

pub mod domain;
pub mod ports;
pub mod service;

pub use domain::models::{
    GameCatalogItem, GameCatalogPage, GameCatalogQuery, GameError, GameResult,
};
pub use ports::repository::GameCatalogRepository;
pub use service::GameCatalogService;
