//! `Discord` client events handlers

use poise::{
	async_trait,
	serenity_prelude::{Context as SerenityContext, EventHandler as SerenityEventHandler, Ready},
};

/// Implement Serenity's [`EventHandler`] to react to `Discord` events
pub struct EventHandler {}

#[async_trait]
impl SerenityEventHandler for EventHandler {
	async fn ready(&self, _ctx: SerenityContext, bot: Ready) {
		// register_application_commands(ctx, true);
		log::info!(target: "bot", "{} is ready!", bot.user.name);
	}
}
