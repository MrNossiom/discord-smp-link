//! Command to disconnect Discord and Google accounts together.

use crate::{
	database::{models::VerifiedMember, prelude::*, schema, DieselError},
	states::{InteractionResult, MessageComponentContext},
	translation::Translate,
};
use anyhow::anyhow;
use diesel_async::RunQueryDsl;
use poise::serenity_prelude::{ButtonStyle, CollectComponentInteraction};
use std::time::Duration;

/// Custom ID for the disconnect button
const CUSTOM_ID_DISCONNECT: &str = "event.logout.disconnect";

/// Starts the dissociate accounts process
/// Function used in the login and the setup command
pub(crate) async fn logout(ctx: MessageComponentContext<'_>) -> InteractionResult {
	let member = ctx
		.interaction
		.member
		.as_ref()
		.ok_or_else(|| anyhow!("used only in guild"))?;

	let member_id: Option<i32> = match VerifiedMember::with_ids(&member.user.id, &member.guild_id)
		.select(schema::verified_members::member_id)
		.first(&mut ctx.data.database.get().await?)
		.await
	{
		Ok(x) => Some(x),
		Err(DieselError::NotFound) => None,
		Err(error) => return Err(error.into()),
	};

	let member_id = match member_id {
		Some(member_id) => member_id,
		None => {
			ctx.shout("Member does not exist").await?;

			return Ok(());
		}
	};

	let reply = ctx
		.send(|reply| {
			reply
				.ephemeral(true)
				.content(ctx.get("event-logout-warning", None))
				.components(|components| {
					components.create_action_row(|action_row| {
						action_row.create_button(|button| {
							button
								.label(ctx.get("event-logout-disconnect-button", None))
								.custom_id(CUSTOM_ID_DISCONNECT)
								.style(ButtonStyle::Danger)
						})
					})
				})
		})
		.await?;

	match CollectComponentInteraction::new(ctx.discord)
		.message_id(reply.message().await?.id)
		.timeout(Duration::from_secs(60))
		.await
	{
		Some(interaction) => {
			interaction.defer(ctx.discord).await?;

			match &*interaction.data.custom_id {
				CUSTOM_ID_DISCONNECT => inner_logout(ctx, member_id).await?,

				_ => unreachable!(),
			};
		}
		None => {
			let get = ctx.get("error-user-timeout", None);
			ctx.shout(get).await?;
		}
	}

	Ok(())
}

/// Disconnects Discord and Google accounts together
async fn inner_logout(ctx: MessageComponentContext<'_>, member_id: i32) -> InteractionResult {
	diesel::delete(VerifiedMember::from_member_id(member_id))
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	let get = ctx.get("event-logout-success", None);
	ctx.shout(get).await?;

	Ok(())
}
