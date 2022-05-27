//! Handles all the states of the bot and initial configuration

use crate::{database::DatabasePool, handlers::auth::AuthLink};
use anyhow::{Error, Result};
use diesel::{
	r2d2::{ConnectionManager, Pool},
	PgConnection,
};
use dotenv::dotenv;
use lazy_static::lazy_static;
use oauth2::{ClientId, ClientSecret};
use std::{
	env,
	sync::atomic::{AtomicBool, Ordering},
};

/// Store `true` if [`Data`] has been initialized
static DATA_CALLED: AtomicBool = AtomicBool::new(false);

lazy_static! {
	/// The globally available [`Data`]
	pub static ref STATE: Data = Data::new();
}

/// App global configuration
pub struct Config {
	/// The token needed to access the `Discord` Api
	pub discord_token: String,
	/// The postgresql connection uri
	pub database_url: String,
	/// The google auth client id and secret pair
	pub google_client: (ClientId, ClientSecret),

	/// The port to run the server on
	pub port: usize,
	/// Whether or not to use production defaults
	pub production: bool,
}

impl Config {
	/// Parse the config from `.env` file
	fn from_dotenv() -> Self {
		match dotenv() {
			Ok(_) => (),
			Err(_) => {
				panic!("Couldn't find .env file, please create one");
			}
		}

		Self {
			database_url: env::var("DATABASE_URL").expect("DATABASE_URL is not set"),
			discord_token: env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set"),
			google_client: (
				ClientId::new(env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID is not set")),
				ClientSecret::new(
					env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_ID is not set"),
				),
			),
			port: env::var("PORT")
				.unwrap_or_else(|_| "8080".into())
				.parse::<usize>()
				.expect("PORT is not a number"),
			production: env::var("PRODUCTION")
				.unwrap_or_else(|_| "false".into())
				.parse::<bool>()
				.expect("PRODUCTION is not a boolean"),
		}
	}
}

/// App global data
pub struct Data {
	/// An access to the database
	pub database: DatabasePool,
	/// A instance of the auth provider
	pub auth: AuthLink,
	/// An instance of the parsed initial config
	pub config: Config,
	// The discord channel webhook to send logs to
	// pub webhook: WebhookLogs<'static>,
}

impl Data {
	/// Parse the bot data from
	pub fn new() -> Self {
		if DATA_CALLED.swap(true, Ordering::Relaxed) {
			panic!("Data can only be initialized once");
		}

		let config = Config::from_dotenv();

		let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
		let database = Pool::builder()
			.build(manager)
			.expect("failed to create database pool");

		// let http = SerenityHttp::new(&config.discord_token);

		// let logs_webhook_url = env::var("LOGS_WEBHOOK").expect("LOGS_WEBHOOK is not set in .env");
		// let logs_webhook = block_on(http.get_webhook_from_url(&logs_webhook_url))
		// 	.expect("webhook in config file is invalid");

		// let webhook = WebhookLogs::new(http, logs_webhook);

		// Looper::start(Arc::new(webhook));

		Self {
			database,
			auth: AuthLink::new(&config),
			config,
			// webhook,
		}
	}
}

/// Common command return type
pub type CommandResult<E = Error> = Result<(), E>;
/// The poise [`poise::Context`] provided to each command
pub type Context<'a> = poise::Context<'a, &'static Data, Error>;
/// The [`poise::Command`] type alias
pub type _Command = poise::Command<Data, Error>;
/// The [`poise::Framework`] type alias
pub type Framework = poise::Framework<&'static Data, Error>;
/// The [`poise::FrameworkError`] type alias
pub type FrameworkError<'a> = poise::FrameworkError<'a, &'static Data, Error>;
