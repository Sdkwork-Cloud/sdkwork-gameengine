//! SQLx-backed game catalog repository.

mod kind;
mod memory;
mod sqlx;

pub use kind::GameCatalogRepositoryKind;
pub use memory::InMemoryGameCatalogRepository;
pub use sqlx::SqlxGameCatalogRepository;
