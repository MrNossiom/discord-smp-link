//! [`ServerError`] is a custom error type for handlers to return [`anyhow::Error`]s

use axum::response::{IntoResponse, Response};
use hyper::{Body, StatusCode};
use std::fmt;

/// Result alias for server request handlers
pub(super) type ServerResult<R = Response<Body>> = Result<R, ServerError>;

/// Custom error type for server request handlers
pub(super) struct ServerError(anyhow::Error);

impl<E> From<E> for ServerError
where
	E: Into<anyhow::Error>,
{
	fn from(err: E) -> Self {
		Self(err.into())
	}
}

impl IntoResponse for ServerError {
	fn into_response(self) -> Response {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			format!("Something went wrong: {}", self.0),
		)
			.into_response()
	}
}

impl fmt::Display for ServerError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.0, f)
	}
}
