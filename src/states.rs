//! Handles all the states of the bot and initial configuration

use crate::{auth::GoogleAuthentification, database::DatabasePool, translation::Translations};
use anyhow::{anyhow, Context as _};
use diesel::{
	r2d2::{ConnectionManager, Pool},
	MysqlConnection,
};
use dotenvy::dotenv;
use hyper::Uri;
use oauth2::{ClientId, ClientSecret};
use poise::{
	async_trait, send_application_reply,
	serenity_prelude::{self as serenity},
	CreateReply, ReplyHandle,
};
use secrecy::{ExposeSecret, Secret};
use std::{
	env::{self, VarError},
	fs, io,
	sync::Arc,
};
use unic_langid::langid;

/// HTTPS Certificates for the server
#[derive(Debug, Clone)]
pub(crate) struct Certificates(
	pub(crate) Vec<rustls::Certificate>,
	pub(crate) rustls::PrivateKey,
);

impl Certificates {
	/// Loads the certificates from the certs folder
	fn from_certs_folder() -> anyhow::Result<Self> {
		// TODO
		let certs = Self::load_certs("certs/cert.pem")?;
		let private_key = Self::load_private_key("certs/private.key")?;

		Ok(Self(certs, private_key))
	}

	/// Load public certificate from file.
	fn load_certs(filename: &str) -> io::Result<Vec<rustls::Certificate>> {
		// Open certificate file.
		let certificate_file = fs::File::open(filename).map_err(|e| {
			io::Error::new(
				io::ErrorKind::Other,
				format!("failed to open {}: {}", filename, e),
			)
		})?;
		let mut reader = io::BufReader::new(certificate_file);

		// Load and return certificate.
		let certs = rustls_pemfile::certs(&mut reader)
			.map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to load certificate"))?;
		Ok(certs.into_iter().map(rustls::Certificate).collect())
	}

	/// Load private key from file.
	fn load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
		// Open key file.
		let key_file = fs::File::open(filename).map_err(|e| {
			io::Error::new(
				io::ErrorKind::Other,
				format!("failed to open {}: {}", filename, e),
			)
		})?;
		let mut reader = io::BufReader::new(key_file);

		// Load and return a single private key.
		let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
			.map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to load private key"))?;

		if keys.len() != 1 {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"expected a single private key",
			));
		}

		Ok(rustls::PrivateKey(keys[0].clone()))
	}
}

/// App global configuration
#[derive(Debug)]
pub(crate) struct Config {
	/// The token needed to access the `Discord` Api
	pub(crate) discord_token: Secret<String>,
	/// The `Postgres` connection uri
	pub(crate) database_url: Secret<String>,
	/// The `Google` auth client id and secret pair
	pub(crate) google_client: (ClientId, ClientSecret),
	/// The `Discord` invite link to rejoin the support server
	pub(crate) discord_invite_link: Uri,

	/// The url of the oauth2 callback
	pub(crate) server_url: String,
	/// The port to run the HTTP server on
	pub(crate) port_http: u16,
	/// The port to run the HTTPS server on
	pub(crate) port_https: u16,

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
		let discord_invite_link = format!("https://discord.gg/{}", discord_invite_code)
			.parse()
			.context("DISCORD_INVITE_CODE env must be wrong")?;

		let port_http = env::var("PORT")
			.unwrap_or_else(|_| "80".into())
			.parse::<u16>()
			.map_err(|_| anyhow!("PORT environnement variable must be a `u16`"))?;

		let port_https = env::var("PORT_HTTPS")
			.unwrap_or_else(|_| "443".into())
			.parse::<u16>()
			.map_err(|_| anyhow!("PORT environnement variable must be a `u16`"))?;

		let production = env::var("PRODUCTION")
			.unwrap_or_else(|_| "false".into())
			.parse::<bool>()
			.map_err(|_| anyhow!("PRODUCTION environnement variable must be a `bool`"))?;

		Ok(Self {
			discord_token: Secret::new(get_required_env_var("DISCORD_TOKEN")?),
			database_url: Secret::new(get_required_env_var("DATABASE_URL")?),
			google_client: (
				ClientId::new(get_required_env_var("GOOGLE_CLIENT_ID")?),
				ClientSecret::new(get_required_env_var("GOOGLE_CLIENT_SECRET")?),
			),
			discord_invite_link,

			server_url: get_required_env_var("SERVER_URL")?,
			port_http,
			port_https,

			production,
		})
	}
}

/// App global data
#[derive(Debug)]
pub(crate) struct Data {
	/// An access to the database
	pub(crate) database: DatabasePool,
	/// A instance of the auth provider
	pub(crate) auth: GoogleAuthentification,
	/// An instance of the parsed initial config
	pub(crate) config: Config,
	/// The translations for the client
	pub(crate) translations: Translations,
	/// The HTTPS certificates
	pub(crate) certificates: Certificates,
}

impl Data {
	/// Parse the bot data from
	pub(crate) fn new() -> anyhow::Result<Self> {
		let config = Config::from_dotenv()?;

		let manager =
			ConnectionManager::<MysqlConnection>::new(config.database_url.expose_secret());
		let database = Pool::builder()
			.build(manager)
			.context("failed to create database pool")?;

		// TODO: make the default locale configurable
		let translations = Translations::from_folder("translations", langid!("fr"))
			.context("failed to load translations")?;

		Ok(Self {
			database,
			auth: GoogleAuthentification::new(&config)?,
			config,
			translations,
			certificates: Certificates::from_certs_folder()?,
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
pub(crate) type PrefixContext<'a> = poise::PrefixContext<'a, ArcData, anyhow::Error>;

/// The [`poise::Framework`] type alias
pub(crate) type Framework = poise::Framework<ArcData, anyhow::Error>;
/// The [`poise::FrameworkContext`] type alias
pub(crate) type FrameworkContext<'a> = poise::FrameworkContext<'a, ArcData, anyhow::Error>;
/// The [`poise::FrameworkError`] type alias
pub(crate) type FrameworkError<'a> = poise::FrameworkError<'a, ArcData, anyhow::Error>;
/// The [`poise::FrameworkBuilder`] type alias
pub(crate) type FrameworkBuilder = poise::FrameworkBuilder<ArcData, anyhow::Error>;

// TODO: move elsewhere
#[allow(dead_code)]
mod polyfill {
	//! Polyfill for the [`MessageComponentInteraction`](poise::serenity_prelude::MessageComponentInteraction) type

	use poise::{
		serenity_prelude::{self as serenity, MessageComponentInteraction},
		CreateReply,
	};
	use std::{
		borrow::Cow,
		sync::atomic::{AtomicBool, Ordering},
	};

	/// The [`poise::Context`] like for Message components interactions
	#[derive(Copy, Clone)]
	pub(crate) struct MessageComponentContext<'a, U: Send + Sync, E> {
		/// The underlying interaction
		pub(crate) interaction: &'a MessageComponentInteraction,
		/// The custom user data
		pub(crate) data: &'a U,
		/// The underlying serenity context
		pub(crate) discord: &'a serenity::Context,
		/// Read-only reference to the framework
		///
		/// Useful if you need the list of commands, for example for a custom help command
		pub(crate) framework: poise::FrameworkContext<'a, U, E>,
		/// Keeps track of whether an initial response has been sent.
		///
		/// Discord requires different HTTP endpoints for initial and additional responses.
		pub(crate) has_sent_initial_response: &'a AtomicBool,
	}

	impl<U: Send + Sync, E> MessageComponentContext<'_, U, E> {
		/// Send a message to the user
		pub(crate) async fn send<'a>(
			&'a self,
			builder: impl for<'b> FnOnce(&'b mut CreateReply<'a>) -> &'b mut CreateReply<'a> + Send,
		) -> Result<MessageComponentReplyHandle<'a>, serenity::Error> {
			let mut reply = CreateReply::default();
			builder(&mut reply);

			let has_sent_initial_response = self.has_sent_initial_response.load(Ordering::SeqCst);

			let followup = if has_sent_initial_response {
				Some(Box::new(
					self.interaction
						.create_followup_message(self.discord, |f| {
							reply.to_slash_followup_response(f);
							f
						})
						.await?,
				))
			} else {
				self.interaction
					.create_interaction_response(self.discord, |r| {
						r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
							.interaction_response_data(|f| {
								reply.to_slash_initial_response(f);
								f
							})
					})
					.await?;
				self.has_sent_initial_response
					.store(true, std::sync::atomic::Ordering::SeqCst);

				None
			};

			// ReplyHandle contains private fields, so we can't construct nor return it
			// We use our own copy of ReplyHandle
			Ok(MessageComponentReplyHandle {
				http: &self.discord.http,
				interaction: self.interaction,
				followup,
			})
		}

		/// Send an ephemeral message to the user
		#[inline]
		pub(crate) async fn shout(
			&self,
			content: impl Into<String> + Send,
		) -> Result<MessageComponentReplyHandle<'_>, serenity::Error> {
			self.send(|builder| builder.content(content.into()).ephemeral(true))
				.await
		}
	}

	/// Returned from [`MessageComponentContext::send()`] to operate on the sent message
	///
	/// Discord sometimes returns the [`serenity::Message`] object directly, but sometimes you have to
	/// request it manually. This enum abstracts over the two cases
	#[derive(Clone)]
	pub(crate) struct MessageComponentReplyHandle<'a> {
		/// Serenity HTTP instance that can be used to request the interaction response message
		/// object
		http: &'a serenity::Http,
		/// Interaction which contains the necessary data to request the interaction response
		/// message object
		interaction: &'a serenity::MessageComponentInteraction,
		/// If this is a followup response, the Message object (which Discord only returns for
		/// followup responses, not initial)
		followup: Option<Box<serenity::Message>>,
	}

	impl MessageComponentReplyHandle<'_> {
		/// Retrieve the message object of the sent reply.
		///
		/// If you don't need ownership of Message, you can use [`Self::message`]
		///
		/// Only needs to do an HTTP request in the application command response case
		pub(crate) async fn into_message(self) -> Result<serenity::Message, serenity::Error> {
			self.interaction.get_interaction_response(self.http).await
		}

		/// Retrieve the message object of the sent reply.
		///
		/// Returns a reference to the known Message object, or fetches the message from the discord API.
		///
		/// To get an owned [`serenity::Message`], use [`Self::into_message()`]
		pub(crate) async fn message(&self) -> Result<Cow<'_, serenity::Message>, serenity::Error> {
			Ok(Cow::Owned(
				self.interaction.get_interaction_response(self.http).await?,
			))
		}

		/// Edits the message that this [`Self`] points to
		pub(crate) async fn edit<'att>(
			&self,
			builder: impl for<'a> FnOnce(&'a mut CreateReply<'att>) -> &'a mut CreateReply<'att> + Send,
		) -> Result<(), serenity::Error> {
			let mut reply = poise::CreateReply::default();
			builder(&mut reply);

			if let Some(followup) = &self.followup {
				self.interaction
					.edit_followup_message(self.http, followup.id, |b| {
						reply.to_slash_followup_response(b);
						b
					})
					.await?;
			} else {
				self.interaction
					.edit_original_interaction_response(self.http, |b| {
						reply.to_slash_initial_response_edit(b);
						b
					})
					.await?;
			}

			Ok(())
		}
	}
}
