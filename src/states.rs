//! Handles all the states of the bot and initial configuration

use crate::{database::DatabasePool, handlers::auth::AuthLink};
use anyhow::{Error, Result};
use diesel::{
	r2d2::{ConnectionManager, Pool},
	MysqlConnection,
};
use dotenv::dotenv;
use oauth2::{ClientId, ClientSecret};
use poise::{async_trait, serenity_prelude as serenity, ReplyHandle};
use std::{env, sync::Arc};

/// App global configuration
pub struct Config {
	/// The token needed to access the `Discord` Api
	pub discord_token: String,
	/// The postgresql connection uri
	pub database_url: String,
	/// The google auth client id and secret pair
	pub google_client: (ClientId, ClientSecret),

	/// The url of the oauth2 callback
	pub server_url: String,
	/// The port to run the server on
	pub port: String,
	/// Whether or not to use production defaults
	pub production: bool,
}

impl Config {
	/// Parse the config from `.env` file
	fn from_dotenv() -> Self {
		if dotenv().is_err() {
			panic!("Couldn't find `.env` file, please create one");
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
			server_url: env::var("SERVER_URL").expect("SERVER_URL is not set"),
			port: env::var("PORT").expect("PORT is not set"),
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
}

impl Data {
	/// Parse the bot data from
	pub fn new() -> Self {
		let config = Config::from_dotenv();

		let manager = ConnectionManager::<MysqlConnection>::new(&config.database_url);
		let database = Pool::builder()
			.build(manager)
			.expect("failed to create database pool");

		Self {
			database,
			auth: AuthLink::new(&config),
			config,
		}
	}
}

/// Trait for sending ephemeral messages
#[async_trait]
pub trait Shout: Send + Sync {
	/// Send an ephemeral message to the user
	async fn shout(&self, content: String) -> Result<ReplyHandle<'_>, serenity::Error>;
}

#[async_trait]
impl Shout for Context<'_> {
	async fn shout(&self, content: String) -> Result<ReplyHandle<'_>, serenity::Error> {
		self.send(|builder| {
			builder
				.content(Into::<String>::into(content))
				.ephemeral(true)
		})
		.await
	}
}

/// Common command return type
pub type InteractionResult<E = Error> = Result<(), E>;
/// The poise [`poise::Context`] provided to each command
pub type Context<'a> = poise::Context<'a, Arc<Data>, Error>;
/// The [`poise::Command`] type alias
pub type Command = poise::Command<Data, Error>;
/// The [`poise::Framework`] type alias
pub type Framework = poise::Framework<Arc<Data>, Error>;
/// The [`poise::FrameworkError`] type alias
pub type FrameworkError<'a> = poise::FrameworkError<'a, Arc<Data>, Error>;
