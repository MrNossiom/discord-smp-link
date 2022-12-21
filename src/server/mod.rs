// Triggers in all `Rocket` macros
#![allow(clippy::no_effect_underscore_binding)]

//! Servers configs and request handlers to serve `OAuth2` callbacks and the web pages.
//! Rocket server luncher to answer `OAuth2` Google redirects and serve the basic web pages.

pub(self) mod handler;

use crate::states::ArcData;
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
	let figment = rocket::Config::figment();

	let rocket = rocket::custom(figment)
		.mount(
			"/",
			routes![
				index,
				contact,
				privacy_policy,
				terms_and_conditions,
				discord_redirect,
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
