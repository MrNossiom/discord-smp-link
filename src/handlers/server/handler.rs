//! The request handlers that serves content

use super::templates::*;
use crate::states::ArcData;
use askama::Template;
use futures::{future, Future};
use hyper::{service::Service, Body, Method, Request, Response, StatusCode};
use hyper_staticfile::Static as StaticRouter;
use oauth2::{reqwest::http_client, AuthorizationCode};
use std::{
	collections::HashMap,
	pin::Pin,
	sync::Arc,
	task::{Context, Poll},
};
use url::Url;

/// The `MakeService` that creates [`RequestHandler`]s to handle every request
#[derive(Clone)]
pub(crate) struct MakeRequestHandler {
	/// The data shared between the handlers
	pub(crate) data: ArcData,
	/// The router to serve static files
	pub(crate) static_router: StaticRouter,
}

impl<T> Service<T> for MakeRequestHandler {
	type Response = RequestHandler;
	type Error = anyhow::Error;
	type Future = future::Ready<Result<Self::Response, Self::Error>>;

	fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Ok(()).into()
	}

	fn call(&mut self, _: T) -> Self::Future {
		future::ok(RequestHandler {
			data: Arc::clone(&self.data),
			static_router: self.static_router.clone(),
		})
	}
}

/// The Service that handles every request
#[derive(Clone)]
pub(crate) struct RequestHandler {
	/// The data shared between the handlers
	pub(crate) data: ArcData,
	/// The router to serve static files
	pub(crate) static_router: StaticRouter,
}

impl Service<Request<Body>> for RequestHandler {
	type Response = Response<Body>;
	type Error = anyhow::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}

	fn call(&mut self, req: Request<Body>) -> Self::Future {
		let data = Arc::clone(&self.data);
		let static_router = self.static_router.clone();

		Box::pin(async {
			let res = match (req.method(), req.uri().path()) {
				(&Method::GET, "/") => Response::template(IndexTemplate {}),
				(&Method::GET, "/contact") => Response::template(ContactTemplate {}),
				(&Method::GET, "/privacy-policy") => Response::template(PrivacyPolicyTemplate {}),
				(&Method::GET, "/terms-and-conditions") => {
					Response::template(TermsAmdConditionsTemplate {})
				}
				(&Method::GET, "/oauth2") => handle_oauth2(data, req)?,
				_ => static_router.serve(req).await?,
			};

			Ok(res)
		})
	}
}

/// The trait for a custom `rouille` template response
trait TemplateResponse {
	/// Render a given template or return a 500 error
	fn template<D>(content: D) -> Response<Body>
	where
		D: Template,
	{
		match content.render() {
			Ok(content) => Response::new(Body::from(content)),
			Err(error) => {
				tracing::error!("could not render template: {}", error);

				let error_template = Code500Template {
					error_message: "Could not render template",
				}
				.render();

				// TODO: handle errors instead of unwrapping

				if let Ok(html_content) = error_template {
					Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(Body::from(html_content))
						.expect("could not build response")
				} else {
					Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(Body::empty())
						.expect("could not build response")
				}
			}
		}
	}
}

impl TemplateResponse for Response<Body> {}

// TODO: show more comprehensive errors to the user
/// Handle requests to `/oauth2` endpoints
fn handle_oauth2(data: ArcData, request: Request<Body>) -> anyhow::Result<Response<Body>> {
	let url = Url::parse(&request.uri().to_string())?;
	let params = url.query_pairs().collect::<HashMap<_, _>>();

	let code = match params.get("code") {
		Some(code) => code,
		None => {
			return Ok(Response::template(AuthTemplate {
				is_success: false,
				error_message: "You need to provide a 'code' param in url",
				..Default::default()
			}));
		}
	};

	let state = match params.get("state") {
		Some(state) => state,
		None => {
			return Ok(Response::template(AuthTemplate {
				is_success: false,
				error_message: "You need to provide a 'state' param in url",
				..Default::default()
			}));
		}
	};

	let mut queue = data.auth.queue.write().expect("RwLock poisoned");

	if queue.get(&state.to_string()).is_none() {
		return Ok(Response::template(AuthTemplate {
			is_success: false,
			error_message: "The given 'state' wasn't queued anymore",
			..Default::default()
		}));
	};

	let oauth2_response = data
		.auth
		.client
		.exchange_code(AuthorizationCode::new(code.to_string()))
		.request(http_client);

	let token_response = match oauth2_response {
		Ok(token_res) => token_res,
		Err(error) => {
			return Ok(Response::template(AuthTemplate {
				is_success: false,
				error_message: &error.to_string(),
				..Default::default()
			}));
		}
	};

	queue.insert(state.to_string(), Some(token_response));

	Ok(Response::template(AuthTemplate {
		is_success: true,
		// TODO
		username: "",
		..Default::default()
	}))
}
