use crate::states::State;
use handlebars::Handlebars;
use label_logger::info;
use oauth2::{reqwest::http_client, AuthorizationCode, RefreshToken};
use rouille::{router, Response, Server};
use serde::Serialize;
use std::{io::stdout, thread};

#[derive(Serialize)]
struct IndexTemplateData {}

#[derive(Serialize)]
struct AuthSuccessTemplateData {
	token: String,
}

#[derive(Serialize)]
struct AuthErrorTemplateData {
	error_message: String,
}

pub fn launch_server(port: usize, data: State) {
	let templates = vec![
		("index", "index"),
		("auth_success", "auth/success"),
		("auth_error", "auth/error"),
	];

	let templates = {
		let mut handlebars = Handlebars::new();

		for (name, path) in templates {
			handlebars
				.register_template_file(name, format!("./server/templates/{path}.hbs"))
				.unwrap();
		}

		handlebars
	};

	thread::spawn(move || {
		Server::new(format!("localhost:{port}"), move |request| {
			rouille::log(request, stdout(), || {
				router!(request,
					(GET) (/) => {
						Response::html(templates.render("index", &IndexTemplateData {}).unwrap())
					},
					(GET) (/oauth2) => {
						let code = match request.get_param("code") {
							Some(code) => code,
							None => {
								return Response::html(templates.render("auth_error", &AuthErrorTemplateData {
									error_message: "You need to provide a 'code' param in url".into(),
								}).unwrap());
							}
						};

						let state = match request.get_param("state") {
							Some(state) => state,
							None => {
								return Response::html(
									templates.render("auth_error", &AuthErrorTemplateData {
										error_message: "You need to provide a 'state' param in url".to_owned(),
									}).unwrap(),
								);
							}
						};

						let mut queue = data.auth.queue.write().unwrap();

						dbg!(&queue);

						let rlt = queue.get(&state).unwrap();

						info!("Google: {rlt:?} vs The expected: {state}");

						let token_res = data.auth.client
							.exchange_code(AuthorizationCode::new(code))
							.request(http_client);

						if let Ok(_token) = token_res {
							queue.insert(state, Some(RefreshToken::new("".into())));
						}

						Response::html(templates.render("auth_success", &AuthSuccessTemplateData { token: "".into() }).unwrap())
					},
					_ => {
						let response = rouille::match_assets(request, "./server/static");

						if response.is_success() {
							response
						} else {
							Response::empty_404()
						}
					}
				)
			})
		})
		.unwrap()
		.pool_size(4)
		.run();

		panic!("Server crashed");
	});
}
