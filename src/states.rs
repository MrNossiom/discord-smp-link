//! Handles all the states of the bot and initial configuration

use crate::handlers::auth::AuthLink;
use anyhow::{Error, Result};
use diesel::{
	r2d2::{ConnectionManager, Pool},
	PgConnection,
};
use dotenv::dotenv;
use oauth2::{ClientId, ClientSecret};
use poise::{
	serenity_prelude::{
		ExecuteWebhook, Http as SerenityHttp, Message, Result as SerenityResult, Webhook,
	},
	Command as PoiseCommand, Context as PoiseContext, Framework as PoiseFramework,
};
use std::{env, sync::Arc};

/// The initial config of the bot
pub struct Config {
	/// The token needed to access the discord api
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

/// The data that is passed to the framework
pub struct Data {
	/// An access to the database
	pub database: Pool<ConnectionManager<PgConnection>>,
	/// A instance of the auth provider
	pub auth: AuthLink,
	/// An instance of the parsed initial config
	pub config: Config,

	/// The discord channel webhook to send logs to
	pub logs_webhook: Webhook,

	/// A http client to make discord requests
	http: SerenityHttp,
}

impl Data {
	/// Parse the bot data from
	pub async fn new() -> Self {
		let config = Config::from_dotenv();

		let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
		let database = Pool::builder()
			.build(manager)
			.expect("failed to create database pool");

		let http = SerenityHttp::new(&config.discord_token);

		let logs_webhook_url = env::var("LOGS_WEBHOOK").expect("LOGS_WEBHOOK is not set in .env");
		let logs_webhook = http
			.get_webhook_from_url(&logs_webhook_url)
			.await
			.expect("webhook in config file is invalid");

		Self {
			database,
			auth: AuthLink::new(&config),
			config,
			logs_webhook,
			http,
		}
	}

	/// Sends a message to the discord log webhook
	pub async fn log<'a, F>(&self, func: F) -> SerenityResult<Option<Message>>
	where
		for<'b> F: FnOnce(&'b mut ExecuteWebhook<'a>) -> &'b mut ExecuteWebhook<'a>,
	{
		self.logs_webhook.execute(&self.http, false, func).await
	}
}

/// Common command result
pub type CommandResult<E = Error> = Result<(), E>;
/// The context provided to each command
pub type Context<'a> = PoiseContext<'a, Arc<Data>, Error>;
/// The command type alias
pub type _Command = PoiseCommand<Data, Error>;
/// The framework type alias
pub type Framework = PoiseFramework<Arc<Data>, Error>;
