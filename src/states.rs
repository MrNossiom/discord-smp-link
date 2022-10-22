//! Handles all the states of the bot and initial configuration

use crate::{
	auth::GoogleAuthentification, database::DatabasePool, polyfill, translation::Translations,
};
use anyhow::{anyhow, Context as _};
use diesel_async::{
	pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
	AsyncMysqlConnection,
};
use dotenvy::dotenv;
use oauth2::{ClientId, ClientSecret};
use poise::{
	async_trait, send_application_reply,
	serenity_prelude::{self as serenity},
	CreateReply, ReplyHandle,
};
use secrecy::{ExposeSecret, Secret};
use std::{
	env::{self, VarError},
	fmt,
	sync::Arc,
};
use unic_langid::langid;

/// App global configuration
#[derive(Debug)]
pub(crate) struct Config {
	/// The token needed to access the `Discord` Api
	pub(crate) discord_token: Secret<String>,
	/// The guild on witch you can access development commands
	pub(crate) discord_development_guild: GuildId,
	/// The `Postgres` connection uri
	pub(crate) database_url: Secret<String>,
	/// The `Google` auth client id and secret pair
	pub(crate) google_client: (ClientId, ClientSecret),
	/// The `Discord` invite link to rejoin the support server
	pub(crate) discord_invite_code: String,
	/// The url of the `OAuth2` callback
	pub(crate) server_url: String,

	/// Whether or not to use production defaults
	pub(crate) production: bool,
}

/// Resolve an environment variable or return an appropriate error
fn get_required_env_var(name: &str) -> anyhow::Result<String> {
	match env::var(name) {
		Ok(val) => Ok(val),
		Err(VarError::NotPresent) => Err(anyhow!("{} must be set in the environnement", name)),
		Err(VarError::NotUnicode(_)) => {
			Err(anyhow!("{} does not contains Unicode valid text", name))
		}
	}
}

// TODO: use the `figment` crate
impl Config {
	/// Parse the config from `.env` file
	fn from_dotenv() -> anyhow::Result<Self> {
		// Load the `.env` file ond error if not found
		if dotenv().is_err() {
			return Err(anyhow!("Couldn't find `.env` file, please create one"));
		}

		let discord_invite_code = get_required_env_var("DISCORD_INVITE_CODE")?;

		let discord_development_guild =
			get_required_env_var("DISCORD_DEV_GUILD")?
				.parse::<u64>()
				.map_err(|_| anyhow!("DISCORD_DEV_GUILD environnement variable must be a `u64`"))?;

		let production = env::var("PRODUCTION")
			.unwrap_or_else(|_| "false".into())
			.parse::<bool>()
			.map_err(|_| anyhow!("PRODUCTION environnement variable must be a `bool`"))?;

		Ok(Self {
			discord_token: Secret::new(get_required_env_var("DISCORD_TOKEN")?),
			discord_development_guild: GuildId(discord_development_guild),
			database_url: Secret::new(get_required_env_var("DATABASE_URL")?),
			google_client: (
				ClientId::new(get_required_env_var("GOOGLE_CLIENT_ID")?),
				ClientSecret::new(get_required_env_var("GOOGLE_CLIENT_SECRET")?),
			),
			discord_invite_code,
			server_url: get_required_env_var("SERVER_URL")?,

			production,
		})
	}
}

/// App global data
pub(crate) struct Data {
	/// An access to the database
	pub(crate) database: DatabasePool,
	/// A instance of the auth provider
	pub(crate) auth: GoogleAuthentification,
	/// An instance of the parsed initial config
	pub(crate) config: Config,
	/// The translations for the client
	pub(crate) translations: Translations,
}

impl fmt::Debug for Data {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Data")
			.field("auth", &&self.auth)
			.field("config", &&self.config)
			.field("translations", &&self.translations)
			.finish()
	}
}

impl Data {
	/// Parse the bot data from
	pub(crate) fn new() -> anyhow::Result<Self> {
		let config = Config::from_dotenv()?;

		let manager = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(
			config.database_url.expose_secret(),
		);
		let database = Pool::builder(manager)
			.build()
			.context("failed to create database pool")?;

		// TODO: make the default locale configurable
		let translations = Translations::from_folder("translations", langid!("fr"))
			.context("failed to load translations")?;

		Ok(Self {
			database,
			auth: GoogleAuthentification::new(&config)?,
			config,
			translations,
		})
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
	async fn shout(
		&self,
		content: impl Into<String> + Send,
	) -> Result<ReplyHandle<'_>, serenity::Error>;
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
	async fn shout(
		&self,
		content: impl Into<String> + Send,
	) -> Result<ReplyHandle<'_>, serenity::Error> {
		self.send(|builder| builder.content(content).ephemeral(true))
			.await
	}
}

/// Trait for sending ephemeral messages
#[async_trait]
pub(crate) trait ContextPolyfill: Send + Sync {
	/// Send an ephemeral message to the user
	async fn shout(
		&self,
		content: impl Into<String> + Send,
	) -> Result<ReplyHandle<'_>, serenity::Error>;
}

#[async_trait]
impl ContextPolyfill for Context<'_> {
	#[inline]
	async fn shout(
		&self,
		content: impl Into<String> + Send,
	) -> Result<ReplyHandle<'_>, serenity::Error> {
		self.send(|builder| builder.content(content).ephemeral(true))
			.await
	}
}

/// The [`poise::Command`] type alias
pub(crate) type Command = poise::Command<ArcData, anyhow::Error>;
/// Common command return type
pub(crate) type InteractionResult = anyhow::Result<()>;
/// Common wrapper for the [`Data`]
pub(crate) type ArcData = Arc<Data>;

/// The poise [`poise::Context`] provided to each command
pub(crate) type Context<'a> = poise::Context<'a, ArcData, anyhow::Error>;
/// The poise [`poise::ApplicationContext`] provided to each slash command
pub(crate) type ApplicationContext<'a> = poise::ApplicationContext<'a, ArcData, anyhow::Error>;
/// The local [`polyfill::MessageComponentContext`] provided to each message component interaction
pub(crate) type MessageComponentContext<'a> =
	polyfill::MessageComponentContext<'a, ArcData, anyhow::Error>;
/// The poise [`poise::PrefixContext`] provided to each prefix command
pub(crate) type _PrefixContext<'a> = poise::PrefixContext<'a, ArcData, anyhow::Error>;

/// The [`poise::Framework`] type alias
pub(crate) type Framework = poise::Framework<ArcData, anyhow::Error>;
/// The [`poise::FrameworkContext`] type alias
pub(crate) type FrameworkContext<'a> = poise::FrameworkContext<'a, ArcData, anyhow::Error>;
/// The [`poise::FrameworkError`] type alias
pub(crate) type FrameworkError<'a> = poise::FrameworkError<'a, ArcData, anyhow::Error>;
/// The [`poise::FrameworkBuilder`] type alias
pub(crate) type FrameworkBuilder = poise::FrameworkBuilder<ArcData, anyhow::Error>;
