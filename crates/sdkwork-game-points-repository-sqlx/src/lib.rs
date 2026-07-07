//! SQLx-backed game point ledger repository.

#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryGamePointRepository;
pub use sqlx::SqlxGamePointRepository;
