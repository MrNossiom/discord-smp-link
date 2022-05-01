use std::sync::Arc;

use crate::states::Data;
use label_logger::info;
use poise::{
	async_trait,
	serenity_prelude::{
		Context as SerenityContext, EventHandler as SerenityEventHandler, GuildId, Ready,
	},
};

pub struct EventHandler {
	pub state: Arc<Data>,
}

#[async_trait]
impl SerenityEventHandler for EventHandler {
	async fn ready(&self, _ctx: SerenityContext, bot: Ready) {
		// register_application_commands(ctx, true);
		info!("{} is ready!", bot.user.name);

		self.state
			.log(|b| b.content("**SMP Bot** is ready to go!"))
			.await
			.unwrap();
	}

	async fn cache_ready(&self, _ctx: SerenityContext, guilds: Vec<GuildId>) {
		info!("{} guilds cached!", guilds.len());
	}
}
