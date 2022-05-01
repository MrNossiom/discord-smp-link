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
use std::sync::Arc;

#[derive(Deserialize)]
pub struct Config {
	pub discord_token: String,
	pub database_url: String,
	pub google_client: (ClientId, ClientSecret),

	pub logs_webhook: String,

	pub port: usize,
	pub production: bool,
	pub log_level: String,
}

pub struct Data {
	pub database: Pool<ConnectionManager<PgConnection>>,
	pub auth: AuthLink,
	pub config: Config,
	pub logs_webhook: Webhook,
	http: SerenityHttp,
}

impl Data {
	pub async fn new() -> Self {
		let config: Config =
			ron::from_str(include_str!("../Config.ron")).expect("Config.ron is invalid");

		let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
		let database = Pool::builder()
			.build(manager)
			.expect("failed to create database pool");

		let http = SerenityHttp::new(&config.discord_token);

		let logs_webhook = http
			.get_webhook_from_url(&config.logs_webhook)
			.await
			.unwrap();

		Data {
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

pub type State = Arc<Data>;

// Discord framework structs
pub type CommandError = Error;
pub type CommandResult<E = Error> = Result<(), E>;
pub type Context<'a> = PoiseContext<'a, Arc<Data>, Error>;
pub type _Command = PoiseCommand<Data, CommandError>;
pub type Framework = PoiseFramework<Arc<Data>, CommandError>;
