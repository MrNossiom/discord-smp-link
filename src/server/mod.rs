// Triggers in all `Rocket` macros
#![allow(clippy::no_effect_underscore_binding)]

//! Servers configs and request handlers to serve `OAuth2` callbacks and the web pages.
//! Rocket server luncher to answer `OAuth2` Google redirects and serve the basic web pages.

mod handler;

use crate::states::ArcData;
use handler::{
	catch_404, catch_500, contact, discord_redirect, handle_oauth2, index, privacy_policy,
	terms_and_conditions,
};
use hyper::header::ACCEPT_LANGUAGE;
use rocket::{
	catchers,
	fs::FileServer,
	request::{FromRequest, Outcome},
	response::Responder,
	routes, Ignite, Request, Rocket,
};
use rocket_dyn_templates::{context, Template};
use std::ops::Deref;
use tokio::task::{self, JoinHandle};
use unic_langid::LanguageIdentifier;

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
		.spawn(rocket.launch())?;

	Ok(handle)
}

/// A wrapper around `LanguageIdentifier` to implement `FromRequest`
#[derive(Debug)]
pub(crate) struct AcceptLanguage(pub(crate) LanguageIdentifier);

impl Deref for AcceptLanguage {
	type Target = LanguageIdentifier;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AcceptLanguage {
	type Error = ();

	async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
		if let Some(lang) = request.headers().get_one(ACCEPT_LANGUAGE.as_str()) {
			if let Some(lang) = lang.split(',').next() {
				if let Ok(lang) = lang.parse::<LanguageIdentifier>() {
					return Outcome::Success(Self(lang));
				}
			}
		}

		Outcome::Error((rocket::http::Status::BadRequest, ()))
	}
}

/// An error return by a route handler
#[derive(Debug, thiserror::Error)]
enum ServerError {
	/// A message to show to the user
	#[error("User facing error: {0}")]
	User(String),
	/// An error that should be logged with a generic message shown to the user
	#[error(transparent)]
	Other(#[from] anyhow::Error),
}

impl<'r> Responder<'r, 'static> for ServerError {
	fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
		let message = match self {
			Self::User(message) => message,
			Self::Other(error) => {
				tracing::error!("Internal server error: {}", error);
				"Internal server error".to_string()
			}
		};

		Template::render("500", context! { message }).respond_to(request)
	}
}
