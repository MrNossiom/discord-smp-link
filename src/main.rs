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
mod constants;
mod database;
mod events;
mod handlers;
mod logging;
mod states;
mod translation;

use crate::{
	commands::{command_on_error, post_command, pre_command},
	database::run_migrations,
	events::event_handler,
	handlers::server::spawn_server,
	logging::setup_logging,
	states::{Data, Framework, FrameworkBuilder},
};
use poise::serenity_prelude::GatewayIntents;
use std::{process::ExitCode, sync::Arc};

/// Build the `poise` [framework](poise::Framework)
fn build_client(data: Arc<Data>) -> FrameworkBuilder {
	Framework::builder()
		.token(&data.config.discord_token)
		.intents(
			GatewayIntents::GUILDS
				| GatewayIntents::GUILD_VOICE_STATES
				| GatewayIntents::DIRECT_MESSAGES
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::MESSAGE_CONTENT,
		)
		.user_data_setup({
			let data = Arc::clone(&data);
			move |_ctx, _ready, _framework| Box::pin(async move { Ok(data) })
		})
		.options(poise::FrameworkOptions {
			pre_command,
			on_error: command_on_error,
			post_command,
			listener: |ctx, event, fw, data| {
				Box::pin(async move { event_handler(ctx, event, fw, data).await })
			},
			prefix_options: Default::default(),
			commands: {
				use commands::*;

				#[rustfmt::skip]
				let mut commands = vec![
					setup(),
					information(),
					helpers::debug(),
				];

				data.translations
					.apply_translations_to_interactions(&mut commands, None);

				commands.push(helpers::_register());

				commands
			},
			..Default::default()
		})
		.initialize_owners(true)
}

#[tokio::main]
async fn main() -> ExitCode {
	let data = Arc::new(Data::new());

	run_migrations(&mut data.database.get().expect("failed to get a connection"))
		.expect("failed to run migrations");

	let _guard = setup_logging(Arc::clone(&data));
	let _handle = spawn_server(Arc::clone(&data));

	if let Err(error) = build_client(Arc::clone(&data)).run().await {
		tracing::error!("Client exited with error: {}", error);

		return ExitCode::FAILURE;
	}

	ExitCode::SUCCESS
}
