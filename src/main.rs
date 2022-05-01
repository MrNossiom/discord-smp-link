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
use poise::{serenity_prelude::GatewayIntents, PrefixFrameworkOptions};
use states::{Context, Data, Framework};
use std::sync::Arc;

#[tokio::main]
async fn main() {
	let state = Arc::new(Data::new().await);
	launch_server(state.config.port, Arc::clone(&state));

	let event_handler_state = Arc::clone(&state);

	let mut client = Framework::build()
		.token(&state.config.discord_token)
		.client_settings(move |fw| {
			fw.event_handler(EventHandler {
				state: event_handler_state,
			})
		})
		.intents(
			GatewayIntents::GUILDS
				| GatewayIntents::GUILD_VOICE_STATES
				| GatewayIntents::DIRECT_MESSAGES
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::MESSAGE_CONTENT,
		)
		.user_data_setup(move |_ctx, _ready, _framework| {
			Box::pin(async move { Ok(Arc::clone(&state)) })
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

					ctx.data()
						.log(|wh| {
							wh.content(format!(
								"Command `{}` invoked by `{}`",
								ctx.invoked_command_name(),
								ctx.author().name,
							))
						})
						.await
						.unwrap();
				})
			},
			prefix_options: PrefixFrameworkOptions {
				mention_as_prefix: true,
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
					test::db(),
				]
			},
			..Default::default()
		});

	client.initialize_owners(true);

	client.run().await.unwrap();
}
