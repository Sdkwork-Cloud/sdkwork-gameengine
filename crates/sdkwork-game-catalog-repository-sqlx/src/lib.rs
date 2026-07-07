//! SQLx-backed game catalog repository.

mod catalog_list_query;
mod kind;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

pub use kind::GameCatalogRepositoryKind;
#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryGameCatalogRepository;
pub use sqlx::SqlxGameCatalogRepository;
