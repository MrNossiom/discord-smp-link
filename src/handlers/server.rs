use crate::states::State;
use askama::Template;
use oauth2::{reqwest::http_client, AuthorizationCode, TokenResponse};
use rouille::{Response, Server};
use std::{io::stdout, thread};

#[derive(Template)]
#[template(path = "index.jinja")]
struct IndexTemplate {}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "auth/success.jinja")]
struct AuthSuccessTemplate<'a> {
	token: &'a str,
}

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "auth/error.jinja")]
struct AuthErrorTemplate<'a> {
	error_message: &'a str,
}

pub fn launch_server(port: usize, data: State) {
	thread::spawn(move || {
		Server::new(format!("localhost:{port}"), move |request| {
			rouille::log(request, stdout(), || {
				let request_url = {
					let url = request.raw_url();
					let pos = url.find('?').unwrap_or(url.len());
					&url[..pos]
				};

				match (request.method(), request_url) {
					("GET", "/") => Response::html(IndexTemplate {}.render().unwrap()),
					("GET", "/oauth2") => {
						let code = match request.get_param("code") {
							Some(code) => code,
							None => {
								return Response::html(
									AuthErrorTemplate {
										error_message: "You need to provide a 'code' param in url",
									}
									.render()
									.unwrap(),
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
									.unwrap(),
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
								.unwrap(),
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
									.unwrap(),
								);
							}
						};

						queue.insert(
							state,
							Some(refresh_token.refresh_token().unwrap().to_owned()),
						);

						Response::html(
							AuthSuccessTemplate {
								token: refresh_token
									.refresh_token()
									.unwrap()
									.to_owned()
									.secret()
									.to_string()
									.as_str(),
							}
							.render()
							.unwrap(),
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
		.unwrap()
		.pool_size(4)
		.run();

		panic!("Server crashed");
	});
}
