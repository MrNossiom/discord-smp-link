//! Command to disconnect Discord and Google accounts together.

use crate::{
	constants::events,
	database::{
		models::{Guild, VerifiedMember},
		prelude::*,
		schema,
	},
	states::{InteractionResult, MessageComponentContext},
	translation::Translate,
};
use poise::serenity_prelude::{ButtonStyle, CollectComponentInteraction, RoleId};
use std::time::Duration;

/// Starts the dissociate accounts process
/// Function used in the login and the setup command
#[tracing::instrument(skip_all, fields(caller_id = %ctx.interaction.user.id))]
pub(crate) async fn logout(ctx: MessageComponentContext<'_>) -> InteractionResult {
	let mut member = ctx.guild_only_member();

	let Some(member_id) = VerifiedMember::with_ids(member.user.id, member.guild_id)
		.select(schema::verified_members::member_id)
		.first(&mut ctx.data.database.get().await?)
		.await
		.optional()?
	else {
		ctx.shout("Member does not exist").await?;

		return Ok(());
	};

	let reply = ctx
		.send(|reply| {
			reply
				.ephemeral(true)
				.content(ctx.translate("event-logout-warning", None))
				.components(|components| {
					components.create_action_row(|action_row| {
						action_row.create_button(|button| {
							button
								.label(ctx.translate("event-logout-disconnect-button", None))
								.custom_id(events::LOGOUT_OK_BUTTON_INTERACTION)
								.style(ButtonStyle::Danger)
						})
					})
				})
		})
		.await?;

	if let Some(interaction) = CollectComponentInteraction::new(&ctx)
		.message_id(reply.message().await?.id)
		.timeout(Duration::from_secs(60))
		.await
	{
		interaction.defer(&ctx).await?;

		match &*interaction.data.custom_id {
			events::LOGOUT_OK_BUTTON_INTERACTION => {}
			events::LOGOUT_CANCEL_BUTTON_INTERACTION => {}

			_ => unreachable!(),
		}
	} else {
		ctx.shout(ctx.translate("error-user-timeout", None)).await?;
	}

	db_dsl::delete(VerifiedMember::from_member_id(member_id))
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	let role_id: Option<u64> = Guild::with_id(member.guild_id)
		.select(schema::guilds::verified_role_id)
		.first(&mut ctx.data.database.get().await?)
		.await?;

	if let Some(role) = role_id.map(RoleId) {
		member.remove_role(&ctx, role).await?;
	}

	reply
		.edit(|msg| {
			msg.ephemeral(true)
				.content(ctx.translate("event-logout-success", None))
		})
		.await?;

	Ok(())
}
