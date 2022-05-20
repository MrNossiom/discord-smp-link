//! All the discord events handlers

use crate::states::STATE;
use log::info;
use poise::{
	async_trait,
	serenity_prelude::{
		Context as SerenityContext, EventHandler as SerenityEventHandler, GuildId, Ready,
	},
};

/// Implement serenity's event handler to interact with discord
pub struct EventHandler {}

#[async_trait]
impl SerenityEventHandler for EventHandler {
	async fn ready(&self, _ctx: SerenityContext, bot: Ready) {
		// register_application_commands(ctx, true);
		info!(target: "BOT", "{} is ready!", bot.user.name);

		if STATE.config.production {
			STATE.log(|b| b.content("**SMP Bot** is ready to go in production mode!"));
		}
	}

	async fn cache_ready(&self, _ctx: SerenityContext, guilds: Vec<GuildId>) {
		info!(target: "BOT", "{} guilds cached!", guilds.len());
	}
}
