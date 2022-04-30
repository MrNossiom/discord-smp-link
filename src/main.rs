mod commands;
mod database;
mod events;
mod handlers;
mod states;

#[macro_use]
extern crate diesel;

use events::EventHandler;
use handlers::server::launch_server;
use label_logger::info;
use poise::{
	serenity_prelude::{GatewayIntents, UserId},
	PrefixFrameworkOptions,
};
use states::{Context, Data, Framework};
use std::{collections::HashSet, sync::Arc};

#[tokio::main]
async fn main() {
	let state = Arc::new(Data::default());
	launch_server(state.config.port, Arc::clone(&state));

	Framework::build()
		.token(&state.config.discord_token)
		.client_settings(move |fw| fw.event_handler(EventHandler))
		.intents(
			GatewayIntents::GUILDS
				| GatewayIntents::GUILD_VOICE_STATES
				| GatewayIntents::DIRECT_MESSAGES
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::MESSAGE_CONTENT,
		)
		.user_data_setup(move |_ctx, _ready, _framework| {
			Box::pin(async move { Ok(Data::default()) })
		})
		.options(poise::FrameworkOptions {
			pre_command: |ctx| {
				Box::pin(async move {
					info!(
						label: "Command",
						"{} invoked by {}",
						ctx.invoked_command_name(),
						ctx.author().name,
					);
				})
			},
			prefix_options: PrefixFrameworkOptions {
				mention_as_prefix: true,
				prefix: Some(".".into()),
				..Default::default()
			},
			owners: {
				let mut hs = HashSet::new();
				hs.insert(UserId(414017710091927552));
				hs
			},
			commands: {
				use commands::*;

				vec![
					helpers::register(),
					helpers::reset_global(),
					login::login(),
					login::logout(),
					test::db(),
				]
			},
			..Default::default()
		})
		.run()
		.await
		.unwrap();
}
