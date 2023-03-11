//! Command to disconnect Discord and Google accounts together.

use crate::{
	database::{
		models::{Guild, VerifiedMember},
		prelude::*,
		schema,
	},
	states::{InteractionResult, MessageComponentContext},
	translation::Translate,
};
use poise::serenity_prelude::{ButtonStyle, CollectComponentInteraction, Member, RoleId};
use std::time::Duration;

/// Custom ID for the disconnect button
const CUSTOM_ID_DISCONNECT: &str = "event.logout.disconnect";

/// Starts the dissociate accounts process
/// Function used in the login and the setup command
#[tracing::instrument(skip_all, fields(caller_id = %ctx.interaction.user.id))]
pub(crate) async fn logout(ctx: MessageComponentContext<'_>) -> InteractionResult {
	let mut member = ctx.guild_only_member();

	let Some(member_id) =  VerifiedMember::with_ids(member.user.id, member.guild_id)
		.select(schema::verified_members::member_id)
		.first(&mut ctx.data.database.get().await?)
		.await.optional()? else
	{
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
								.custom_id(CUSTOM_ID_DISCONNECT)
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
			CUSTOM_ID_DISCONNECT => inner_logout(ctx, &mut member, member_id).await?,

			_ => unreachable!(),
		}
	} else {
		let get = ctx.translate("error-user-timeout", None);
		ctx.shout(get).await?;
	}

	Ok(())
}

/// Disconnects Discord and Google accounts together
async fn inner_logout(
	ctx: MessageComponentContext<'_>,
	member: &mut Member,
	member_id: i32,
) -> InteractionResult {
	diesel::delete(VerifiedMember::from_member_id(member_id))
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	let role_id: Option<u64> = Guild::with_id(member.guild_id)
		.select(schema::guilds::verified_role_id)
		.first(&mut ctx.data.database.get().await?)
		.await?;

	if let Some(role) = role_id.map(RoleId) {
		member.remove_role(&ctx, role).await?;
	}

	let get = ctx.translate("event-logout-success", None);
	ctx.shout(get).await?;

	Ok(())
}
