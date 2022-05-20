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
	clippy::missing_docs_in_private_items
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

use events::EventHandler;
use handlers::server::spawn_server;
use logging::setup_logging;
use poise::{serenity_prelude::GatewayIntents, FrameworkBuilder, PrefixFrameworkOptions};
use states::{Context, Data, Framework, STATE};

/// ?
fn run_client() -> FrameworkBuilder<&'static Data, anyhow::Error> {
	let mut client = Framework::build()
		.token(&STATE.config.discord_token)
		.client_settings(move |fw| fw.event_handler(EventHandler {}))
		.intents(
			GatewayIntents::GUILDS
				| GatewayIntents::GUILD_VOICE_STATES
				| GatewayIntents::DIRECT_MESSAGES
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::MESSAGE_CONTENT,
		)
		.user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(&*STATE) }))
		.options(poise::FrameworkOptions {
			pre_command: |ctx| {
				Box::pin(async move {
					log::info!(
						target: "command",
						"{} invoked by {}",
						ctx.invoked_command_name(),
						ctx.author().name,
					);

					if ctx.data().config.production {
						ctx.data().log(|wh| {
							wh.content(format!(
								"Command `{}` invoked by `{}`",
								ctx.invoked_command_name(),
								ctx.author().name,
							))
						});
					}
				})
			},
			prefix_options: PrefixFrameworkOptions {
				prefix: Some(".".into()),
				..Default::default()
			},
			commands: {
				use commands::*;

				vec![
					helpers::register(),
					helpers::reset_global(),
					login::login(),
					login::logout(),
				]
			},
			..Default::default()
		});

	client.initialize_owners(true);

	client
}

#[tokio::main]
async fn main() {
	setup_logging();
	spawn_server();
	run_client().run().await.expect("client crashed");
}
