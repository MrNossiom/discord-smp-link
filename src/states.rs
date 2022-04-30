use std::sync::Arc;

use crate::handlers::auth::AuthLink;
use anyhow::{Error, Result};
use diesel::{
	r2d2::{ConnectionManager, Pool},
	PgConnection,
};
use oauth2::{ClientId, ClientSecret};
use poise::{Command as PoiseCommand, Context as PoiseContext, Framework as PoiseFramework};
use serde::Deserialize;

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
}

impl Default for Data {
	fn default() -> Self {
		let config: Config =
			ron::from_str(include_str!("../Config.ron")).expect("Config.ron is invalid");

		let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
		let database = Pool::builder()
			.build(manager)
			.expect("failed to create database pool");

		Data {
			database,
			auth: AuthLink::new(&config),
			config,
		}
	}
}

pub type State = Arc<Data>;

// Discord framework structs
pub type CommandError = Error;
pub type CommandResult<E = Error> = Result<(), E>;
pub type Context<'a> = PoiseContext<'a, Data, Error>;
pub type _Command = PoiseCommand<Data, CommandError>;
pub type Framework = PoiseFramework<Data, CommandError>;
