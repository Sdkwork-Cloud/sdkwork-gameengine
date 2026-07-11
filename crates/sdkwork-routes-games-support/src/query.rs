use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use serde::de::DeserializeOwned;

use crate::problem::{GamesApiError, GamesApiProblem};

pub const INVALID_PAGINATION_DETAIL: &str =
    "page and page_size must follow SDKWork pagination bounds";

#[derive(Debug, Clone, Copy, Default)]
pub struct StrictListQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for StrictListQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = GamesApiProblem;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::try_from_uri(&parts.uri)
            .map(|Query(query)| Self(query))
            .map_err(|_| GamesApiError::invalid_parameter(INVALID_PAGINATION_DETAIL).into())
    }
}

impl<T> std::ops::Deref for StrictListQuery<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
