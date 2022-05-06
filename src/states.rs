//! Handles all the states of the bot and initial configuration

use crate::handlers::auth::AuthLink;
use anyhow::{Error, Result};
use diesel::{
	r2d2::{ConnectionManager, Pool},
	PgConnection,
};
use oauth2::{ClientId, ClientSecret};
use poise::{
	serenity_prelude::{
		ExecuteWebhook, Http as SerenityHttp, Message, Result as SerenityResult, Webhook,
	},
	Command as PoiseCommand, Context as PoiseContext, Framework as PoiseFramework,
};
use serde::Deserialize;
use std::{fs::read_to_string, sync::Arc};

/// The initial config of the bot
#[derive(Deserialize)]
pub struct Config {
	/// The token needed to access the discord api
	pub discord_token: String,
	/// The postgres connection uri
	pub database_uri: String,
	/// The google auth client id and secret pair
	pub google_client: (ClientId, ClientSecret),

	/// The discord channel webhook to send logs to
	pub logs_webhook: String,

	/// The port to run the server on
	pub port: usize,
	/// Whether or not to use production defaults
	pub production: bool,
}

impl Config {
	/// Parse config from `Config.ron`
	fn from_config_file() -> Self {
		let data = read_to_string("./Config.ron").expect("Config.ron doesn't exist");

		ron::from_str(data.as_str()).expect("Config.ron is invalid")
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
	pub logs_webhook: Webhook,
	http: SerenityHttp,
}

impl Data {
	pub async fn new() -> Self {
		let config = Config::from_config_file();

		let manager = ConnectionManager::<PgConnection>::new(&config.database_uri);
		let database = Pool::builder()
			.build(manager)
			.expect("failed to create database pool");

		let http = SerenityHttp::new(&config.discord_token);

		let logs_webhook = http
			.get_webhook_from_url(&config.logs_webhook)
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
