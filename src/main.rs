mod commands;
mod database;
mod events;
mod handlers;
mod states;

#[macro_use]
extern crate diesel;

use dotenv::dotenv;
use events::event_listener;
use handlers::auth::AuthLink;
use poise::{serenity_prelude::*, Framework, PrefixFrameworkOptions};
use states::{Context, Data};
use std::env;

#[tokio::main]
async fn main() {
	dotenv().ok();

	let database = database::establish_connection();
	let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

	Framework::build()
		.token(&token)
		.client_settings(move |f| {
			f.intents(
				GatewayIntents::GUILDS
					| GatewayIntents::GUILD_VOICE_STATES
					| GatewayIntents::DIRECT_MESSAGES
					| GatewayIntents::GUILD_MESSAGES,
			)
		})
		.user_data_setup(move |_ctx, _ready, _framework| {
			Box::pin(async move {
				Ok(Data {
					database,
					auth: AuthLink::new().await,
				})
			})
		})
		.options(poise::FrameworkOptions {
			pre_command: |ctx| {
				Box::pin(async move {
					println!(
						"{} invoked {}",
						ctx.author().name,
						ctx.invoked_command_name()
					);
				})
			},
			prefix_options: PrefixFrameworkOptions {
				mention_as_prefix: true,
				..Default::default()
			},
			listener: |ctx, event, fw, ud| Box::pin(event_listener(ctx, event, fw, ud)),
			commands: vec![commands::register(), commands::login()],
			..Default::default()
		})
		.run()
		.await
		.unwrap();
}
