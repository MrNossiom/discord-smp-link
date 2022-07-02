#![warn(
	clippy::unwrap_used,
	clippy::str_to_string,
	clippy::suspicious_operation_groupings,
	clippy::todo,
	clippy::too_many_lines,
	clippy::unicode_not_nfc,
	clippy::unused_async,
	clippy::use_self,
	clippy::dbg_macro,
	clippy::doc_markdown,
	clippy::else_if_without_else,
	clippy::future_not_send,
	clippy::implicit_clone,
	clippy::match_bool,
	clippy::missing_panics_doc,
	clippy::redundant_closure_for_method_calls,
	clippy::redundant_else,
	clippy::must_use_candidate,
	clippy::return_self_not_must_use,
	clippy::missing_docs_in_private_items,
	rustdoc::broken_intra_doc_links
)]

//! Discord SMP Bot

mod commands;
mod database;
mod events;
mod handlers;
mod logging;
mod states;

#[macro_use]
extern crate diesel;

use commands::{command_on_error, post_command, pre_command};
use events::event_handler;
use handlers::server::spawn_server;
use logging::setup_logging;
use poise::{serenity_prelude::GatewayIntents, FrameworkBuilder, PrefixFrameworkOptions};
use states::{Context, Data, Framework, STATE};
use std::process::ExitCode;

/// Build the `poise` [framework](poise::Framework)
fn build_client() -> FrameworkBuilder<&'static Data, anyhow::Error> {
	let mut client = Framework::build()
		.token(&STATE.config.discord_token)
		.intents(
			GatewayIntents::GUILDS
				| GatewayIntents::GUILD_VOICE_STATES
				| GatewayIntents::DIRECT_MESSAGES
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::MESSAGE_CONTENT,
		)
		.user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(&*STATE) }))
		.options(poise::FrameworkOptions {
			pre_command,
			on_error: command_on_error,
			post_command,
			listener: |ctx, event, fw, data| {
				Box::pin(async move { event_handler(ctx, event, fw, data) })
			},
			prefix_options: PrefixFrameworkOptions {
				prefix: Some(".".into()),
				..Default::default()
			},
			commands: {
				use commands::*;

				vec![
					helpers::help(),
					helpers::register(),
					helpers::reset_global(),
					login::login(),
					login::logout(),
					setup(),
				]
			},
			..Default::default()
		});

	client.initialize_owners(true);

	client
}

#[tokio::main]
async fn main() -> ExitCode {
	setup_logging();
	log::trace!("Logging setup");

	spawn_server();
	log::trace!("Server has spawned");

	if let Err(error) = build_client().run().await {
		log::error!("Client exited with error: {}", error);

		return ExitCode::FAILURE;
	}

	ExitCode::SUCCESS
}
