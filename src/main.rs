#![feature(let_chains)]
#![warn(
	clippy::missing_docs_in_private_items,
	clippy::unwrap_used,
	clippy::nursery,
	clippy::pedantic,
	clippy::cargo,
	rustdoc::broken_intra_doc_links
)]
#![allow(clippy::redundant_pub_crate, clippy::multiple_crate_versions)]

//! Discord SMP Bot

mod auth;
mod commands;
mod constants;
mod database;
mod events;
mod logging;
mod polyfill;
mod server;
mod states;
mod translation;

use crate::{
	commands::{command_on_error, post_command, pre_command},
	database::run_migrations,
	events::event_handler,
	logging::setup_logging,
	server::start_server,
	states::{ArcData, Data, Framework, FrameworkBuilder},
};
use anyhow::{anyhow, Context};
use poise::serenity_prelude::GatewayIntents;
use secrecy::ExposeSecret;
use std::sync::Arc;
use tracing::instrument;

/// Build the `poise` [framework](poise::Framework)
#[instrument]
fn build_client(data: ArcData) -> FrameworkBuilder {
	Framework::builder()
		.token(data.config.discord_token.expose_secret())
		.intents(
			GatewayIntents::GUILDS
				| GatewayIntents::GUILD_VOICE_STATES
				| GatewayIntents::DIRECT_MESSAGES
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::GUILD_MEMBERS,
		)
		.setup({
			let data = Arc::clone(&data);
			move |_ctx, _ready, _framework| Box::pin(async move { Ok(data) })
		})
		.options(poise::FrameworkOptions {
			pre_command,
			on_error: command_on_error,
			post_command,
			event_handler: |ctx, event, fw, data| {
				Box::pin(async move { event_handler(ctx, event, fw, data).await })
			},
			commands: {
				use commands::{classes, groups, helpers, information, levels, setup};

				#[rustfmt::skip]
				let mut commands = vec![
					setup(),
					levels(),
					classes(),
					groups(),
					information(),
					helpers::debug(),
				];

				data.translations
					.apply_translations_to_interactions(&mut commands, &None);

				commands
			},
			..Default::default()
		})
		.initialize_owners(true)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let data = Arc::new(Data::new()?);

	setup_logging(&data)?;
	let _handle = start_server(Arc::clone(&data))?;

	run_migrations(data.config.database_url.expose_secret()).context("failed to run migrations")?;

	if let Err(error) = build_client(Arc::clone(&data)).run().await {
		return Err(anyhow!("Client exited with error: {}", error));
	}

	Ok(())
}
