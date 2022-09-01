//! Http Server to answer `OAuth2` redirects and show a presentation page

use crate::states::Data;
use askama::Template;
use oauth2::{reqwest::http_client, AuthorizationCode};
use rouille::{log_custom, Request, Response, Server};
use std::{
	process,
	sync::Arc,
	thread::{self, JoinHandle},
};

/// A template for the 404 page
#[derive(Template, Default)]
#[template(path = "404.jinja")]
struct Code404Template<'a> {
	/// The error message of 404 page
	ressource_path: &'a str,
}

/// A template for the 500 page
#[derive(Template, Default)]
#[template(path = "500.jinja")]
struct Code500Template<'a> {
	/// The error message of 500 page
	error_message: &'a str,
}

/// A template for the index page
#[derive(Template, Default)]
#[template(path = "index.jinja")]
struct IndexTemplate {}

/// A template for the contact page
#[derive(Template, Default)]
#[template(path = "contact.jinja")]
struct ContactTemplate {}

/// A template for the Privacy Policy page
#[derive(Template, Default)]
#[template(path = "privacy-policy.jinja")]
struct PrivacyPolicyTemplate {}

/// A template for the Terms of Service page
#[derive(Template, Default)]
#[template(path = "terms-and-conditions.jinja")]
struct TermsAmdConditionsTemplate {}

/// A template for the `OAuth2` success or error page
#[derive(Template, Default)]
#[template(path = "auth.jinja")]
struct AuthTemplate<'a> {
	/// The token response from google
	is_success: bool,
	/// The person discord username
	username: &'a str,
	/// The message in case of error
	error_message: &'a str,
}

/// Spawn the server in a separate thread
#[must_use]
pub(crate) fn spawn_server(data: Arc<Data>) -> JoinHandle<()> {
	tracing::debug!("Spawning server");

	thread::spawn(move || {
		// Listen on external interfaces `0.0.0.0`
		Server::new(format!("0.0.0.0:{}", data.config.port), move |request| {
			log_custom(
				request,
				|req, res, elapsed| {
					tracing::info!(
						"{} {} - {}s - {}",
						req.method(),
						req.raw_url(),
						elapsed.as_secs(),
						res.status_code
					);
				},
				|req, elapsed| {
					tracing::error!(
						"{} {} - {}s - PANIC!",
						req.method(),
						req.raw_url(),
						elapsed.as_secs()
					);
				},
				|| handle_request(Arc::clone(&data), request),
			)
		})
		.unwrap_or_else(|err| {
			tracing::error!("Could not create socket : {}", err);

			process::exit(1);
		})
		.pool_size(4)
		.run();
	})
}

// TODO: move each handle in a separate function
/// Handles server requests
fn handle_request(data: Arc<Data>, request: &Request) -> Response {
	let request_url = {
		let url = request.raw_url();
		let pos = url.find('?').unwrap_or(url.len());
		&url[..pos]
	};

	match (request.method(), request_url) {
		("GET", "/") => Response::template(IndexTemplate {}),
		("GET", "/contact") => Response::template(ContactTemplate {}),
		("GET", "/privacy-policy") => Response::template(PrivacyPolicyTemplate {}),
		("GET", "/terms-and-conditions") => Response::template(TermsAmdConditionsTemplate {}),
		("GET", "/oauth2") => {
			let code = match request.get_param("code") {
				Some(code) => code,
				None => {
					return Response::template(AuthTemplate {
						is_success: false,
						error_message: "You need to provide a 'code' param in url",
						..Default::default()
					});
				}
			};

			let state = match request.get_param("state") {
				Some(state) => state,
				None => {
					return Response::template(AuthTemplate {
						is_success: false,
						error_message: "You need to provide a 'state' param in url",
						..Default::default()
					});
				}
			};

			let mut queue = data.auth.queue.write().expect("RwLock poisoned");

			if queue.get(&state).is_none() {
				return Response::template(AuthTemplate {
					is_success: false,
					error_message: "The given 'state' wasn't queued anymore",
					..Default::default()
				});
			};

			let oauth2_response = data
				.auth
				.client
				.exchange_code(AuthorizationCode::new(code))
				.request(http_client);

			let token_response = match oauth2_response {
				Ok(token_res) => token_res,
				Err(error) => {
					return Response::template(AuthTemplate {
						is_success: false,
						error_message: &error.to_string(),
						..Default::default()
					});
				}
			};

			queue.insert(state, Some(token_response));

			Response::template(AuthTemplate {
				is_success: true,
				username: "",
				..Default::default()
			})
		}
		_ => {
			let response = rouille::match_assets(request, "./public");

			if response.is_success() {
				response
			} else {
				Response::template(Code404Template {
					ressource_path: request_url,
				})
				.with_status_code(404)
			}
		}
	}
}

/// The trait for a custom `rouille` template response
trait TemplateResponse {
	/// Render a given template or return a 500 error
	fn template<D>(content: D) -> Response
	where
		D: Template,
	{
		match content.render() {
			Ok(content) => Response::html(content),
			Err(error) => {
				println!("{}", error);
				Response::html(
					Code500Template {
						error_message: "Could not render template",
					}
					.render()
					.expect("could not render template properly"),
				)
				.with_status_code(500)
			}
		}
	}
}

impl TemplateResponse for Response {}
