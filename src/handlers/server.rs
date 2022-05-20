//! Handle the server for `OAuth2` and presentation page

use crate::states::STATE;
use askama::Template;
use log::{error, info};
use oauth2::{reqwest::http_client, AuthorizationCode, TokenResponse};
use rouille::{log_custom, Request, Response, Server};
use std::thread;

/// A template for the index page
#[derive(Template)]
#[template(path = "index.jinja")]
struct IndexTemplate {}

/// A template for the `OAuth2` success page
#[derive(Template)]
#[template(path = "auth/success.jinja")]
struct AuthSuccessTemplate<'a> {
	/// The token response from google
	token: &'a str,
}

/// A template for the `OAuth2` error page
#[derive(Template)]
#[template(path = "auth/error.jinja")]
struct AuthErrorTemplate<'a> {
	/// The error message to show on the page
	error_message: &'a str,
}

/// Spawn the server and setup logs
pub fn spawn_server() {
	thread::spawn(move || {
		Server::new(format!("localhost:{}", STATE.config.port), move |request| {
			log_custom(
				request,
				|req, res, elapsed| {
					info!(
						target: "SERVER",
						"{} {} - {}s - {}", req.raw_url(), req.method(), elapsed.as_secs(), res.status_code
					);
				},
				|req, elapsed| {
					let _ = error!(
						"{} {} - {}s - PANIC!",
						req.method(),
						req.raw_url(),
						elapsed.as_secs()
					);
				},
				|| handle_request(request),
			)
		})
		.expect("could not create socket")
		.pool_size(4)
		.run();

		panic!("Server crashed");
	});
}

/// Handles server requests
// TODO : to `OAuth2` response from Google
fn handle_request(request: &Request) -> Response {
	let request_url = {
		let url = request.raw_url();
		let pos = url.find('?').unwrap_or(url.len());
		&url[..pos]
	};

	match (request.method(), request_url) {
		("GET", "/") => Response::template(IndexTemplate {}),
		("GET", "/oauth2") => {
			let code = match request.get_param("code") {
				Some(code) => code,
				None => {
					return Response::template(AuthErrorTemplate {
						error_message: "You need to provide a 'code' param in url",
					});
				}
			};

			let state = match request.get_param("state") {
				Some(state) => state,
				None => {
					return Response::template(AuthErrorTemplate {
						error_message: "You need to provide a 'state' param in url",
					});
				}
			};

			let mut queue = STATE.auth.queue.write().expect("RwLock poisoned");

			if queue.get(&state).is_none() {
				return Response::template(AuthErrorTemplate {
					error_message: "The given 'state' wasn't queued anymore",
				});
			};

			let oauth2_response = STATE
				.auth
				.client
				.exchange_code(AuthorizationCode::new(code))
				.request(http_client);

			let token_response = match oauth2_response {
				Ok(token_res) => token_res,
				Err(error) => {
					return Response::template(AuthErrorTemplate {
						error_message: &error.to_string(),
					});
				}
			};

			queue.insert(state, Some(token_response.clone()));

			let refresh_token = token_response
				.refresh_token()
				.expect("google response didn't contain a token")
				.secret()
				.as_str();

			Response::template(AuthSuccessTemplate {
				token: refresh_token,
			})
		}
		_ => {
			let response = rouille::match_assets(request, "./server/static");

			if response.is_success() {
				response
			} else {
				Response::empty_404()
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
					AuthErrorTemplate {
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
