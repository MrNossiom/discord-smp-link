//! The request handlers that serves content

use crate::states::ArcData;

use super::{error::ServerResult, templates::*};
use askama::Template;
use axum::{
	body::boxed,
	extract::Query,
	response::{Html, IntoResponse, Response},
	Extension,
};
use hyper::{Body, StatusCode, Uri};
use oauth2::{reqwest::async_http_client, AuthorizationCode};
use std::{collections::HashMap, io};

/// A wrapper that implement [`IntoResponse`]
pub(super) struct TemplateResponse<T>(pub T);

impl<T: Template> IntoResponse for TemplateResponse<T> {
	fn into_response(self) -> Response {
		match self.0.render() {
			Ok(content) => Html(content).into_response(),
			Err(error) => {
				tracing::error!("could not render template: {}", error);

				let error_template = Code500Template {
					error_message: "Could not render template".into(),
				}
				.render();

				// TODO: handle errors instead of unwrapping

				if let Ok(html_content) = error_template {
					Html(html_content).into_response()
				} else {
					Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(boxed(Body::empty()))
						.expect("could not build response")
				}
			}
		}
	}
}

// TODO: show more comprehensive errors to the user
/// Handle requests to `/oauth2` endpoints
pub(super) async fn handle_oauth2(
	Query(params): Query<HashMap<String, String>>,
	Extension(data): Extension<ArcData>,
) -> ServerResult<impl IntoResponse> {
	let code = match params.get("code") {
		Some(code) => code,
		None => {
			return Ok(TemplateResponse(AuthTemplate {
				is_success: false,
				error_message: "You need to provide a 'code' param in url".into(),
				..Default::default()
			}));
		}
	};

	let state = match params.get("state") {
		Some(state) => state,
		None => {
			return Ok(TemplateResponse(AuthTemplate {
				is_success: false,
				error_message: "You need to provide a 'state' param in url".into(),
				..Default::default()
			}));
		}
	};

	{
		let queue = data.auth.queue.read().expect("poisoned");

		if queue.get(&state.to_string()).is_none() {
			return Ok(TemplateResponse(AuthTemplate {
				is_success: false,
				error_message: "The given 'state' wasn't queued anymore".into(),
				..Default::default()
			}));
		};
	}

	let oauth2_response = data
		.auth
		.client
		.exchange_code(AuthorizationCode::new(code.to_string()))
		.request_async(async_http_client)
		.await;

	let token_response = match oauth2_response {
		Ok(token_res) => token_res,
		Err(error) => {
			return Ok(TemplateResponse(AuthTemplate {
				is_success: false,
				error_message: error.to_string(),
				..Default::default()
			}));
		}
	};

	{
		let mut queue = data.auth.queue.write().expect("poisoned");

		queue.insert(state.to_string(), Some(token_response));
	}

	Ok(TemplateResponse(AuthTemplate {
		is_success: true,
		// TODO
		username: "".into(),
		..Default::default()
	}))
}

/// Fallback handler for non-matched routes
#[allow(clippy::unused_async)]
pub(super) async fn fallback(uri: Uri) -> impl IntoResponse {
	TemplateResponse(Code404Template {
		ressource_path: uri.path().to_owned(),
	})
}

/// Fallback handler for non-matched routes
#[allow(clippy::unused_async)]
pub(super) async fn handle_error_static(error: io::Error) -> impl IntoResponse {
	tracing::error!("could not serve static file: {}", error);

	StatusCode::INTERNAL_SERVER_ERROR
}
