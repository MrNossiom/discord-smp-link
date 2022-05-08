//! All the discord events handlers

use crate::states::Data;
use label_logger::{info, success};
use poise::{
	async_trait,
	serenity_prelude::{
		Context as SerenityContext, EventHandler as SerenityEventHandler, GuildId, Ready,
	},
};
use std::sync::Arc;

/// Implement serenity's event handler to interact with discord
pub struct EventHandler {
	/// A handle to the bot data
	pub state: Arc<Data>,
}

#[async_trait]
impl SerenityEventHandler for EventHandler {
	async fn ready(&self, _ctx: SerenityContext, bot: Ready) {
		// register_application_commands(ctx, true);
		success!(label: "Bot", "{} is ready!", bot.user.name);

		if self.state.config.production {
			self.state
				.log(|b| b.content("**SMP Bot** is ready to go in production mode!"))
				.await
				.unwrap();
		}
	}

	async fn cache_ready(&self, _ctx: SerenityContext, guilds: Vec<GuildId>) {
		info!(label: "Bot", "{} guilds cached!", guilds.len());
	}
}
