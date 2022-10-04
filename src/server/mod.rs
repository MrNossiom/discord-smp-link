//! Servers configs and request handlers to serve `OAuth2` callbacks and the web pages.
//! Rocket server luncher to answer `OAuth2` Google redirects and serve the basic web pages.

pub(self) mod handler;

use crate::states::ArcData;
use figment::{
	providers::{Env, Format, Toml},
	Figment,
};
use handler::{
	catch_404, catch_500, contact, discord_redirect, handle_oauth2, index, privacy_policy,
	terms_and_conditions,
};
use rocket::{catchers, fs::FileServer, routes, Ignite, Rocket};
use rocket_dyn_templates::Template;
use tokio::task::{self, JoinHandle};

/// Start the HTTP and HTTPS server in a new tokio task
pub(crate) fn start_server(
	data: ArcData,
) -> anyhow::Result<JoinHandle<Result<Rocket<Ignite>, rocket::Error>>> {
	let figment = Figment::from(rocket::Config::default())
		.merge(Toml::file("Config.toml").nested())
		.merge(Env::prefixed("SMP_").global());

	let rocket = rocket::custom(figment)
		.mount(
			"/",
			routes![
				index,
				discord_redirect,
				privacy_policy,
				contact,
				terms_and_conditions,
				handle_oauth2
			],
		)
		.mount("/static", FileServer::from("public/"))
		.register("/", catchers![catch_404, catch_500])
		.attach(Template::fairing())
		.manage(data);

	// Create a TCP listener for HTTP via tokio
	let handle = task::Builder::new()
		.name("Rocket Server")
		.spawn(async move { rocket.launch().await })?;

	Ok(handle)
}
