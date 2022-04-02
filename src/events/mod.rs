use crate::states::Data;
use poise::{serenity_prelude::*, Event, Framework};

pub async fn event_listener(
	_ctx: &Context,
	event: &Event<'_>,
	_framework: &Framework<Data, Error>,
	_data: &Data,
) -> Result<()> {
	match event {
		Event::Ready { data_about_bot } => {
			// register_application_commands(ctx, true);
			println!("{} is ready!", data_about_bot.user.name);
		}
		Event::CacheReady { guilds } => {
			println!("{} guilds cached!", guilds.len());
		}
		_ => {}
	}

	Ok(())
}
