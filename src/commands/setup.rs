//! Setup messages for roles interactions

use crate::{
	constants::events::{LOGIN_BUTTON_INTERACTION, LOGOUT_BUTTON_INTERACTION},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use diesel::update;
use poise::{command, serenity_prelude::component::ButtonStyle};

#[allow(clippy::missing_docs_in_private_items)]
#[command(
	slash_command,
	guild_only,
	default_member_permissions = "ADMINISTRATOR"
)]
pub(crate) async fn setup(ctx: ApplicationContext<'_>) -> InteractionResult {
	let reply = ctx
		.interaction
		.channel_id()
		.send_message(ctx.discord, |m| {
			m.content(ctx.get("setup-message", None)).components(|com| {
				com.create_action_row(|row| {
					row.create_button(|butt| {
						butt.label(ctx.get("setup-button-login", None))
							.style(ButtonStyle::Success)
							.custom_id(LOGIN_BUTTON_INTERACTION)
					})
					.create_button(|butt| {
						butt.label(ctx.get("setup-button-logout", None))
							.style(ButtonStyle::Danger)
							.custom_id(LOGOUT_BUTTON_INTERACTION)
					})
				})
			})
		})
		.await?;

	{
		let get = ctx.get("done", None);
		ctx.shout(get).await?;
	}

	// Update the setup message id
	{
		use crate::database::schema::guilds::dsl::{guilds, setup_message_id};
		use diesel::prelude::*;

		update(guilds.find(ctx.interaction.guild_id().expect("command is guild_only").0))
			.set(setup_message_id.eq(reply.id.0))
			.execute(&mut ctx.data.database.get()?)?;
	}

	Ok(())
}
