//! Setup messages for roles interactions

use crate::{
	constants::events::{LOGIN_BUTTON_INTERACTION, LOGOUT_BUTTON_INTERACTION},
	database::{models::Guild, prelude::*, schema},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use poise::{
	command,
	serenity_prelude::{ButtonStyle, CreateActionRow, CreateButton, CreateMessage},
};

/// Sets the login and logout message.
#[command(slash_command, guild_only, rename = "login_message")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user.id))]
pub(crate) async fn setup_login_message(ctx: ApplicationContext<'_>) -> InteractionResult {
	let mut connection = ctx.data.database.get().await?;
	let guild_id = ctx.guild_only_id();

	let verified_role_was_registered = {
		let role: Option<u64> = Guild::with_id(guild_id)
			.select(schema::guilds::verified_role_id)
			.first(&mut connection)
			.await?;

		role.is_some()
	};

	if !verified_role_was_registered {
		ctx.shout("You must first use the `/setup role` command")
			.await?;

		return Ok(());
	}

	// TODO: use guild locale or interaction locale as fallback

	let action_row = CreateActionRow::Buttons(vec![
		CreateButton::new(LOGIN_BUTTON_INTERACTION)
			.label(ctx.translate("event-setup-login-button", None))
			.style(ButtonStyle::Success),
		CreateButton::new(LOGOUT_BUTTON_INTERACTION)
			.label(ctx.translate("event-setup-logout-button", None))
			.style(ButtonStyle::Danger),
	]);

	let message = CreateMessage::new()
		.content(ctx.translate("setup_login_message-message", None))
		.components(vec![action_row]);

	let reply = ctx
		.interaction
		.channel_id
		.send_message(&ctx.serenity_context, message)
		.await?;

	// Update the `setup_message_id`
	diesel::update(schema::guilds::table.find(guild_id.get()))
		.set(schema::guilds::login_message_id.eq(reply.id.get()))
		.execute(&mut connection)
		.await?;

	ctx.shout(ctx.translate("done", None)).await?;

	Ok(())
}
