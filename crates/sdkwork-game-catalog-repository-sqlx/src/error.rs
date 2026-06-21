use sdkwork_game_catalog_service::{GameCatalogItem, GameError, GameResult};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(String),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

pub fn map_repository_error(error: impl std::fmt::Display) -> GameError {
    GameError::invalid(format!("repository error: {error}"))
}

pub fn sql_error(error: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::Database(error.to_string())
}
