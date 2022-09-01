//! Handles all the states of the bot and initial configuration

use crate::{database::DatabasePool, handlers::auth::AuthLink, translation::Translations};
use anyhow::Result;
use diesel::{
	r2d2::{ConnectionManager, Pool},
	MysqlConnection,
};
use dotenv::dotenv;
use oauth2::{ClientId, ClientSecret};
use poise::{
	async_trait, send_application_reply, serenity_prelude as serenity, CreateReply, ReplyHandle,
};
use std::{env, sync::Arc};
use unic_langid::langid;

/// App global configuration
pub(crate) struct Config {
	/// The token needed to access the `Discord` Api
	pub(crate) discord_token: String,
	/// The Postgres connection uri
	pub(crate) database_url: String,
	/// The google auth client id and secret pair
	pub(crate) google_client: (ClientId, ClientSecret),

	/// The url of the oauth2 callback
	pub(crate) server_url: String,
	/// The port to run the server on
	pub(crate) port: String,
	/// Whether or not to use production defaults
	pub(crate) production: bool,
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
pub(crate) struct Data {
	/// An access to the database
	pub(crate) database: DatabasePool,
	/// A instance of the auth provider
	pub(crate) auth: AuthLink,
	/// An instance of the parsed initial config
	pub(crate) config: Config,
	/// The translations for the client
	pub(crate) translations: Translations,
}

impl Data {
	/// Parse the bot data from
	pub(crate) fn new() -> Self {
		let config = Config::from_dotenv();

		let manager = ConnectionManager::<MysqlConnection>::new(&config.database_url);
		let database = Pool::builder()
			.build(manager)
			.expect("failed to create database pool");

		let translations = Translations::from_folder("translations", langid!("en-US"))
			.expect("failed to load translations");

		Self {
			database,
			auth: AuthLink::new(&config),
			config,
			translations,
		}
	}
}

/// Trait for sending ephemeral messages
#[async_trait]
pub(crate) trait ApplicationContextPolyfill<'b>: Send + Sync {
	/// Send an ephemeral message to the user
	async fn send<'att>(
		self,
		builder: impl for<'a> FnOnce(&'a mut CreateReply<'att>) -> &'a mut CreateReply<'att> + Send,
	) -> Result<ReplyHandle<'b>, serenity::Error>;

	/// Send an ephemeral message to the user
	async fn shout(&self, content: String) -> Result<ReplyHandle<'_>, serenity::Error>;
}

#[async_trait]
impl<'b> ApplicationContextPolyfill<'b> for ApplicationContext<'b> {
	/// Send an ephemeral message to the user
	#[inline]
	async fn send<'att>(
		self,
		builder: impl for<'a> FnOnce(&'a mut CreateReply<'att>) -> &'a mut CreateReply<'att> + Send,
	) -> Result<ReplyHandle<'b>, serenity::Error> {
		send_application_reply(self, builder).await
	}

	#[inline]
	async fn shout(&self, content: String) -> Result<ReplyHandle<'_>, serenity::Error> {
		self.send(|builder| builder.content(content).ephemeral(true))
			.await
	}
}

/// Trait for sending ephemeral messages
#[async_trait]
pub(crate) trait ContextPolyfill: Send + Sync {
	/// Send an ephemeral message to the user
	async fn shout(&self, content: String) -> Result<ReplyHandle<'_>, serenity::Error>;
}

#[async_trait]
impl ContextPolyfill for Context<'_> {
	#[inline]
	async fn shout(&self, content: String) -> Result<ReplyHandle<'_>, serenity::Error> {
		self.send(|builder| builder.content(content).ephemeral(true))
			.await
	}
}

/// Common command return type
pub(crate) type InteractionResult = anyhow::Result<()>;
/// The poise [`poise::Context`] provided to each command
pub(crate) type Context<'a> = poise::Context<'a, Arc<Data>, anyhow::Error>;
/// The poise [`poise::ApplicationContext`] provided to each slash command
pub(crate) type ApplicationContext<'a> = poise::ApplicationContext<'a, Arc<Data>, anyhow::Error>;
/// The poise [`poise::PrefixContext`] provided to each prefix command
pub(crate) type PrefixContext<'a> = poise::PrefixContext<'a, Arc<Data>, anyhow::Error>;
/// The [`poise::Command`] type alias
pub(crate) type Command = poise::Command<Arc<Data>, anyhow::Error>;
/// The [`poise::Framework`] type alias
pub(crate) type Framework = poise::Framework<Arc<Data>, anyhow::Error>;
/// The [`poise::FrameworkContext`] type alias
pub(crate) type FrameworkContext<'a> = poise::FrameworkContext<'a, Arc<Data>, anyhow::Error>;
/// The [`poise::FrameworkError`] type alias
pub(crate) type FrameworkError<'a> = poise::FrameworkError<'a, Arc<Data>, anyhow::Error>;
/// The [`poise::FrameworkBuilder`] type alias
pub(crate) type FrameworkBuilder = poise::FrameworkBuilder<Arc<Data>, anyhow::Error>;
