//! Setup messages for roles interactions

use crate::{
	constants::events::{LOGIN_BUTTON_INTERACTION, LOGOUT_BUTTON_INTERACTION},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use anyhow::anyhow;
use poise::{command, serenity_prelude::component::ButtonStyle};

/// Sets the login and logout message.
#[command(
	slash_command,
	guild_only,
	default_member_permissions = "ADMINISTRATOR"
)]
pub(crate) async fn message(ctx: ApplicationContext<'_>) -> InteractionResult {
	let mut connection = ctx.data.database.get()?;
	let guild_id = ctx
		.interaction
		.guild_id()
		.ok_or_else(|| anyhow!("guild only command"))?;

	let verified_role_was_registered = {
		use crate::database::schema::guilds;
		use diesel::prelude::*;

		let role: Option<u64> = guilds::table
			.filter(guilds::id.eq(guild_id.0))
			.select(guilds::verified_role_id)
			.first(&mut connection)?;

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
	{
		use crate::database::schema::guilds::dsl::{guilds, setup_message_id};
		use diesel::prelude::*;

		diesel::update(guilds.find(guild_id.0))
			.set(setup_message_id.eq(reply.id.0))
			.execute(&mut connection)?;
	}

	Ok(())
}
