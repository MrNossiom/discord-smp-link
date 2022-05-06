//! Handle the server for OAuth2 and presentation page

use crate::states::Data;
use askama::Template;
use oauth2::{reqwest::http_client, AuthorizationCode, TokenResponse};
use rouille::{Response, Server};
use std::{io::stdout, sync::Arc, thread};

/// A template for the index page
#[derive(Template)]
#[template(path = "index.jinja")]
struct IndexTemplate {}

/// A template for the oauth2 success page
#[derive(Template)]
#[template(path = "auth/success.jinja")]
struct AuthSuccessTemplate<'a> {
	/// The token response from google
	token: &'a str,
}

/// A template for the oauth2 error page
#[derive(Template)]
#[template(path = "auth/error.jinja")]
struct AuthErrorTemplate<'a> {
	/// The error message to show on the page
	error_message: &'a str,
}

/// Launch a local server to handle `OAuth2` response from Google
pub fn launch_server(port: usize, data: Arc<Data>) {
	thread::spawn(move || {
		Server::new(format!("localhost:{port}"), move |request| {
			rouille::log(request, stdout(), || {
				let request_url = {
					let url = request.raw_url();
					let pos = url.find('?').unwrap_or(url.len());
					&url[..pos]
				};

				match (request.method(), request_url) {
					("GET", "/") => Response::html(
						IndexTemplate {}
							.render()
							.expect("could not render template properly"),
					),
					("GET", "/oauth2") => {
						let code = match request.get_param("code") {
							Some(code) => code,
							None => {
								return Response::html(
									AuthErrorTemplate {
										error_message: "You need to provide a 'code' param in url",
									}
									.render()
									.expect("could not render template properly"),
								);
							}
						};

						let state = match request.get_param("state") {
							Some(state) => state,
							None => {
								return Response::html(
									AuthErrorTemplate {
										error_message: "You need to provide a 'state' param in url",
									}
									.render()
									.expect("could not render template properly"),
								);
							}
						};

						let mut queue = data.auth.queue.write().unwrap();

						if queue.get(&state).is_none() {
							return Response::html(
								AuthErrorTemplate {
									error_message: "The given 'state' wasn't queued anymore",
								}
								.render()
								.expect("could not render template properly"),
							);
						};

						let token_response = data
							.auth
							.client
							.exchange_code(AuthorizationCode::new(code))
							.request(http_client);

						let refresh_token = match token_response {
							Ok(token_res) => token_res,
							Err(error) => {
								return Response::html(
									AuthErrorTemplate {
										error_message: &error.to_string(),
									}
									.render()
									.expect("could not render template properly"),
								);
							}
						};

						queue.insert(state, Some(refresh_token.refresh_token().unwrap().clone()));

						Response::html(
							AuthSuccessTemplate {
								token: refresh_token
									.refresh_token()
									.unwrap()
									.clone()
									.secret()
									.to_string()
									.as_str(),
							}
							.render()
							.expect("could not render template properly"),
						)
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
			})
		})
		.expect("could not create socket")
		.pool_size(4)
		.run();

		panic!("Server crashed");
	});
}
