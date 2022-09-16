//! Handles all the states of the bot and initial configuration

use crate::{database::DatabasePool, handlers::auth::AuthLink, translation::Translations};
use anyhow::Result;
use diesel::{
	r2d2::{ConnectionManager, Pool},
	MysqlConnection,
};
use dotenvy::dotenv;
use oauth2::{ClientId, ClientSecret};
use poise::{
	async_trait, send_application_reply,
	serenity_prelude::{self as serenity},
	CreateReply, ReplyHandle,
};
use std::{
	env::{self},
	sync::Arc,
};
use unic_langid::langid;

/// App global configuration
#[derive(Debug)]
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
#[derive(Debug)]
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

		let translations = Translations::from_folder("translations", langid!("fr"))
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
