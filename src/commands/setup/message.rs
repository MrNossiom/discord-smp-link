//! Setup messages for roles interactions

use crate::{
	constants::events::{LOGIN_BUTTON_INTERACTION, LOGOUT_BUTTON_INTERACTION},
	database::{models::Guild, prelude::*, schema},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use anyhow::anyhow;
use poise::{command, serenity_prelude::component::ButtonStyle};

/// Sets the login and logout message.
#[command(slash_command, guild_only, rename = "message")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn setup_message(ctx: ApplicationContext<'_>) -> InteractionResult {
	let mut connection = ctx.data.database.get().await?;
	let guild_id = ctx
		.interaction
		.guild_id()
		.ok_or_else(|| anyhow!("guild only command"))?;

	let verified_role_was_registered = {
		let role: Option<u64> = Guild::with_id(&guild_id)
			.select(schema::guilds::verified_role_id)
			.first(&mut connection)
			.await?;

		role.is_some()
	};

	if !verified_role_was_registered {
		ctx.shout("You must first use the `/setup role` command")
			.await?;
	}

	// TODO: use guild locale or interaction locale as fallback

	let reply = ctx
		.interaction
		.channel_id()
		.send_message(ctx.discord, |m| {
			m.content(ctx.get("setup-message-message", None))
				.components(|com| {
					com.create_action_row(|row| {
						row.create_button(|butt| {
							butt.label(ctx.get("event-setup-login-button", None))
								.style(ButtonStyle::Success)
								.custom_id(LOGIN_BUTTON_INTERACTION)
						})
						.create_button(|butt| {
							butt.label(ctx.get("event-setup-logout-button", None))
								.style(ButtonStyle::Danger)
								.custom_id(LOGOUT_BUTTON_INTERACTION)
						})
					})
				})
		})
		.await?;

	// Update the `setup_message_id`
	diesel::update(schema::guilds::table.find(guild_id.0))
		.set(schema::guilds::setup_message_id.eq(reply.id.0))
		.execute(&mut connection)
		.await?;

	Ok(())
}
